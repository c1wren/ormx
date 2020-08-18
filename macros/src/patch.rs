use crate::{Entity, EntityField};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn patch(entity: &Entity) -> TokenStream {
    let patch_struct_ident = match &entity.patch {
        Some(ident) => ident,
        None => return quote!(),
    };

    if entity.patchable_fields().count() == 0 {
        panic!("#[ormx(patchable)] does not apply no any field!");
    }

    let patch_struct = patch_struct(entity, &patch_struct_ident);
    let methods = methods(entity, &patch_struct_ident);

    quote! {
        #patch_struct
        #methods
    }
}

fn patch_struct(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let vis = &entity.vis;
    let fields = entity
        .patchable_fields()
        .map(|EntityField { ident, ty, .. }| quote!(#vis #ident: #ty));

    quote! {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        #vis struct #patch_struct_ident {
            #(#fields),*
        }
    }
}

fn methods(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let sql = format!(
        "UPDATE {} SET {} WHERE {} = $1",
        entity.table_name,
        entity
            .patchable_fields()
            .enumerate()
            .map(|(index, field)| format!("{} = {}", field.column_name, index + 2))
            .join(","),
        entity.id.column_name
    );
    let patchable_fields = entity
        .patchable_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let id_ty = &entity.id.ty;
    let id_ident = &entity.id.ident;
    let entity_ident = &entity.ident;
    let vis = &entity.vis;

    quote! {
        impl #patch_struct_ident {
            #vis async fn patch(
                &self,
                con: impl sqlx::Executor<'_, Database=sqlx::Postgres>,
                id: &#id_ty,
            ) -> sqlx::Result<()> {
                sqlx::query!(
                    #sql,
                    #(self.#patchable_fields,)*
                    id
                )
                .execute(con)
                .await?;

                Ok(())
            }
        }

        impl #entity_ident {
            #vis async fn patch(
                &mut self,
                con: impl sqlx::Executor<'_, Database=sqlx::Postgres>,
                update: #patch_struct_ident,
            ) -> sqlx::Result<()> {
                #patch_struct_ident::patch(&update, con, &self.#id_ident).await?;

                #(self.#patchable_fields = update.#patchable_fields;)*

                Ok(())
            }

            #vis fn to_patch(&self) -> #patch_struct_ident {
                #patch_struct_ident {
                    #(#patchable_fields: self.#patchable_fields.clone()),*
                }
            }
        }
    }
}
