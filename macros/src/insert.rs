use crate::transform::{
    bind_transform_params, build_select_clause, process_transform, TransformBinding,
};
use crate::{attrs::ConvertType, Entity, EntityField};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

pub fn insert(entity: &Entity) -> TokenStream {
    let struct_ident = match &entity.insert {
        Some(ident) => ident,
        None => return quote!(),
    };

    let vis = &entity.vis;
    let fields = entity
        .insertable_fields()
        .map(|EntityField { ident, ty, .. }| quote!(#vis #ident: #ty));
    let insert_fn = insert_fn(entity);

    quote! {
        #[derive(Debug)]
        #vis struct #struct_ident {
            #(#fields),*
        }

        impl #struct_ident {
            #insert_fn
        }
    }
}

fn insert_fn(entity: &Entity) -> TokenStream {
    let query_idents = entity
        .insertable_fields()
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

    let vis = &entity.vis;
    let entity_ident = &entity.ident;
    let (insert_sql, set_bindings, get_bindings) = match insert_sql(entity) {
        Ok(parts) => parts,
        Err(err) => return err.to_compile_error(),
    };
    let bindings = set_bindings
        .into_iter()
        .chain(get_bindings)
        .collect::<Vec<_>>();
    let transform_params = bind_transform_params(&bindings);
    let param_binds = &transform_params.expressions;
    let insert_query = transform_params.wrap(quote! {
        sqlx::query_as!(
            #entity_ident,
            #insert_sql,
            #(#query_idents),*
            #(, #param_binds)*
        )
        .fetch_one(&mut *conn)
        .await
    });

    let has_trigger = entity.before_insert.is_some() || entity.after_insert.is_some();

    let before_insert = if let Some(before_fn) = &entity.before_insert {
        quote!(
            #before_fn(&self, context, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let after_insert = if let Some(after_fn) = &entity.after_insert {
        quote!(
            let rec = #insert_query?;
            #after_fn(&rec, context, conn).await?;
        )
    } else {
        quote!(
            let rec = #insert_query?;
        )
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<#entity_ident, #e_type>)
    } else {
        quote!(Result<#entity_ident, sqlx::Error>)
    };

    let no_trigger_variant = if has_trigger {
        quote! {
            /// Insert a row into the database.
            ///
            /// Does not call the before and after triggers.
            #vis async fn no_trigger_insert(
                self,
                conn: &mut sqlx::PgConnection,
            ) -> #ret_type {
                let rec = #insert_query?;
                Ok(rec)
            }
        }
    } else {
        quote!()
    };

    let context_variant = if let Some(context_type) = &entity.context_type {
        quote! {
            #vis async fn insert_with_context(
                self,
                conn: &mut sqlx::PgConnection,
                context: Option<&#context_type>,
            ) -> #ret_type {
                #before_insert
                #after_insert
                Ok(rec)
            }
        }
    } else {
        quote! {}
    };

    let context = if entity.context_type.is_none() {
        if entity.before_insert.is_some() || entity.after_insert.is_some() {
            quote! { let context = None::<()>; }
        } else {
            quote! {}
        }
    } else {
        quote! { let context = None::<_>; }
    };

    quote! {
        /// Insert a row into the database.
        #vis async fn insert(
            self,
            conn: &mut sqlx::PgConnection,
        ) -> #ret_type {
            #context
            #before_insert
            #after_insert
            Ok(rec)
        }

        #context_variant

        #no_trigger_variant
    }
}

fn insert_sql(
    entity: &Entity,
) -> syn::Result<(String, Vec<TransformBinding>, Vec<TransformBinding>)> {
    let insertable = entity.insertable_fields().collect::<Vec<_>>();
    let mut param_offset = insertable.len();
    let mut set_bindings = Vec::new();
    let mut values = Vec::with_capacity(insertable.len());

    for (index, field) in insertable.iter().enumerate() {
        let value_placeholder = format!("${}", index + 1);
        if let Some(transform_set) = &field.transform_set {
            let (expr, count) = process_transform(transform_set, &value_placeholder, param_offset)?;
            param_offset += count;
            if let Some(params_fn) = &field.transform_set_params {
                set_bindings.push(TransformBinding {
                    params_fn: params_fn.clone(),
                    count,
                });
            }
            values.push(expr);
        } else {
            values.push(value_placeholder);
        }
    }

    let (columns, get_bindings, _) = build_select_clause(&entity.fields, param_offset)?;
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
        entity.table_name,
        insertable
            .iter()
            .map(|field| field.column_name.replace("r#", ""))
            .join(", "),
        values.join(", "),
        columns
    );

    Ok((sql, set_bindings, get_bindings))
}
