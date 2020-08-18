use crate::{Entity, EntityField};
use proc_macro2::Ident;
use quote::quote;
use syn::export::TokenStream2;

pub fn setters(entity: &Entity) -> TokenStream2 {
    entity
        .fields
        .iter()
        .flat_map(|field| field.set.as_ref().map(|name| setter(entity, field, name)))
        .collect()
}

fn setter(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let query = format!(
        "UPDATE {} SET {} = $1 WHERE {} = $2",
        entity.table_name, field.column_name, entity.id.column_name
    );

    let field_ty = &field.ty;
    let field_ident = &field.ident;
    let pkey = &entity.id.ident;
    let vis = &entity.vis;

    quote! {
        #vis async fn #fn_name(
            &mut self,
            con: impl sqlx::Executor<'_, Database=sqlx::Postgres>,
            value: #field_ty
        ) -> sqlx::Result<()> {
            sqlx::query!(#query, value, &self.#pkey)
                .execute(con)
                .await?;
            self.#field_ident = value;
            Ok(())
        }
    }
}
