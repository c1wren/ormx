use crate::{connection_type, Accessor, Entity, EntityField, function_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, Type};

pub(crate) fn generate(entity: &Entity, field: &EntityField, accessor: &Accessor) -> TokenStream2 {
    let fn_name = function_name("get", )
    let fn_name = &accessor
        .clone()
        .unwrap_or_else(|| Ident::new(&format!("by_{}", field.rust_ident), Span::call_site()));


    let ty = &field.ty;
    match &get.quantity {
        GetQuantity::Optional => fetch_optional(fn_name, ty, &query),
        GetQuantity::Single => fetch_single(fn_name, ty, &query),
        GetQuantity::Multiple => fetch_multiple(fn_name, ty, &query),
    }
}

fn build_query(table_name: &str, fields: &[EntityField], by: &EntityField) -> String {
    let columns = fields
        .iter()
        .map(|field| format!("{} AS {}", field.db_ident, field.rust_ident))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "SELECT {} FROM {} WHERE {} = ?",
        columns, table_name, by.db_ident
    )
}

fn fetch_multiple(fn_name: &Ident, by: &Type, query: &str) -> TokenStream2 {
    let con = connection_type();
    quote! {
        pub async fn #fn_name<'e>(
            con: &'e mut #con,
            by: &'e #by
        ) -> sqlx::Result<Self> {
            sqlx::query_as!(Self, #query, by)
                .fetch(con)
                .await
        }
    }
}

fn fetch_single(fn_name: &Ident, by: &Type, query: &str) -> TokenStream2 {
    let con = connection_type();
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

fn fetch_optional(fn_name: &Ident, by: &Type, query: &str) -> TokenStream2 {
    let con = connection_type();
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
