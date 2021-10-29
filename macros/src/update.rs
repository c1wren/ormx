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

    let before_update = if let Some(before_fn) = &entity.before_update {
        quote!(
            #before_fn(self, &mut *con).await?;
        )
    } else {
        quote!()
    };

    let sqlx_call = quote!(sqlx::query!(#sql, self.#id_ident, #(#updatable_fields,)*));

    let after_update = if let Some(after_fn) = &entity.after_update {
        quote!(
            #sqlx_call.execute(&mut *con).await?;

            #after_fn(self, con).await?;
        )
    } else {
        quote!(
            #sqlx_call.execute(con).await?;
        )
    };

    quote! {
        /// Updates a given row in the database by updating all fields, even if some fields haven't been changed.
        #vis async fn update(
            &self,
            con: &mut sqlx::PgConnection
        ) -> sqlx::Result<()> {
            #before_update

            #after_update

            Ok(())
        }

        /// Updates a given row in the database by updating all fields, even if some fields haven't been changed.
        ///
        /// Does not call the before and after triggers.
        #vis async fn no_trigger_update(
            &self,
            con: &mut sqlx::PgConnection
        ) -> sqlx::Result<()> {
            #sqlx_call.execute(con).await?;

            Ok(())
        }

    }
}
