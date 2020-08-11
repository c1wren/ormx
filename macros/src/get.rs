use crate::{connection_type, Entity, EntityField};
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

fn build_query(table_name: &str, fields: &[EntityField], by: &EntityField) -> String {
    let columns = fields
        .iter()
        .map(|field| {
            let rust_ident = field.ident.to_string();
            if field.column_name == rust_ident {
                rust_ident
            } else {
                format!("{} AS {}", field.column_name, rust_ident)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "SELECT {} FROM {} WHERE {} = ?",
        columns, table_name, by.column_name
    )
}

// TODO
// pub(crate) fn many(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
//     let con = connection_type();
//     let by = &field.ty;
//     let query = build_query(&entity.table_name, &entity.fields, field);
//
//     quote! {
//         pub fn #fn_name<'e>(
//             con: &'e mut #con,
//             by: #by
//         ) -> futures::stream::BoxStream<'e, sqlx::Result<Self>> {
//             sqlx::query_as!(Self, #query, by)
//                 .fetch(con)
//         }
//     }
// }

pub(crate) fn single(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let con = connection_type();
    let by = &field.ty;
    let query = build_query(&entity.table_name, &entity.fields, field);

    quote! {
        pub async fn #fn_name<'e>(
            con: &'e mut #con,
            by: &'e #by
        ) -> sqlx::Result<Self> {
            sqlx::query_as!(Self, #query, by)
                .fetch_one(con)
                .await
        }
    }
}

pub(crate) fn optional(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let con = connection_type();
    let by = &field.ty;
    let query = build_query(&entity.table_name, &entity.fields, field);

    quote! {
        pub async fn #fn_name<'e>(
            con: &'e mut #con,
            by: &'e #by
        ) -> sqlx::Result<Option<Self>> {
            sqlx::query_as!(Self, #query, by)
                .fetch_optional(con)
                .await
        }
    }
}
