use crate::transform::{
    build_select_clause, count_transform_params, runtime_transform_expr, transform_bind_expressions,
};
use crate::{attrs::ConvertType, Entity, EntityField};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub fn patch(entity: &Entity) -> TokenStream {
    let patch_struct_ident = match &entity.patch {
        Some(ident) => ident,
        None => return quote!(),
    };

    if entity.patchable_fields().count() == 0 {
        panic!("#[ormx(patchable)] does not apply to any field!");
    }

    let patch_struct = patch_struct(entity, patch_struct_ident);
    let methods = methods(entity, patch_struct_ident);

    quote! {
        #patch_struct
        #methods
    }
}

fn patch_struct(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let vis = &entity.vis;
    let fields = entity
        .patchable_fields()
        .map(|EntityField { ident, ty, .. }| {
            quote! {
                #[allow(dead_code)]
                #vis #ident: Option<#ty>
            }
        });

    let setters = entity
        .patchable_fields()
        .map(|EntityField { ident, ty, set, .. }| {
            let setter = set.clone().unwrap_or_else(|| {
                Ident::new(
                    &format!("set_{}", ident.to_string().replace("r#", "")),
                    Span::call_site(),
                )
            });
            quote!(#vis fn #setter(mut self, value: #ty) -> Self {
                self.#ident = Some(value);
                self
            })
        });

    quote! {
        #[derive(Default, Clone, Debug)]
        #vis struct #patch_struct_ident {
            #(#fields),*
        }

        impl #patch_struct_ident {
            #(#setters)*
        }
    }
}

fn methods(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let patchable_fields = entity
        .patchable_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    if patchable_fields.is_empty() {
        return quote! {};
    }

    let primary_keys: Vec<&EntityField> = entity.fields.iter().filter(|x| x.is_key).collect();
    let primary_key_idents: Vec<&Ident> = primary_keys.iter().map(|x| &x.ident).collect();
    let entity_ident = &entity.ident;
    let table_name = &entity.table_name;
    let vis = &entity.vis;

    let column_building = entity.patchable_fields().map(|field| {
        let ident = &field.ident;
        let column_name = field.column_name.replace("r#", "");

        if let Some(transform_set) = &field.transform_set {
            let max_param = count_transform_params(transform_set);
            let transform_expr = runtime_transform_expr(
                transform_set,
                quote!(format!("${}", value_index)),
                quote!(transform_param_offset),
                max_param,
            );
            quote! {
                if self.#ident.is_some() {
                    let expr = #transform_expr;
                    columns.push(format!("{} = {}", #column_name, expr));
                    value_index += 1;
                    transform_param_offset += #max_param;
                }
            }
        } else {
            quote! {
                if self.#ident.is_some() {
                    columns.push(format!("{} = ${}", #column_name, value_index));
                    value_index += 1;
                }
            }
        }
    });

    let binding = entity.patchable_fields().map(|field| {
        let ident = &field.ident;
        let value_getter = match &field.convert {
            Some(ConvertType::As(t)) => quote! { *value as #t },
            Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&value) },
            None => quote! { value },
        };

        quote! {
            if let Some(value) = self.#ident.as_ref() {
                query = query.bind(#value_getter);
            }
        }
    });

    let transform_bindings = entity.patchable_fields().filter_map(|field| {
        let ident = &field.ident;
        let params_fn = field.transform_set_params.as_ref()?;
        let count = count_transform_params(field.transform_set.as_ref()?);
        let bind_params = (0..count).map(|i| {
            let index = syn::Index::from(i);
            quote! {
                query = query.bind(transform_params.#index);
            }
        });
        Some(quote! {
            if self.#ident.is_some() {
                let transform_params = #params_fn();
                #(#bind_params)*
            }
        })
    });

    let (columns, before_patch_bindings, _) =
        match build_select_clause(&entity.fields, primary_keys.len()) {
            Ok(parts) => parts,
            Err(err) => return err.to_compile_error(),
        };
    let before_patch_transform_binds = transform_bind_expressions(&before_patch_bindings);

    let mut patch_sql_statement = String::from("UPDATE {} SET {} WHERE");
    let mut before_patch_sql = format!("SELECT {} FROM {} WHERE", columns, entity.table_name);
    for (index, key) in primary_keys.iter().enumerate() {
        let condition = format!(" {} = ${}", key.column_name, index + 1);
        before_patch_sql.push_str(condition.as_str());
        patch_sql_statement.push_str(condition.as_str());
        if index + 1 != primary_keys.len() {
            before_patch_sql.push_str(" AND ");
            patch_sql_statement.push_str(" AND ");
        }
    }

    let has_trigger = entity.before_patch.is_some() || entity.after_patch.is_some();

    let before_patch = if let Some(before_fn) = &entity.before_patch {
        quote!(
            #before_fn(self, &patch, context, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let keys = primary_keys.iter().map(|x| (&x.ident, &x.ty));

    let mut items = Vec::new();
    for (ident, _) in keys.clone() {
        items.push(quote! { &self.#ident });
    }

    let mut fn_items = Vec::new();
    for (ident, ty) in keys {
        fn_items.push(quote! { #ident: &#ty });
    }

    let ident_keys: Vec<_> = primary_keys
        .iter()
        .map(|k| {
            let ident = &k.ident;
            if k.custom_type {
                quote!(self.#ident as _)
            } else {
                quote!(&self.#ident)
            }
        })
        .collect();

    let after_patch = if let Some(after_fn) = &entity.after_patch {
        quote!(
            let previous = sqlx::query_as!(
                Self,
                #before_patch_sql,
                #(#ident_keys),*
                #(, #before_patch_transform_binds)*
            )
            .fetch_one(&mut *conn)
            .await?;

            #patch_struct_ident::patch(&patch, &mut *conn, #( #items ),*).await?;

            #(if let Some(new_value) = patch.#patchable_fields {
                self.#patchable_fields = new_value;
            })*

            #after_fn(self, previous, context, conn).await?;
        )
    } else {
        quote!(
            #patch_struct_ident::patch(&patch, conn, #( #items ),*).await?;

            #(if let Some(new_value) = patch.#patchable_fields {
                self.#patchable_fields = new_value;
            })*
        )
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<(), #e_type>)
    } else {
        quote!(Result<(), sqlx::Error>)
    };

    let no_trigger_variant = if has_trigger {
        quote! {
            /// Updates the row as specified by the entity's primary key with only the fields that are included in the patch.
            ///
            /// Does not call the before and after triggers.
            #vis async fn no_trigger_patch(
                &mut self,
                conn: &mut sqlx::PgConnection,
                patch: #patch_struct_ident,
            ) -> #ret_type {
                #patch_struct_ident::patch(&patch, conn, #( #items ),*).await?;

                #(if let Some(new_value) = patch.#patchable_fields {
                    self.#patchable_fields = new_value;
                })*

                Ok(())
            }
        }
    } else {
        quote!()
    };

    let context_variant = if let Some(context_type) = &entity.context_type {
        quote! {
            /// Updates the row as specified by the entity's keys with only the fields that are included in the patch.
            #vis async fn patch_with_context(
                &mut self,
                conn: &mut sqlx::PgConnection,
                patch: #patch_struct_ident,
                context: Option<&#context_type>,
            ) -> #ret_type {
                #before_patch
                #after_patch
                Ok(())
            }
        }
    } else {
        quote! {}
    };

    let context = if entity.context_type.is_none() {
        if entity.before_patch.is_some() || entity.after_patch.is_some() {
            quote! { let context = None::<()>; }
        } else {
            quote! {}
        }
    } else {
        quote! { let context = None::<_>; }
    };

    let num_keys = primary_keys.len() + 1;
    let key_count = primary_keys.len();

    quote! {
        impl #patch_struct_ident {
            #vis async fn patch(
                &self,
                conn: &mut sqlx::PgConnection,
                #( #fn_items ),*,
            ) -> #ret_type {
                let mut columns = vec![];
                let selected_fields = 0 #( + usize::from(self.#patchable_fields.is_some()) )*;
                let mut value_index = #num_keys;
                let mut transform_param_offset = #key_count + selected_fields;

                #(#column_building)*

                let columns = columns.join(", ");
                let sql = format!(#patch_sql_statement, #table_name, columns);

                let mut query = sqlx::query::<sqlx::Postgres>(&sql)#(.bind(#primary_key_idents))*;
                #(#binding)*
                #(#transform_bindings)*

                query.execute(conn).await?;

                Ok(())
            }
        }

        impl #entity_ident {
            /// Updates the row as specified by the entity's keys with only the fields that are included in the patch.
            #vis async fn patch(
                &mut self,
                conn: &mut sqlx::PgConnection,
                patch: #patch_struct_ident,
            ) -> #ret_type {
                #context
                #before_patch
                #after_patch
                Ok(())
            }

            #context_variant

            #no_trigger_variant
        }
    }
}
