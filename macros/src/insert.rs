use crate::{connection_type, Entity, EntityField};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;
use syn::Result;

pub fn insert_struct(entity: &Entity) -> TokenStream {
    if entity.generated_fields().next().is_none() {
        return quote!();
    }

    let fields = entity
        .data_fields()
        .map(|EntityField { ident: rust_ident, ty, .. }| quote!(pub #rust_ident : #ty))
        .collect::<Vec<_>>();
    let struct_ident = Ident::new(&format!("Insert{}", entity.ident), Span::call_site());
    let doc = format!("Helper to insert [{}]({}) into the database.", entity.ident, entity.ident);
    let vis = &entity.visibility;

    quote! {
        #[derive(Debug)]
        #[cfg_attr(feature = "serde-support", derive(serde::Serialize, serde::Deserialize))]
        #[doc = #doc]
        #vis struct #struct_ident {
            #(#fields),*
        }
    }
}

pub fn insert_fn(entity: &Entity) -> Result<TokenStream> {
    let con = connection_type();
    let data_fields: Vec<&EntityField> = entity.data_fields().collect();
    let data_field_idents = entity.data_fields().map(|field| &field.ident);

    let query = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        entity.table_name,
        data_fields
            .iter()
            .map(|field| field.column_name.clone())
            .collect::<Vec<_>>()
            .join(","),
        std::iter::repeat("?")
            .take(entity.data_fields().count())
            .collect::<Vec<_>>()
            .join(",")
    );

    Ok(quote! {
        /// Insert a row into the database.
        pub async fn insert(
            self,
            con: &mut #con
        ) -> sqlx::Result<()> {
            sqlx::query!(
                #query,
                #(self.#data_field_idents),*
            )
            .execute(con)
            .await?;

            Ok(())
        }
    })
}

