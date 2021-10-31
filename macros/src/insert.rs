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
    let insert_sql = insert_sql(entity);

    let before_insert = if let Some(before_fn) = &entity.before_insert {
        quote!(
            #before_fn(&self, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let after_insert = if let Some(after_fn) = &entity.after_insert {
        quote!(
            let rec = sqlx::query_as!(#entity_ident, #insert_sql, #(#query_idents),*)
                .fetch_one(&mut *conn)
                .await?;

            #after_fn(&rec, conn).await?;
        )
    } else {
        quote!(
            let rec = sqlx::query_as!(#entity_ident, #insert_sql, #(#query_idents),*)
                .fetch_one(conn)
                .await?;

        )
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<#entity_ident, #e_type>)
    } else {
        quote!(Result<#entity_ident, sqlx::Error>)
    };

    quote! {
        /// Insert a row into the database.
        #vis async fn insert(
            self,
            conn: &mut sqlx::PgConnection,
        ) -> #ret_type {
            #before_insert

            #after_insert

            Ok(rec)
        }

        /// Insert a row into the database.
        ///
        /// Does not call the before and after triggers.
        #vis async fn no_trigger_insert(
            self,
            conn: &mut sqlx::PgConnection,
        ) -> #ret_type {
            let rec = sqlx::query_as!(#entity_ident, #insert_sql, #(#query_idents),*)
                .fetch_one(conn)
                .await?;

            Ok(rec)
        }
    }
}

fn insert_sql(entity: &Entity) -> String {
    let columns = entity
        .fields
        .iter()
        .map(EntityField::fmt_for_select)
        .join(", ");

    let insertable = entity.insertable_fields().collect::<Vec<_>>();
    format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
        entity.table_name,
        insertable
            .iter()
            .map(|field| field.column_name.replace("r#", ""))
            .join(","),
        (1..=insertable.len()).map(|i| format!("${}", i)).join(","),
        columns
    )
}
