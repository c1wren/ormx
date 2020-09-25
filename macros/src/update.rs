use crate::{attrs::ConvertType, Entity};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

pub fn update(entity: &Entity) -> TokenStream {
    let sql = format!(
        "UPDATE {} SET {} WHERE {} = $1",
        entity.table_name,
        entity
            .updatable_fields()
            .enumerate()
            .map(|(index, field)| format!(
                "{} = ${}",
                field.column_name.replace("r#", ""),
                index + 2
            ))
            .join(", "),
        entity.id.column_name
    );

    let id_ident = &entity.id.ident;
    let vis = &entity.vis;

    let updatable_fields = entity.updatable_fields().map(|field| {
        let ident = &field.ident;
        let value = match &field.convert {
            Some(ConvertType::As(t)) => quote! { self.#ident as #t },
            Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&self.#ident) },
            None => quote! { self.#ident },
        };

        if field.custom_type {
            quote! { #value as _ }
        } else {
            value
        }
    });

    quote! {
        #vis async fn update(
            &self,
            con: &mut sqlx::PgConnection
        ) -> sqlx::Result<()> {
            sqlx::query!(#sql, self.#id_ident, #(#updatable_fields,)*).execute(con).await?;
            Ok(())
        }
    }
}
