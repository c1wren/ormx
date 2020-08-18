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

    quote! {
        #vis async fn #fn_name(
            self,
            con: impl sqlx::Executor<'_, Database=sqlx::Postgres>,
        ) -> sqlx::Result<()> {
            sqlx::query!(#sql, self.#id_ident).execute(con).await?;
            Ok(())
        }
    }
}

fn delete_by(entity: &Entity, by: &EntityField, fn_name: &Ident) -> TokenStream {
    let vis = &entity.vis;
    let by_ty = &by.ty;
    let sql = format!(
        "DELETE FROM {} WHERE {} = $1",
        entity.table_name, by.column_name
    );

    quote! {
        #vis async fn #fn_name(
            con: impl sqlx::Executor<'_, Database=sqlx::Postgres>,
            by: &#by_ty,
        ) -> sqlx::Result<u64> {
            use sqlx::Done;
            let result = sqlx::query!(#sql, by).execute(con).await?;
            Ok(result.rows_affected())
        }
    }
}
