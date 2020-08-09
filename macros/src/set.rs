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

use crate::{Entity, EntityField, connection_type};
use proc_macro2::{Ident, Span};
use syn::export::TokenStream2;
use syn::{Error, Result};
use quote::quote;

/*
pub(crate) fn update() -> Result<TokenStream2> {
    let query = format!("UPDATE {} SET {} = ? WHERE {} = ?")

    Ok(quote! {
        pub async fn update(con: &mut #connection) -> sqlx::Result<()> {

        }
    })
}

 */

pub(crate) fn set(
    entity: &Entity,
    primary_key: &EntityField,
    field: &EntityField,
    fn_name: &Ident
) -> Result<TokenStream2> {
    /*
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
     */

    let query = format!(
        "UPDATE {} SET {} = ? WHERE {} = ?",
        entity.table_name,
        field.db_ident,
        primary_key.db_ident
    );

    let field_ty = &field.ty;
    let con = super::connection_type();
    let pkey = &primary_key.rust_ident;

    Ok(quote! {
        pub async fn #fn_name<'e>(
            &mut self,
            con: &'e mut #con,
            value: &#field_ty
        ) -> sqlx::Result<()> {
            sqlx::query!(#query, value, &self.#pkey)
                .execute(con)
                .await
        }
    })
}
