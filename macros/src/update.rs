use crate::{attrs::ConvertType, entity::EntityField, Entity};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn update(entity: &Entity) -> TokenStream {
    let length = entity.updatable_fields().count();
    if length == 0 {
        return quote! {};
    }

    let primary_keys: Vec<&EntityField> = entity.fields.iter().filter(|x| x.is_key).collect();
    let mut where_part = String::new();
    for (index, key) in primary_keys.iter().enumerate() {
        where_part.push_str(format!("{} = ${}", key.column_name, index + 1).as_str());
        if index + 1 != primary_keys.len() {
            where_part.push_str(" AND ")
        }
    }

    let sql = format!(
        "UPDATE {} SET {} WHERE {where_part}",
        entity.table_name,
        entity
            .updatable_fields()
            .enumerate()
            .map(|(index, field)| format!(
                "{} = ${}",
                field.column_name.replace("r#", ""),
                index + primary_keys.len() + 1
            ))
            .join(", "),
    );

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

    let mut before_value_sql = format!("SELECT {} FROM {} WHERE", columns, entity.table_name);
    for (index, key) in primary_keys.iter().enumerate() {
        before_value_sql.push_str(format!(" {} = ${}", key.column_name, index + 1).as_str());
        if index + 1 != primary_keys.len() {
            before_value_sql.push_str(" AND ")
        }
    }

    let before_update = if let Some(before_fn) = &entity.before_update {
        quote!(
            #before_fn(self, context, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let ident_keys: Vec<&Ident> = primary_keys.iter().map(|x| &x.ident).collect();

    let sqlx_call = quote!(sqlx::query!(#sql, #(self.#ident_keys),*, #(#updatable_fields,)*));

    let has_trigger = entity.after_update.is_some() || entity.before_update.is_some();

    let after_update = if let Some(after_fn) = &entity.after_update {
        quote!(
            let previous = sqlx::query_as!(Self, #before_value_sql, #(self.#ident_keys),*)
                .fetch_one(&mut *conn)
                .await?;

            #sqlx_call.execute(&mut *conn).await?;

            #after_fn(self, previous, context, conn).await?;
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

    let fn_name = &entity.update;

    let context_variant = if let Some(context_type) = &entity.context_type {
        let fn_name_with_context =
            Ident::new(&format!("{}_with_context", fn_name), Span::call_site());

        quote! {
            /// Updates the row in the database specified by the keys
            ///
            /// Updates all fields of row, even ones that have not changed. Use Patch if this is not desired behavior.
            #vis async fn #fn_name_with_context(
                &self,
                conn: &mut sqlx::PgConnection,
                context: Option<&#context_type>
            ) -> #ret_type {
                #before_update

                #after_update

                Ok(())
            }
        }
    } else {
        quote! {}
    };

    let ty = if entity.context_type.is_none() {
        quote! { () }
    } else {
        quote! { _ }
    };

    quote! {
        /// Updates the row in the database specified by the keys
        ///
        /// Updates all fields of row, even ones that have not changed. Use Patch if this is not desired behavior.
        #vis async fn #fn_name(
            &self,
            conn: &mut sqlx::PgConnection
        ) -> #ret_type {
            let context = None::<#ty>;

            #before_update

            #after_update

            Ok(())
        }

        #context_variant

        #no_trigger_variant
    }
}
