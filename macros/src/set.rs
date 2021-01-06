use crate::{attrs::ConvertType, Entity, EntityField};
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

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

    let value_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { value as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&value) },
        None => quote! { value },
    };

    quote! {
        #vis async fn #fn_name(
            &mut self,
            con: &mut sqlx::PgConnection,
            value: #field_ty
        ) -> sqlx::Result<()> {
            sqlx::query!(#query, #value_converter, &self.#pkey)
                .execute(con)
                .await?;
            self.#field_ident = value;
            Ok(())
        }
    }
}
