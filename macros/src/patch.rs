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
    let id_ty = &entity.id.ty;
    let id_ident = &entity.id.ident;
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

    let before_value_sql = format!(
        "SELECT {} FROM {} WHERE {} = $1",
        columns, entity.table_name, entity.id.column_name
    );

    let has_trigger = entity.before_patch.is_some() || entity.after_patch.is_some();

    let before_patch = if let Some(before_fn) = &entity.before_patch {
        quote!(
            #before_fn(self, &patch, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let after_patch = if let Some(after_fn) = &entity.after_patch {
        quote!(
            let previous = sqlx::query_as!(Self, #before_value_sql, self.id)
                .fetch_one(&mut *conn)
                .await?;

            #patch_struct_ident::patch(&patch, &mut *conn, &self.#id_ident).await?;

            #(if let Some(new_value) = patch.#patchable_fields {
                self.#patchable_fields = new_value;
            })*

            #after_fn(self, previous, conn).await?;
        )
    } else {
        quote!(
            #patch_struct_ident::patch(&patch, conn, &self.#id_ident).await?;

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
            impl #entity_ident {
                #vis async fn no_trigger_patch(
                    &mut self,
                    conn: &mut sqlx::PgConnection,
                    patch: #patch_struct_ident,
                ) -> #ret_type {
                    #patch_struct_ident::patch(&patch, conn, &self.#id_ident).await?;

                    #(if let Some(new_value) = patch.#patchable_fields {
                        self.#patchable_fields = new_value;
                    })*

                    Ok(())
                }
            }
        }
    } else {
        quote!()
    };

    quote! {
        impl #patch_struct_ident {
            #vis async fn patch(
                &self,
                conn: &mut sqlx::PgConnection,
                id: &#id_ty,
            ) -> #ret_type {
                let mut columns = vec![];
                let mut count = 2;

                #(#column_building)*

                let columns = columns.join(", ");

                let sql = format!("UPDATE {} SET {} WHERE id = $1", #table_name, columns);

                let mut query = sqlx::query(&sql).bind(id);
                #(#binding)*

                query.execute(conn).await?;

                Ok(())
            }
        }

        /// Updates the row as specified by the entity's primary key with only the fields that are included in the patch.
        impl #entity_ident {
            #vis async fn patch(
                &mut self,
                conn: &mut sqlx::PgConnection,
                patch: #patch_struct_ident,
            ) -> #ret_type {
                #before_patch

                #after_patch

                Ok(())
            }
        }

        #no_trigger_variant
    }
}
