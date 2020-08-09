/*
pub(crate) fn generate() {
    quote! {
        pub async fn #fn_name(
            con: &mut #con,
            value: &#val
        ) -> sqlx::Result<Self> {
            sqlx::query!("")
        }
    }
}*/

use crate::{Entity, EntityField};
use proc_macro2::{Ident, Span};
use syn::export::TokenStream2;
use syn::{Error, Result};

pub(crate) fn update() -> Result<TokenStream2> {
    let query = format!("UPDATE {} SET ")

    Ok(quote! {
        pub async fn update(con: &mut #connection) -> sqlx::Result<()> {

        }
    })
}

pub(crate) fn set(entity: &Entity, field: &EntityField, func: &Ident) -> Result<TokenStream2> {
    let primary_key = entity.primary_key.as_ref().ok_or_else(|| {
        Error::new(
            Span::call_site(),
            "#[ormx(set)] only works for entities with a primary key",
        )
    })?;

    let query = format!(
        "UPDATE {} SET {} = ? WHERE {} = ?",
        entity.table_name, field.db_ident, primary_key.db_ident
    );

    quote! {
        pub async fn #func(&mut self, )

    }

    unimplemented!()
}
