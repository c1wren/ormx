use crate::{connection_type, Entity, EntityField};
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;
use syn::Result;

pub fn insert_struct(entity: &Entity) -> TokenStream {
    let data_fields = entity
        .data_fields()
        .collect::<Vec<_>>();

    if data_fields.is_empty() {
        return quote!();
    }

    let struct_ident = insert_struct_ident(entity);
    let doc = format!(
        "Helper to insert [{}]({}) into the database.",
        entity.ident, entity.ident
    );
    let vis = &entity.visibility;

    quote! {
        #[derive(Debug)]
        #[cfg_attr(feature = "serde-support", derive(serde::Serialize, serde::Deserialize))]
        #[doc = #doc]
        #vis struct #struct_ident {
            #(#data_fields),*
        }
    }
}

pub fn insert_fn(entity: &Entity) -> Result<TokenStream> {
    let con = connection_type();
    let data_fields: Vec<&EntityField> = entity.data_fields().collect();
    let data_field_idents = data_fields.iter().map(|field| &field.ident);

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        entity.table_name,
        data_fields.iter().map(|field| &field.column_name).join(","),
        std::iter::repeat("?").take(data_fields.len()).join(",")
    );

    let insert_struct_param = match data_fields.len() {
        0 => quote!(),
        _ => {
            let ident = insert_struct_ident(entity);
            quote!(insert: &#ident)
        }
    };

    Ok(quote! {
        /// Insert a row into the database.
        pub async fn insert(
            con: &mut #con,
            #insert_struct_param,
        ) -> ormx::sqlx::Result<()> {
            use ormx::sqlx;

            sqlx::query!(
                #query,
                #(insert.#data_field_idents),*
            )
            .execute(con)
            .await?;

            Ok(())
        }
    })
}

fn insert_struct_ident(entity: &Entity) -> Ident {
    Ident::new(&format!("Insert{}", entity.ident), Span::call_site())
}