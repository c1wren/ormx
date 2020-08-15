use crate::Entity;
use proc_macro2::TokenStream;
use quote::quote;

pub fn delete(entity: &Entity) -> TokenStream {
    let query = format!(
        "DELETE FROM {} WHERE {} = ?",
        entity.table_name, entity.id.column_name
    );
    let pkey_ident = &entity.id.ident;
    let entity_ident = &entity.ident;

    quote! {
        impl #entity_ident {
            /// Delete a row from the database.
            pub async fn delete(
                self,
                con: impl sqlx::Executor<'_, Database=sqlx::MySql>
            ) -> sqlx::Result<()> {
                sqlx::query!(#query, self.#pkey_ident)
                    .execute(con)
                    .await?;

                Ok(())
            }
        }
    }
}
