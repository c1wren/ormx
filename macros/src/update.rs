use crate::{connection_type, Entity};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

fn update_struct_ident(entity: &Entity) -> Ident {
    Ident::new(&format!("Update{}", entity.ident), Span::call_site())
}

pub fn update_struct(entity: &Entity) -> TokenStream {
    let ident = update_struct_ident(entity);
    let vis = &entity.visibility;
    let fields = entity.updatable_fields();
    let doc = format!(
        "Helper to update [{}]({}) in the database",
        entity.ident, entity.ident
    );
    quote! {
        #[derive(Debug)]
        #[cfg_attr(feature = "serde-support", derive(serde::Serialize, serde::Deserialize))]
        #[doc = #doc]
        #vis struct #ident {
            #(#fields),*
        }
    }
}

pub fn update_fn(entity: &Entity) -> TokenStream {
    let pkey = match &entity.primary_key {
        Some(pkey) => pkey,
        None => return quote!(),
    };
    let query = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        entity.table_name,
        entity
            .updatable_fields()
            .map(|field| format!("{}=?", field.column_name))
            .collect::<Vec<_>>()
            .join(","),
        pkey.column_name
    );
    let con = connection_type();
    let update_fields = entity
        .updatable_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let pkey_ident = &pkey.ident;
    let update_ident = update_struct_ident(entity);

    quote! {
        pub async fn update(
            &mut self,
            con: &mut #con,
            update: #update_ident,
        ) -> sqlx::Result<()> {
            sqlx::query!(
                #query,
                #(self.#update_fields,)*
                self.#pkey_ident,
            )
            .execute(con)
            .await?;

            #(self.#update_fields = update.#update_fields;)*

            Ok(())
        }
    }
}
