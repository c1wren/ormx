use crate::{attrs::ConvertType, entity::EntityField, Entity};
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

    let columns = entity
        .fields
        .iter()
        .map(EntityField::fmt_for_select)
        .join(", ");

    let before_value_sql = format!(
        "SELECT {} FROM {} WHERE {} = $1",
        columns, entity.table_name, entity.id.column_name
    );

    let before_update = if let Some(before_fn) = &entity.before_update {
        quote!(
            #before_fn(self, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let sqlx_call = quote!(sqlx::query!(#sql, self.#id_ident, #(#updatable_fields,)*));

    let has_trigger = entity.after_update.is_some() || entity.before_update.is_some();

    let after_update = if let Some(after_fn) = &entity.after_update {
        quote!(
            let previous = sqlx::query_as!(Self, #before_value_sql, self.id)
                .fetch_one(&mut *conn)
                .await?;

            #sqlx_call.execute(&mut *conn).await?;

            #after_fn(self, previous, conn).await?;
        )
    } else {
        quote!(
            #sqlx_call.execute(conn).await?;
        )
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<(), #e_type>)
    } else {
        quote!(Result<(), sqlx::Error>)
    };

    let no_trigger_variant = if has_trigger {
        quote! {
            /// Updates the row in the database specified by the primary key.
            ///
            /// This will update every field except the primary key field. `Patch` should be used if only updating some of the fields.
            ///
            /// Does not call the before and after triggers.
            #vis async fn no_trigger_update(
                &self,
                conn: &mut sqlx::PgConnection
            ) -> #ret_type {
                #sqlx_call.execute(conn).await?;

                Ok(())
            }
        }
    } else {
        quote!()
    };

    quote! {
            /// Updates the row in the database specified by the primary key.
            ///
            /// This will update every field except the primary key field. `Patch` should be used if only updating some of the fields.
            #vis async fn update(
            &self,
            conn: &mut sqlx::PgConnection
        ) -> #ret_type {
            #before_update

            #after_update

            Ok(())
        }

        #no_trigger_variant
    }
}
