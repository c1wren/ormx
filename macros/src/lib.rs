extern crate proc_macro;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::convert::TryFrom;
use std::fmt;
use syn::export::Formatter;
use syn::*;

mod get;
mod parse;
mod insert;
mod set;

fn connection_type() -> TokenStream2 {
    #[cfg(feature = "sqlite")]
    let ty = quote!(sqlx::SqliteConnection);
    #[cfg(feature = "mysql")]
    let ty = quote!(sqlx::MySqlConnection);
    #[cfg(feature = "postgres")]
    let ty = quote!(sqlx::PostgresConnection);
    return ty;
}

pub(crate) type Accessor = Option<Ident>;

pub(crate) enum HelperAttr {
    Generated,
    TableName(String),
    Rename(String),
    GetOne(Accessor),
    GetOptional(Accessor),
    GetMany(Accessor),
    Set(Accessor),
    PrimaryKey,
}

impl fmt::Display for HelperAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HelperAttr::Generated => write!(f, "#[generated]"),
            HelperAttr::TableName(table) => write!(f, "#[table = {:?}]", table),
            HelperAttr::Rename(rename) => write!(f, "rename = {:?}", rename),
            HelperAttr::GetOne(None) => write!(f, "get_one"),
            HelperAttr::GetOne(Some(func)) => write!(f, "get_one = {}", func),
            HelperAttr::GetOptional(None) => write!(f, "get_optional"),
            HelperAttr::GetOptional(Some(func)) => write!(f, "get_optional = {}", func),
            HelperAttr::GetMany(None) => write!(f, "get_many"),
            HelperAttr::GetMany(Some(func)) => write!(f, "get_many = {}", func),
            HelperAttr::Set(None) => write!(f, "set"),
            HelperAttr::Set(Some(func)) => write!(f, "set = {}", func),
            HelperAttr::PrimaryKey => write!(f, "primary_key"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct EntityField {
    get_one: Option<Ident>,
    get_optional: Option<Ident>,
    get_many: Option<Ident>,
    set: Option<Ident>,
    generated: bool,
    primary_key: bool,

    db_ident: String,
    rust_ident: Ident,
    ty: Type,
}

pub(crate) struct Entity {
    table_name: String,
    ident: Ident,
    fields: Vec<EntityField>,
    primary_key: Option<EntityField>,
}

impl Entity {
    pub(crate) fn generate_getters(&self) -> TokenStream2 {
        self.fields
            .iter()
            .flat_map(|field| {
                std::iter::empty()
                    .chain(
                        field
                            .get_one
                            .as_ref()
                            .map(|func| get::single(self, field, func)),
                    )
                    .chain(
                        field
                            .get_optional
                            .as_ref()
                            .map(|func| get::optional(self, field, func)),
                    )
                    .chain(
                        field
                            .get_many
                            .as_ref()
                            .map(|func| get::many(self, field, func)),
                    )
            })
            .collect()
    }

    fn generate_setters(&self) -> Result<TokenStream2> {
        self.fields
            .iter()
            .flat_map(|field| field.set.as_ref().map(|func| set::set(self, field, func)))
            .collect()
    }

    fn gen_insert(&self) -> TokenStream2 {
        let fields = self.fields.iter()
            .map(|EntityField { rust_ident, ty, .. }| quote!(#rust_ident));

        quote! {
            pub async fn insert(
                con: &mut #connection,
                #(#field_idents: #field_types),*
            ) -> sqlx::Result<Self> {

            }
        }
    }
}

fn derive_entity(input: DeriveInput) -> Result<TokenStream2> {
    let ty = &input.ident;
    let entity = Entity::try_from(&input)?;
    let getters = entity.generate_getters();
    let setters = entity.generate_setters()?;

    Ok(quote! {
        impl #ty {
            #getters
            #setters
        }
    })
}

#[proc_macro_derive(Entity, attributes(table, get_by, set, rename, ormx))]
pub fn derive_entity_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_entity(input) {
        Ok(out) => out,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

fn function_name(prefix: &str, field: &Ident, rename: &Option<Ident>) -> Ident {
    match rename {
        None => Ident::new(&format!("{}_{}", prefix, field), Span::call_site()),
        Some(accessor) => accessor.clone(),
    }
}
