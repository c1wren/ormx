use crate::{connection_type, Entity, EntityField};
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::Ident;
use syn::Result;

/*
mod sqlite {
    use crate::{Entity, EntityField};
    use quote::quote;
    use proc_macro2::{Span, TokenStream as TokenStream2};

    fn insert(entity: &Entity) -> TokenStream2 {
        let columns = entity
            .fields
            .iter()
            .filter(|field| !field.generated)
            .map(|field| &field.db_ident);

        let column_names = columns.cloned().collect::<Vec<_>>().join(",");
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
 */

fn insert_type(entity: &Entity) -> (Ident, TokenStream2) {
    let generated_fields = entity
        .fields
        .iter()
        .filter(|field| field.generated)
        .collect::<Vec<&EntityField>>();

    if generated_fields.is_empty() {
        return (entity.ident.clone(), quote!());
    }

    let fields = generated_fields
        .into_iter()
        .map(|EntityField { rust_ident, ty, .. }| quote!(pub #rust_ident : #ty))
        .collect();
    let struct_ident = Ident::new(&format!("Insert{}", entity.ident), Span::call_site());

    let insert_struct = quote! {
        pub struct #struct_ident {
            #(#fields)*
        }
    };

    (struct_ident, insert_struct)
}

fn insert_default(entity: &Entity) -> TokenStream2 {
    unimplemented!()
}

fn insert_with_generated(entity: &Entity) -> TokenStream2 {

}

fn insert(entity: &Entity) -> Result<TokenStream2> {
    let data_fields = entity
        .fields
        .iter()
        .filter(|field| !field.generated)
        .collect::<Vec<&EntityField>>();
    let data_idents = data_fields.iter().map(|field| field.rust_ident);
    let data_columns = data_fields
        .iter()
        .map(|field| field.db_ident.clone())
        .collect::<Vec<String>>()
        .join(",");

    let con = connection_type();
    let (insert_struct_ident, insert_struct) = insert_type(entity);
    let entity_ty = &entity.ident;
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        entity.table_name,
        data_columns,
        std::iter::repeat("?")
            .take(data_fields.len())
            .collect::<Vec<_>>()
            .join(",")
    );

    Ok(quote::quote! {
        impl #insert_struct_ident {
            pub async fn insert(
                self,
                con: &mut #con
            ) -> sqlx::Result<#entity_ty> {
                let mut tx = con.begin().await?;
                sqlx::query!(#query, #(#insert_idents),*).execute(con).await?;
                let result = sqlx::query_as!(#entity_ty, "SELECT * FROM")

            }
        }
    })
}

fn join_n(n: usize, )

fn generate() -> TokenStream2 {
    quote::quote! {}
}
