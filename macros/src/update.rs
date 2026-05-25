use crate::transform::{
    build_select_clause, process_transform, transform_bind_expressions, TransformBinding,
};
use crate::{attrs::ConvertType, entity::EntityField, Entity};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn update(entity: &Entity) -> TokenStream {
    let updatable_fields = entity.updatable_fields().collect::<Vec<_>>();
    if updatable_fields.is_empty() {
        return quote! {};
    }

    let primary_keys: Vec<&EntityField> = entity.fields.iter().filter(|x| x.is_key).collect();
    let (set_clause, transform_bindings) =
        match build_set_clause(&updatable_fields, primary_keys.len()) {
            Ok(parts) => parts,
            Err(err) => return err.to_compile_error(),
        };

    let mut where_part = String::new();
    for (index, key) in primary_keys.iter().enumerate() {
        where_part.push_str(format!("{} = ${}", key.column_name, index + 1).as_str());
        if index + 1 != primary_keys.len() {
            where_part.push_str(" AND ")
        }
    }

    let sql = format!(
        "UPDATE {} SET {} WHERE {where_part}",
        entity.table_name, set_clause,
    );

    let vis = &entity.vis;

    let updatable_field_values = updatable_fields
        .iter()
        .map(|field| {
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
        })
        .collect::<Vec<_>>();

    let (select_columns, select_bindings, _) =
        match build_select_clause(&entity.fields, primary_keys.len()) {
            Ok(parts) => parts,
            Err(err) => return err.to_compile_error(),
        };

    let mut before_value_sql =
        format!("SELECT {} FROM {} WHERE", select_columns, entity.table_name);
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

    let ident_keys: Vec<_> = primary_keys
        .iter()
        .map(|k| {
            let ident = &k.ident;
            if k.custom_type {
                quote!(self.#ident as _)
            } else {
                quote!(self.#ident)
            }
        })
        .collect();

    let transform_binds = transform_bind_expressions(&transform_bindings);
    let select_transform_binds = transform_bind_expressions(&select_bindings);
    let has_trigger = entity.after_update.is_some() || entity.before_update.is_some();

    let after_update = if let Some(after_fn) = &entity.after_update {
        quote!(
            let previous = sqlx::query_as!(
                Self,
                #before_value_sql,
                #(#ident_keys),*
                #(, #select_transform_binds)*
            )
            .fetch_one(&mut *conn)
            .await?;

            sqlx::query!(
                #sql,
                #(#ident_keys),*,
                #(#updatable_field_values),*
                #(, #transform_binds)*
            )
            .execute(&mut *conn)
            .await?;

            #after_fn(self, previous, context, conn).await?;
        )
    } else {
        quote!(
            sqlx::query!(
                #sql,
                #(#ident_keys),*,
                #(#updatable_field_values),*
                #(, #transform_binds)*
            )
            .execute(conn)
            .await?;
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
                sqlx::query!(
                    #sql,
                    #(#ident_keys),*,
                    #(#updatable_field_values),*
                    #(, #transform_binds)*
                )
                .execute(conn)
                .await?;
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

    let context = if entity.context_type.is_none() {
        if entity.before_update.is_some() || entity.after_update.is_some() {
            quote! { let context = None::<()>; }
        } else {
            quote! {}
        }
    } else {
        quote! { let context = None::<_>; }
    };

    quote! {
        /// Updates the row in the database specified by the keys
        ///
        /// Updates all fields of row, even ones that have not changed. Use Patch if this is not desired behavior.
        #vis async fn #fn_name(
            &self,
            conn: &mut sqlx::PgConnection
        ) -> #ret_type {
            #context
            #before_update
            #after_update
            Ok(())
        }

        #context_variant

        #no_trigger_variant
    }
}

fn build_set_clause(
    fields: &[&EntityField],
    key_count: usize,
) -> syn::Result<(String, Vec<TransformBinding>)> {
    let primary_param_count = fields.len() + key_count;
    let mut param_offset = primary_param_count;
    let mut bindings = Vec::new();
    let mut clauses = Vec::with_capacity(fields.len());

    for (index, field) in fields.iter().enumerate() {
        let value_placeholder = format!("${}", key_count + index + 1);
        let expr = if let Some(transform_set) = &field.transform_set {
            let (expr, count) = process_transform(transform_set, &value_placeholder, param_offset)?;
            param_offset += count;
            if let Some(params_fn) = &field.transform_set_params {
                bindings.push(TransformBinding {
                    params_fn: params_fn.clone(),
                    count,
                });
            }
            expr
        } else {
            value_placeholder
        };

        clauses.push(format!(
            "{} = {}",
            field.column_name.replace("r#", ""),
            expr
        ));
    }

    Ok((clauses.join(", "), bindings))
}
