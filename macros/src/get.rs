use crate::transform::{build_select_clause, transform_bind_expressions, TransformBinding};
use crate::{attrs::ConvertType, Entity, EntityField};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

pub fn getters(entity: &Entity) -> TokenStream2 {
    let getters = entity
        .fields
        .iter()
        .flat_map(|field| {
            let get_one = field
                .get_one
                .as_ref()
                .map(|name| single(entity, field, name));
            let get_optional = field
                .get_optional
                .as_ref()
                .map(|name| optional(entity, field, name));
            let get_many = field
                .get_many
                .as_ref()
                .map(|name| many(entity, field, name));
            get_one.into_iter().chain(get_optional).chain(get_many)
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
        None => return quote!(),
    };
    let (sql, bindings) = match build_select_query(entity, None) {
        Ok(parts) => parts,
        Err(err) => return err.to_compile_error(),
    };
    let transform_binds = transform_bind_expressions(&bindings);
    let vis = &entity.vis;

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<Vec<Self>, #e_type>)
    } else {
        quote!(Result<Vec<Self>, sqlx::Error>)
    };

    let call = if entity.error_type.is_some() {
        quote! {
            Ok(sqlx::query_as!(Self, #sql #(, #transform_binds)*)
                .fetch_all(conn)
                .await?)
        }
    } else {
        quote! {
            sqlx::query_as!(Self, #sql #(, #transform_binds)*)
                .fetch_all(conn)
                .await
        }
    };

    quote! {
        #vis async fn #fn_name(
            conn: &mut sqlx::PgConnection
        ) -> #ret_type {
            #call
        }
    }
}

fn single(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let val = &field.ty;
    let vis = &entity.vis;
    let (query, bindings) = match build_select_query(entity, Some(field)) {
        Ok(parts) => parts,
        Err(err) => return err.to_compile_error(),
    };
    let transform_binds = transform_bind_expressions(&bindings);

    let by_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { *val as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&val) },
        None => quote! { val },
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<Self, #e_type>)
    } else {
        quote!(Result<Self, sqlx::Error>)
    };

    let call = if entity.error_type.is_some() {
        quote! {
            Ok(sqlx::query_as!(Self, #query, #by_converter #(, #transform_binds)*)
                .fetch_one(conn)
                .await?)
        }
    } else {
        quote! {
            sqlx::query_as!(Self, #query, #by_converter #(, #transform_binds)*)
                .fetch_one(conn)
                .await
        }
    };

    quote! {
        #vis async fn #fn_name(
            conn: &mut sqlx::PgConnection,
            val: &#val
        ) -> #ret_type {
            #call
        }
    }
}

fn optional(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let val = &field.ty;
    let vis = &entity.vis;
    let (query, bindings) = match build_select_query(entity, Some(field)) {
        Ok(parts) => parts,
        Err(err) => return err.to_compile_error(),
    };
    let transform_binds = transform_bind_expressions(&bindings);

    let by_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { *val as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&val) },
        None => quote! { val },
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<Option<Self>, #e_type>)
    } else {
        quote!(Result<Option<Self>, sqlx::Error>)
    };

    let call = if entity.error_type.is_some() {
        quote! {
            Ok(sqlx::query_as!(Self, #query, #by_converter #(, #transform_binds)*)
                .fetch_optional(conn)
                .await?)
        }
    } else {
        quote! {
            sqlx::query_as!(Self, #query, #by_converter #(, #transform_binds)*)
                .fetch_optional(conn)
                .await
        }
    };

    quote! {
        #vis async fn #fn_name(
            conn: &mut sqlx::PgConnection,
            val: &#val
        ) -> #ret_type {
            #call
        }
    }
}

fn many(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let val = &field.ty;
    let vis = &entity.vis;
    let (query, bindings) = match build_select_query(entity, Some(field)) {
        Ok(parts) => parts,
        Err(err) => return err.to_compile_error(),
    };
    let transform_binds = transform_bind_expressions(&bindings);

    let by_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { *val as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&val) },
        None => quote! { val },
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<Vec<Self>, #e_type>)
    } else {
        quote!(Result<Vec<Self>, sqlx::Error>)
    };

    let call = if entity.error_type.is_some() {
        quote! {
            Ok(sqlx::query_as!(Self, #query, #by_converter #(, #transform_binds)*)
                .fetch_all(conn)
                .await?)
        }
    } else {
        quote! {
            sqlx::query_as!(Self, #query, #by_converter #(, #transform_binds)*)
                .fetch_all(conn)
                .await
        }
    };

    quote! {
        #vis async fn #fn_name(
            conn: &mut sqlx::PgConnection,
            val: &#val
        ) -> #ret_type {
            #call
        }
    }
}

fn build_select_query(
    entity: &Entity,
    val: Option<&EntityField>,
) -> syn::Result<(String, Vec<TransformBinding>)> {
    let initial_offset = usize::from(val.is_some());
    let (columns, bindings, _) = build_select_clause(&entity.fields, initial_offset)?;

    if let Some(val) = val {
        Ok((
            format!(
                "SELECT {} FROM {} WHERE {} = $1",
                columns, entity.table_name, val.column_name
            ),
            bindings,
        ))
    } else {
        Ok((
            format!("SELECT {} FROM {}", columns, entity.table_name),
            bindings,
        ))
    }
}
