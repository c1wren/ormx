use std::collections::HashMap;

use crate::transform::{bind_transform_params, build_select_clause, TransformBinding};
use crate::{attrs::ConvertType, Entity, EntityField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Ident};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockMode {
    Update,
    NoKeyUpdate,
    Share,
    KeyShare,
}

impl LockMode {
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Update => "update",
            Self::NoKeyUpdate => "no_key_update",
            Self::Share => "share",
            Self::KeyShare => "key_share",
        }
    }

    fn sql(self) -> &'static str {
        match self {
            Self::Update => "FOR UPDATE",
            Self::NoKeyUpdate => "FOR NO KEY UPDATE",
            Self::Share => "FOR SHARE",
            Self::KeyShare => "FOR KEY SHARE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GetterKind {
    One,
    Optional,
    Many,
    All,
}

impl GetterKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::One => "get_one",
            Self::Optional => "get_optional",
            Self::Many => "get_many",
            Self::All => "get_all",
        }
    }

    fn is_collection(self) -> bool {
        matches!(self, Self::Many | Self::All)
    }
}

#[derive(Clone)]
pub struct LockingGetter {
    pub kind: GetterKind,
    pub mode: LockMode,
    pub name: Ident,
}

pub fn parse_locking_getter(ident: &Ident) -> Option<(GetterKind, LockMode)> {
    let name = ident.to_string();
    let (prefix, suffix) = name.split_once("_for_")?;
    let kind = match prefix {
        "get_one" => GetterKind::One,
        "get_optional" => GetterKind::Optional,
        "get_many" => GetterKind::Many,
        "get_all" => GetterKind::All,
        _ => return None,
    };
    let mode = match suffix {
        "update" => LockMode::Update,
        "no_key_update" => LockMode::NoKeyUpdate,
        "share" => LockMode::Share,
        "key_share" => LockMode::KeyShare,
        _ => return None,
    };
    Some((kind, mode))
}

struct GetterDeclaration<'a> {
    field: Option<&'a EntityField>,
    name: &'a Ident,
    kind: GetterKind,
    lock: Option<LockMode>,
}

fn declarations(entity: &Entity) -> impl Iterator<Item = GetterDeclaration<'_>> {
    let all = entity.get_all.iter().map(|name| GetterDeclaration {
        field: None,
        name,
        kind: GetterKind::All,
        lock: None,
    });
    let locking_all = entity
        .locking_getters
        .iter()
        .map(|getter| GetterDeclaration {
            field: None,
            name: &getter.name,
            kind: getter.kind,
            lock: Some(getter.mode),
        });
    let fields = entity.fields.iter().flat_map(|field| {
        let plain = [
            (&field.get_one, GetterKind::One),
            (&field.get_optional, GetterKind::Optional),
            (&field.get_many, GetterKind::Many),
        ]
        .into_iter()
        .filter_map(move |(name, kind)| {
            name.as_ref().map(|name| GetterDeclaration {
                field: Some(field),
                name,
                kind,
                lock: None,
            })
        });
        let locking = field
            .locking_getters
            .iter()
            .map(move |getter| GetterDeclaration {
                field: Some(field),
                name: &getter.name,
                kind: getter.kind,
                lock: Some(getter.mode),
            });
        plain.chain(locking)
    });
    all.chain(locking_all).chain(fields)
}

pub fn getters(entity: &Entity) -> syn::Result<TokenStream> {
    let mut names = crate::methods::mutation_names(entity)
        .map(|name| {
            (
                name.to_string().trim_start_matches("r#").to_owned(),
                name.span(),
            )
        })
        .collect::<HashMap<_, _>>();
    declarations(entity).try_fold(TokenStream::new(), |mut methods, declaration| {
        let name = declaration.name;
        let key = name.to_string().trim_start_matches("r#").to_owned();
        if let Some(previous) = names.insert(key.clone(), name.span()) {
            let mut error = Error::new(
                name.span(),
                format!("conflicting generated getter name `{key}`"),
            );
            error.combine(Error::new(
                previous,
                "a method with this name is already generated",
            ));
            return Err(error);
        }
        methods.extend(getter(entity, declaration)?);
        Ok(methods)
    })
}

fn getter(entity: &Entity, declaration: GetterDeclaration<'_>) -> syn::Result<TokenStream> {
    let GetterDeclaration {
        field,
        name,
        kind,
        lock,
    } = declaration;
    let (query, bindings) = build_select_query(entity, field, kind, lock)?;
    let transform_params = bind_transform_params(&bindings);
    let transform_binds = &transform_params.expressions;
    let vis = &entity.vis;
    let connection_arg = if lock.is_some() {
        quote!(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>)
    } else {
        quote!(conn: &mut sqlx::PgConnection)
    };
    let executor = if lock.is_some() {
        quote!(&mut **tx)
    } else {
        quote!(conn)
    };
    let value_arg = field.map(|field| {
        let ty = &field.ty;
        quote!(, val: &#ty)
    });
    let value_bind = field.map(|field| {
        let value = match &field.convert {
            Some(ConvertType::As(ty)) => quote!(*val as #ty),
            Some(ConvertType::Function(convert_fn)) => quote!(#convert_fn(&val)),
            None => quote!(val),
        };
        quote!(, #value)
    });
    let (value_type, fetch) = match kind {
        GetterKind::One => (quote!(Self), quote!(fetch_one)),
        GetterKind::Optional => (quote!(Option<Self>), quote!(fetch_optional)),
        GetterKind::Many | GetterKind::All => (quote!(Vec<Self>), quote!(fetch_all)),
    };
    let call = transform_params.wrap(quote! {
        sqlx::query_as!(Self, #query #value_bind #(, #transform_binds)*)
            .#fetch(#executor)
            .await
    });
    let (ret_type, body) = if let Some(error) = &entity.error_type {
        (quote!(Result<#value_type, #error>), quote!(Ok(#call?)))
    } else {
        (quote!(Result<#value_type, sqlx::Error>), call)
    };
    let docs = lock.map(|mode| {
        let text = format!(
            "Selects rows using `{}` in the caller's transaction. Locks are held until transaction end or rollback of the savepoint that acquired them. The caller owns cross-table lock order and business validation.",
            mode.sql()
        );
        quote!(#[doc = #text])
    });
    Ok(quote! {
        #docs
        #vis async fn #name(#connection_arg #value_arg) -> #ret_type {
            #body
        }
    })
}

fn build_select_query(
    entity: &Entity,
    field: Option<&EntityField>,
    kind: GetterKind,
    lock: Option<LockMode>,
) -> syn::Result<(String, Vec<TransformBinding>)> {
    let (columns, bindings, _) = build_select_clause(&entity.fields, usize::from(field.is_some()))?;
    let mut query = format!("SELECT {} FROM {}", columns, entity.table_name);
    if let Some(field) = field {
        let column = if lock.is_some() {
            field.column_name.replace("r#", "")
        } else {
            field.column_name.clone()
        };
        query.push_str(&format!(" WHERE {column} = $1"));
    }
    if let Some(mode) = lock {
        if kind.is_collection() {
            let keys = entity
                .fields
                .iter()
                .filter(|field| field.is_key)
                .map(|field| {
                    format!(
                        "{}.{} ASC",
                        entity.table_name,
                        field.column_name.replace("r#", "")
                    )
                })
                .collect::<Vec<_>>();
            query.push_str(&format!(" ORDER BY {}", keys.join(", ")));
        }
        query.push(' ');
        query.push_str(mode.sql());
    }
    Ok((query, bindings))
}

#[cfg(test)]
mod tests;
