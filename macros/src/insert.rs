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

    let generated_idents = entity
        .generated_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let query_generated = if generated_idents.is_empty() {
        quote!()
    } else {
        let sql = query_generated_sql(entity);
        quote!(let generated = sqlx::query!(#sql, rec.id).fetch_one(&mut tx).await?;)
    };

    let vis = &entity.vis;

    let entity_ident = &entity.ident;
    let insert_sql = insert_sql(entity);
    let id_ident = &entity.id.ident;

    quote! {
        /// Insert a row into the database.
        #vis async fn insert(
            self,
            con: impl sqlx::Executor<'_, Database=sqlx::Postgres>,
        ) -> sqlx::Result<#entity_ident> {
            let rec = sqlx::query_as!(#entity_ident, #insert_sql, #(#query_idents),*)
                .fetch_one(con)
                .await?;

            #query_generated

            Ok(#entity_ident {
                #id_ident: rec.id as _,
                #(#insertable_idents: self.#insertable_idents,)*
                #(#generated_idents: generated.#generated_idents),*
            })
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
        insertable.iter().map(|field| &field.column_name).join(","),
        (1..=insertable.len()).map(|i| format!("${}", i)).join(","),
        columns
    )
}

fn query_generated_sql(entity: &Entity) -> String {
    format!(
        "SELECT {} FROM {} WHERE {} = $1",
        entity
            .generated_fields()
            .map(EntityField::fmt_for_select)
            .join(","),
        entity.table_name,
        entity.id.column_name
    )
}
