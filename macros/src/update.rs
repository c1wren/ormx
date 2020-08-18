use crate::Entity;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

pub fn update(entity: &Entity) -> TokenStream {
    let sql = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        entity.table_name,
        entity
            .updatable_fields()
            .map(|field| format!("{} = ?", field.column_name))
            .join(","),
        entity.id.column_name
    );

    let id_ident = &entity.id.ident;
    let vis = &entity.vis;
    let updatable_fields = entity.updatable_fields().map(|field| &field.ident);

    quote! {
        #vis async fn update(
            &self,
            con: impl sqlx::Executor<'_, Database=sqlx::Postgres>
        ) -> sqlx::Result<()> {
            sqlx::query!(#sql, #(self.#updatable_fields,)* self.#id_ident).execute(con).await?;
            Ok(())
        }
    }
}
