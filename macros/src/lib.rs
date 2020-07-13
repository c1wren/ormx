extern crate proc_macro;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::convert::TryFrom;
use syn::*;

mod get;
mod parse;
// mod set;

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
}

pub(crate) struct EntityField {
    get_one: Option<Accessor>,
    get_optional: Option<Accessor>,
    get_many: Option<Accessor>,
    set: Option<Accessor>,

    db_ident: String,
    rust_ident: Ident,
    ty: Type,
}

pub(crate) struct Entity {
    table_name: String,
    ident: Ident,
    fields: Vec<EntityField>,
}

fn derive_entity(input: DeriveInput) -> Result<TokenStream2> {
    let entity = Entity::try_from(input)?;
    /*
    let ty = input.ident.clone();
    let entity = Entity::try_from(input)?;
    let getters = entity
        .fields
        .iter()
        .map(|field| get::generate(&entity, field))
        .collect::<Vec<_>>();
    Ok(quote! {
        impl #ty {
            #(#getters)*
        }
    })*/
    unimplemented!()
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

fn function_name(prefix: &str, field_ident: &Ident, accessor: &Accessor) -> Ident {
    match accessor {
        None => Ident::new(&format!("{}_{}", prefix, field_ident), Span::call_site()),
        Some(accessor) => accessor.clone(),
    }
}
