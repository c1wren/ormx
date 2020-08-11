use crate::{Entity, EntityField};
use proc_macro2::Ident;
use quote::quote;
use syn::export::TokenStream2;
use syn::Result;

pub(crate) fn set(
    entity: &Entity,
    primary_key: &EntityField,
    field: &EntityField,
    fn_name: &Ident,
) -> Result<TokenStream2> {
    let query = format!(
        "UPDATE {} SET {} = ? WHERE {} = ?",
        entity.table_name, field.column_name, primary_key.column_name
    );

    let field_ty = &field.ty;
    let field_ident = &field.ident;
    let con = super::connection_type();
    let pkey = &primary_key.ident;

    Ok(quote! {
        pub async fn #fn_name<'e>(
            &mut self,
            con: &'e mut #con,
            value: #field_ty
        ) -> sqlx::Result<()> {
            sqlx::query!(#query, value, &self.#pkey)
                .execute(con)
                .await?;
            self.#field_ident = value;
        }
    })
}
