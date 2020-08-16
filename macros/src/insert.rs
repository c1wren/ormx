use crate::{Entity, EntityField};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use std::iter::repeat;

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
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #vis struct #struct_ident {
            #(#fields),*
        }

        impl #struct_ident {
            #insert_fn
        }
    }
}

fn insert_fn(entity: &Entity) -> TokenStream {
    let insertable_idents = entity
        .insertable_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();

    let generated_idents = entity
        .generated_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let query_generated = if generated_idents.is_empty() {
        quote!()
    } else {
        let sql = query_generated_sql(entity);
        quote!(let generated = sqlx::query!(#sql, last_insert_id).fetch_one(&mut tx).await?;)
    };

    let vis = &entity.vis;

    let entity_ident = &entity.ident;
    let insert_sql = insert_sql(entity);
    let id_ident = &entity.id.ident;

    quote! {
        /// Insert a row into the database.
        #vis async fn insert(
            self,
            __con: &mut sqlx::MySqlConnection,
        ) -> sqlx::Result<#entity_ident> {
            use sqlx::Connection;
            let mut tx = __con.begin().await?;

            let last_insert_id = sqlx::query!(#insert_sql, #(self.#insertable_idents),*)
                .execute(&mut tx)
                .await?
                .last_insert_id();

            #query_generated

            tx.commit().await?;

            Ok(#entity_ident {
                #id_ident: last_insert_id as _,
                #(#insertable_idents: self.#insertable_idents,)*
                #(#generated_idents: generated.#generated_idents),*
            })
        }
    }
}

fn insert_sql(entity: &Entity) -> String {
    let insertable = entity.insertable_fields().collect::<Vec<_>>();
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        entity.table_name,
        insertable.iter().map(|field| &field.column_name).join(","),
        repeat("?").take(insertable.len()).join(",")
    )
}

fn query_generated_sql(entity: &Entity) -> String {
    format!(
        "SELECT {} FROM {} WHERE {} = ?",
        entity
            .generated_fields()
            .map(EntityField::fmt_for_select)
            .join(","),
        entity.table_name,
        entity.id.column_name
    )
}
