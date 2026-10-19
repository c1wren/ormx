use crate::methods::{self, MethodNames};
use crate::transform::{bind_transform_params, process_transform, TransformBinding};
use crate::{attrs::ConvertType, Entity, EntityField};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn setters(entity: &Entity) -> TokenStream2 {
    entity
        .fields
        .iter()
        .flat_map(|field| methods::setter(entity, field).map(|names| setter(entity, field, &names)))
        .collect()
}

fn setter(entity: &Entity, field: &EntityField, method_names: &MethodNames) -> TokenStream2 {
    let fn_name = &method_names.main;
    let primary_keys: Vec<&EntityField> = entity.fields.iter().filter(|x| x.is_key).collect();
    let (set_expr, transform_bindings) = match build_set_expr(field, primary_keys.len()) {
        Ok(parts) => parts,
        Err(err) => return err.to_compile_error(),
    };

    let mut query = format!(
        "UPDATE {} SET {} = {} WHERE",
        entity.table_name, field.column_name, set_expr
    );
    for (index, key) in primary_keys.iter().enumerate() {
        query.push_str(format!(" {} = ${}", key.column_name, index + 2).as_str());
        if index + 1 != primary_keys.len() {
            query.push_str(" AND ")
        }
    }

    let field_ty = &field.ty;
    let field_ident = &field.ident;
    let vis = &entity.vis;
    let transform_params = bind_transform_params(&transform_bindings);
    let transform_binds = &transform_params.expressions;

    let value_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { value as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&value) },
        None => quote! { value },
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<(), #e_type>)
    } else {
        quote!(Result<(), sqlx::Error>)
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

    let update_query = transform_params.wrap(quote! {
        sqlx::query!(#query, #value_converter, #(#ident_keys),* #(, #transform_binds)*)
            .execute(&mut *conn)
            .await
    });

    let before_update = if let Some(before_fn) = &entity.before_update {
        quote!(
            #before_fn(self, context, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let after_update = if let Some(after_fn) = &entity.after_update {
        quote!(
            let previous = self.clone();

            #update_query?;
            self.#field_ident = value;

            #after_fn(self, previous, context, conn).await?;
        )
    } else {
        quote!(
            #update_query?;
            self.#field_ident = value;
        )
    };

    let no_trigger_variant = if let Some(fn_name_no_trigger) = &method_names.no_trigger {
        quote! {
            /// Updates the row in the database specified by the primary key.
            ///
            /// This will update every field except the primary key field. `Patch` should be used if only updating some of the fields.
            ///
            /// Does not call the before and after triggers.
            #vis async fn #fn_name_no_trigger(
                &mut self,
                conn: &mut sqlx::PgConnection,
                value: #field_ty
            ) -> #ret_type {
                #update_query?;
                self.#field_ident = value;

                Ok(())
            }
        }
    } else {
        quote!()
    };

    let context_variant = if let (Some(context_type), Some(fn_name_with_context)) =
        (&entity.context_type, &method_names.with_context)
    {
        quote! {
            #vis async fn #fn_name_with_context(
                &mut self,
                conn: &mut sqlx::PgConnection,
                value: #field_ty,
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
        #vis async fn #fn_name(
            &mut self,
            conn: &mut sqlx::PgConnection,
            value: #field_ty
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

fn build_set_expr(
    field: &EntityField,
    key_count: usize,
) -> syn::Result<(String, Vec<TransformBinding>)> {
    if let Some(transform_set) = &field.transform_set {
        let (expr, count) = process_transform(transform_set, "$1", 1 + key_count)?;
        let bindings = field
            .transform_set_params
            .as_ref()
            .map(|params_fn| {
                vec![TransformBinding {
                    params_fn: params_fn.clone(),
                    count,
                }]
            })
            .unwrap_or_default();
        Ok((expr, bindings))
    } else {
        Ok(("$1".to_string(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setter_transform_parameters_follow_all_composite_key_parameters() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[ormx(table = "records")]
            struct Record {
                #[ormx(key)] id: i32,
                #[ormx(key)] version: i32,
                #[ormx(
                    transform_set = "concat($param1::text, $field::text, $param2::text)",
                    transform_set_params = "params"
                )]
                value: String,
            }
        };
        let entity = Entity::try_from(input).unwrap();
        let (sql, bindings) = build_set_expr(&entity.fields[2], 2).unwrap();
        assert_eq!(sql, "concat($4::text, $1::text, $5::text)");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].count, 2);
    }
}
