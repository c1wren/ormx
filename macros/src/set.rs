use crate::{attrs::ConvertType, Entity, EntityField};
use proc_macro2::Ident;
use proc_macro2::Span;
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

    let fn_name_with_context = Ident::new(&format!("{}_with_context", fn_name), Span::call_site());
    let fn_name_no_trigger = Ident::new(&format!("no_trigger_{}", fn_name), Span::call_site());

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

    let ident_keys: Vec<&Ident> = primary_keys.iter().map(|x| &x.ident).collect();

    let before_update = if let Some(before_fn) = &entity.before_update {
        quote!(
            #before_fn(self, context, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let has_trigger = entity.after_update.is_some() || entity.before_update.is_some();

    let after_update = if let Some(after_fn) = &entity.after_update {
        quote!(
            let previous = self.clone();

            sqlx::query!(#query, #value_converter, #(&self.#ident_keys),*)
                .execute(&mut *conn)
                .await?;
            self.#field_ident = value;

            #after_fn(self, previous, context, conn).await?;
        )
    } else {
        quote!(
            sqlx::query!(#query, #value_converter, #(&self.#ident_keys),*)
                .execute(conn)
                .await?;
            self.#field_ident = value;
        )
    };

    let no_trigger_variant = if has_trigger {
        quote! {
            /// Updates the row in the database specified by the primary key.
            ///
            /// This will update every field except the primary key field. `Patch` should be used if only updating some of the fields.
            ///
            /// Does not call the before and after triggers.
            #vis async fn #fn_name_no_trigger(
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
    } else {
        quote!()
    };

    quote! {
        #vis async fn #fn_name(
            &mut self,
            conn: &mut sqlx::PgConnection,
            value: #field_ty
        ) -> #ret_type {
            let context: Option<&()> = None;

            #before_update

            #after_update

            Ok(())
        }

        #vis async fn #fn_name_with_context<T: Serialize>(
            &mut self,
            conn: &mut sqlx::PgConnection,
            value: #field_ty,
            context: Option<&T>,
        ) -> #ret_type {
            #before_update

            #after_update

            Ok(())
        }

        #no_trigger_variant
    }
}
