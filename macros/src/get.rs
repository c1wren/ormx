use crate::{attrs::ConvertType, Entity, EntityField};
use itertools::Itertools;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

pub fn getters(entity: &Entity) -> TokenStream2 {
    let getters = entity
        .fields
        .iter()
        .flat_map(|field| {
            let get_one = field
                .get_one
                .as_ref()
                .map(|name| single(entity, field, name));
            let get_optional = field
                .get_optional
                .as_ref()
                .map(|name| optional(entity, field, name));
            let get_many = field
                .get_many
                .as_ref()
                .map(|name| many(entity, field, name));
            get_one
                .into_iter()
                .chain(get_optional.into_iter())
                .chain(get_many.into_iter())
        })
        .collect::<TokenStream2>();

    let get_all = get_all(entity);

    quote! {
        #get_all
        #getters
    }
}

fn get_all(entity: &Entity) -> TokenStream2 {
    let fn_name = match &entity.get_all {
        Some(ident) => ident,
        None => return quote!(),
    };
    let sql = build_query(entity, None);
    let vis = &entity.vis;

    quote! {
        #vis async fn #fn_name(
            con: &mut sqlx::PgConnection
        ) -> sqlx::Result<Vec<Self>> {
            sqlx::query_as!(Self, #sql)
                .fetch_all(con)
                .await
        }
    }
}

fn single(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let val = &field.ty;
    let vis = &entity.vis;
    let query = build_query(entity, Some(field));

    let by_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { *val as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&val) },
        None => quote! { val },
    };

    quote! {
        #vis async fn #fn_name(
            con: &mut sqlx::PgConnection,
            val: &#val
        ) -> sqlx::Result<Self> {
            sqlx::query_as!(Self, #query, #by_converter)
                .fetch_one(con)
                .await
        }
    }
}

fn optional(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let val = &field.ty;
    let vis = &entity.vis;
    let query = build_query(entity, Some(field));

    let by_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { *val as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&val) },
        None => quote! { val },
    };

    quote! {
        #vis async fn #fn_name(
            con: &mut sqlx::PgConnection,
            val: &#val
        ) -> sqlx::Result<Option<Self>> {
            sqlx::query_as!(Self, #query, #by_converter)
                .fetch_optional(con)
                .await
        }
    }
}

fn many(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let val = &field.ty;
    let vis = &entity.vis;
    let query = build_query(entity, Some(field));

    let by_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { *val as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&val) },
        None => quote! { val },
    };

    quote! {
        #vis async fn #fn_name(
            con: &mut sqlx::PgConnection,
            val: &#val
        ) -> sqlx::Result<Vec<Self>> {
            sqlx::query_as!(Self, #query, #by_converter)
                .fetch_all(con)
                .await
        }
    }
}

fn build_query(entity: &Entity, val: Option<&EntityField>) -> String {
    let columns = entity
        .fields
        .iter()
        .map(EntityField::fmt_for_select)
        .join(",");

    if let Some(val) = val {
        format!(
            "SELECT {} FROM {} WHERE {} = $1",
            columns, entity.table_name, val.column_name
        )
    } else {
        format!("SELECT {} FROM {}", columns, entity.table_name)
    }
}
