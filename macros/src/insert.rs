use proc_macro2::{Span, TokenStream as TokenStream2};

mod sqlite {
    use crate::{Entity, EntityField};
    use proc_macro2::{Span, TokenStream as TokenStream2};

    fn insert(entity: &Entity) -> TokenStream2 {
        let columns = entity
            .fields
            .iter()
            .filter(|field| !field.generated)
            .map(|field| &field.db_ident);

        let column_names = columns.collect::<Vec<_>>().join(",");
        let values = columns
            .map(|column| format!("{} = ?", column))
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            entity.table_name, column_names, values
        );

        let args = entity.fields.iter().map(|field| &field.rust_ident);

        quote! {
            sqlx::query!(#query, #(#args),*)
                .execute(&mut #con)
                .await?;

            let (id): (i64,) = sqlx::query!("SELECT LAST_INSERT_ROWID()")
                .fetch_one()
                .await?;
            id
        }
    }

    fn query(entity: &Entity, generated: &[EntityField]) -> TokenStream2 {
        quote! {
            sqlx::query!(#)
        }
    }
}

fn generate() -> TokenStream2 {
    quote! {
        pub async fn insert()
        sqlx::query!(#query)
    }
}
