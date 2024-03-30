use crate::{attrs::ConvertType, Entity, EntityField};
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn setters(entity: &Entity) -> TokenStream2 {
    entity
        .fields
        .iter()
        .flat_map(|field| field.set.as_ref().map(|name| setter(entity, field, name)))
        .collect()
}

fn setter(entity: &Entity, field: &EntityField, fn_name: &Ident) -> TokenStream2 {
    let mut query = format!(
        "UPDATE {} SET {} = $1 WHERE",
        entity.table_name, field.column_name
    );

    let primary_keys: Vec<&EntityField> = entity.fields.iter().filter(|x| x.is_key).collect();
    for (index, key) in primary_keys.iter().enumerate() {
        query.push_str(format!(" {} = ${}", key.column_name, index + 2).as_str());
        if index + 1 != primary_keys.len() {
            query.push_str(" AND")
        }
    }

    let field_ty = &field.ty;
    let field_ident = &field.ident;
    let vis = &entity.vis;

    let value_converter = match &field.convert {
        Some(ConvertType::As(t)) => quote! { value as #t },
        Some(ConvertType::Function(convert_fn)) => quote! { #convert_fn(&value) },
        None => quote! { value },
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<(), #e_type>)
    } else {
        quote!(Result<(), sqlx::Error>)
    };

    let ident_keys = primary_keys.iter().map(|x| &x.ident);

    quote! {
        #vis async fn #fn_name(
            &mut self,
            conn: &mut sqlx::PgConnection,
            value: #field_ty
        ) -> #ret_type {
            sqlx::query!(#query, #value_converter, #(&self.#ident_keys),*)
                .execute(conn)
                .await?;
            self.#field_ident = value;
            Ok(())
        }
    }
}
