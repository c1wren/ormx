use crate::{attrs::ConvertType, Entity, EntityField};
use itertools::Itertools;
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
        .map(|EntityField { ident, ty, .. }| quote!(#vis #ident: Option<#ty>));

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
    let entity_ident = &entity.ident;
    let table_name = &entity.table_name;
    let vis = &entity.vis;

    let column_building = entity.patchable_fields().map(|field| {
        let ident = &field.ident;
        quote!(
            if self.#ident.is_some() {
                columns.push(format!("{} = ${}", stringify!(#ident), count));
                count += 1;
            }
        )
    });

    let binding = entity.patchable_fields().map(|field| {
        let ident = &field.ident;
        let value_getter = match &field.convert {
            Some(ConvertType::As(t)) => quote! { *value as #t },
            Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&value) },
            None => quote! { value },
        };

        quote!(
            if let Some(value) = self.#ident.as_ref() {
                query = query.bind(#value_getter)
            }
        )
    });

    let columns = entity
        .fields
        .iter()
        .map(EntityField::fmt_for_select)
        .join(", ");

    let primary_keys: Vec<&EntityField> = entity.fields.iter().filter(|x| x.is_key).collect();
    let num_keys = primary_keys.len() + 1;

    let mut patch_sql_statement = String::from("UPDATE {} SET {} WHERE");
    let mut before_patch_sql = format!("SELECT {} FROM {} WHERE", columns, entity.table_name);
    for (index, key) in primary_keys.iter().enumerate() {
        let condition = format!(" {} = ${}", key.column_name, index + 1);
        before_patch_sql.push_str(condition.as_str());
        patch_sql_statement.push_str(condition.as_str());
        if index + 1 != primary_keys.len() {
            before_patch_sql.push_str(" AND");
            patch_sql_statement.push_str(" AND");
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
        let stream = quote! {
            &self.#ident
        };

        items.push(stream);
    }

    let mut fn_items = Vec::new();
    for (ident, ty) in keys {
        let stream = quote! {
            #ident: &#ty
        };

        fn_items.push(stream);
    }

    let ident_keys: Vec<&Ident> = primary_keys.iter().map(|x| &x.ident).collect();

    let after_patch = if let Some(after_fn) = &entity.after_patch {
        quote!(
            let previous = sqlx::query_as!(Self, #before_patch_sql, #(self.#ident_keys),*)
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

    let ty = if entity.context_type.is_none() {
        quote! { () }
    } else {
        quote! { _ }
    };

    quote! {
        impl #patch_struct_ident {
            #vis async fn patch(
                &self,
                conn: &mut sqlx::PgConnection,
                #( #fn_items ),*,
            ) -> #ret_type {
                let mut columns = vec![];
                let mut count = #num_keys;

                #(#column_building)*

                let columns = columns.join(", ");

                let sql = format!(#patch_sql_statement, #table_name, columns);

                let mut query = sqlx::query(&sql)#(.bind(#ident_keys))*;
                #(#binding)*

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
                let context = None::<#ty>;

                #before_patch

                #after_patch

                Ok(())
            }

            #context_variant

            #no_trigger_variant
        }
    }
}
