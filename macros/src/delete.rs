use crate::{Entity, EntityField};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn delete(entity: &Entity) -> TokenStream {
    let delete = entity.delete.as_ref().map(|_| delete_self(entity));

    let delete_by = entity
        .fields
        .iter()
        .flat_map(|field| {
            field
                .delete
                .as_ref()
                .map(|delete_fn| delete_by(entity, field, delete_fn))
        })
        .collect::<TokenStream>();

    quote! {
        #delete
        #delete_by
    }
}

fn delete_self(entity: &Entity) -> TokenStream {
    let vis = &entity.vis;
    let mut sql = format!("DELETE FROM {} WHERE", entity.table_name);
    let keys: Vec<&EntityField> = entity.fields.iter().filter(|x| x.is_key).collect();
    for (index, key) in keys.iter().enumerate() {
        sql.push_str(format!(" {} = ${}", key.column_name, index + 1).as_str());
        if index + 1 != keys.len() {
            sql.push_str(" AND ")
        }
    }

    let has_trigger = entity.before_delete.is_some() || entity.after_delete.is_some();

    let before_delete = if let Some(before_fn) = &entity.before_delete {
        quote!(
            #before_fn(&self, context, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let ident_keys: Vec<_> = keys
        .iter()
        .map(|k| {
            let ident = &k.ident;
            if k.custom_type {
                quote!(self.#ident as _)
            } else {
                quote!(self.#ident)
            }
        })
        .collect();

    let after_delete = if let Some(after_fn) = &entity.after_delete {
        quote!(
            sqlx::query!(#sql, #(#ident_keys),*).execute(&mut *conn).await?;

            #after_fn(self, context, conn).await?;
        )
    } else {
        quote!(
            sqlx::query!(#sql, #(#ident_keys),*).execute(conn).await?;
        )
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<(), #e_type>)
    } else {
        quote!(Result<(), sqlx::Error>)
    };

    let no_trigger_variant = if has_trigger {
        quote! {
            #vis async fn no_trigger_delete(
                self,
                conn: &mut sqlx::PgConnection,
            ) -> #ret_type {
                sqlx::query!(#sql, #(#ident_keys),*).execute(conn).await?;

                Ok(())
            }
        }
    } else {
        quote!()
    };

    let context_variant = if let Some(context_type) = &entity.context_type {
        quote! {
            #vis async fn delete_with_context(
                self,
                conn: &mut sqlx::PgConnection,
                context: Option<&#context_type>,
            ) -> #ret_type {
                #before_delete

                #after_delete

                Ok(())
            }
        }
    } else {
        quote! {}
    };

    let context = if entity.context_type.is_none() {
        if entity.before_delete.is_some() || entity.after_delete.is_some() {
            quote! { let context = None::<()>; }
        } else {
            quote! {}
        }
    } else {
        quote! { let context = None::<_>; }
    };

    quote! {
        /// Deletes the entity with the specified primary key from the database.
        #vis async fn delete(
            self,
            conn: &mut sqlx::PgConnection,
        ) -> #ret_type {
            #context

            #before_delete

            #after_delete

            Ok(())
        }

        #context_variant

        #no_trigger_variant
    }
}

fn delete_by(entity: &Entity, val: &EntityField, fn_name: &Ident) -> TokenStream {
    let vis = &entity.vis;
    let val_ty = &val.ty;
    let sql = format!(
        "DELETE FROM {} WHERE {} = $1",
        entity.table_name, val.column_name
    );

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<u64, #e_type>)
    } else {
        quote!(Result<u64, sqlx::Error>)
    };

    quote! {
        #vis async fn #fn_name(
            conn: &mut sqlx::PgConnection,
            val: &#val_ty,
        ) -> #ret_type {
            use sqlx::Done;

            let result = sqlx::query!(#sql, by).execute(conn).await?;

            Ok(result.rows_affected())
        }
    }
}
