use crate::{Entity, EntityField};
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

pub fn getters(entity: &Entity) -> TokenStream2 {
    let getters = entity
        .fields
        .iter()
        .flat_map(|field| {
            let get_one = field.get_one.as_ref().map(|name| single(entity, field, name));
            let get_optional = field.get_optional.as_ref().map(|name| optional(entity, field, name));
            let get_many = field.get_many.as_ref().map(|name| many(entity, field, name));
            get_one.into_iter()
                .chain(get_optional.into_iter())
                .chain(get_many.into_iter())
        })
        .collect::<TokenStream2>();

    let get_all = get_all(entity);

    quote! {
        #get_all
        #getters
    }
}

fn get_all(entity: &Entity) -> TokenStream2 {
    let fn_name = match &entity.get_all {
        Some(ident) => ident,
        None => return quote!()
    };
    let sql = format!("SELECT * FROM {}", entity.table_name);
    let vis = &entity.vis;

    quote! {
        #vis async fn #fn_name(
            con: impl sqlx::Executor<'_, Database=sqlx::MySql>
        ) -> sqlx::Result<Vec<Self>> {
            sqlx::query_as(Self, #sql)
                .fetch_all(con)
                .await
        }
    }
}

fn single(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    println!("GET ONE!");
    let by = &field.ty;
    let vis = &entity.vis;
    let query = build_query(&entity.table_name, &entity.fields, field);

    quote! {
        #vis async fn #fn_name(
            con: impl sqlx::Executor<'_, Database=sqlx::MySql>,
            by: &#by
        ) -> sqlx::Result<Self> {
            sqlx::query_as!(Self, #query, by)
                .fetch_one(con)
                .await
        }
    }
}

fn optional(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let by = &field.ty;
    let vis = &entity.vis;
    let query = build_query(&entity.table_name, &entity.fields, field);

    quote! {
        #vis async fn #fn_name(
            con: impl sqlx::Executor<'_, Database=sqlx::MySql>,
            by: &#by
        ) -> sqlx::Result<Option<Self>> {
            sqlx::query_as!(Self, #query, by)
                .fetch_optional(con)
                .await
        }
    }
}

fn many(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let by = &field.ty;
    let vis = &entity.vis;
    let query = build_query(&entity.table_name, &entity.fields, field);

    quote! {
        #vis async fn #fn_name(
            con: impl sqlx::Executor<'_, Database=sqlx::MySql>,
            by: #by
        ) -> sqlx::Result<Vec<Self>> {
            sqlx::query_as!(Self, #query, by)
                .fetch_all(con)
                .await
        }
    }
}

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
