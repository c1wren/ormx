use crate::{Entity, EntityField};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn delete(entity: &Entity) -> TokenStream {
    let delete = entity
        .delete
        .as_ref()
        .map(|delete_fn| delete_self(entity, delete_fn));

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

fn delete_self(entity: &Entity, fn_name: &Ident) -> TokenStream {
    let vis = &entity.vis;
    let id_ident = &entity.id.ident;
    let sql = format!(
        "DELETE FROM {} WHERE {} = $1",
        entity.table_name, entity.id.column_name
    );

    let before_delete = if let Some(before_fn) = &entity.before_delete {
        quote!(
            #before_fn(&self, &mut *conn).await?;
        )
    } else {
        quote!()
    };

    let after_delete = if let Some(after_fn) = &entity.after_delete {
        quote!(
            sqlx::query!(#sql, self.#id_ident).execute(&mut *conn).await?;

            #after_fn(self, conn).await?;
        )
    } else {
        quote!(
            sqlx::query!(#sql, self.#id_ident).execute(conn).await?;
        )
    };

    let ret_type = if let Some(e_type) = &entity.error_type {
        quote!(Result<(), #e_type>)
    } else {
        quote!(Result<(), sqlx::Error>)
    };

    quote! {
        /// Deletes a row from the database.
        #vis async fn #fn_name(
            self,
            conn: &mut sqlx::PgConnection,
        ) -> #ret_type {
            #before_delete

            #after_delete

            Ok(())
        }
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
