use crate::{Entity, connection_type};
use proc_macro2::TokenStream;
use quote::quote;

pub fn delete(entity: &Entity) -> TokenStream {
    let pkey = match &entity.primary_key {
        Some(pkey) => pkey,
        None => return quote!()
    };
    let query = format!(
        "DELETE FROM {} WHERE {} = ?",
        entity.table_name,
        pkey.column_name
    );
    let pkey_ident = &pkey.ident;
    let con = connection_type();

    quote! {
        pub async fn delete(self, con: &mut #con) -> sqlx::Result<()> {
            sqlx::query!(#query, self.#pkey_ident).execute(con).await?;
            Ok(())
        }
    }
}