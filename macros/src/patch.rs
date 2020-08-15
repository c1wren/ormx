use crate::Entity;
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
    let patch_method = patch_method(entity, &patch_struct_ident);
    let to_patch_method = to_patch_method(entity, &patch_struct_ident);
    let entity_ident = &entity.ident;

    quote! {
        #patch_struct
        impl #entity_ident {
            #patch_method
            #to_patch_method
        }
    }
}

fn patch_struct(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let vis = &entity.vis;
    let fields = entity.patchable_fields();

    quote! {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        #vis struct #patch_struct_ident {
            #(#fields),*
        }
    }
}

fn patch_method(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let query = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        entity.table_name,
        entity
            .patchable_fields()
            .map(|field| format!("{} = ?", field.column_name))
            .join(","),
        entity.id.column_name
    );
    let update_fields = entity
        .patchable_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let id_ident = &entity.id.ident;
    let vis = &entity.vis;

    quote! {
        #vis async fn patch(
            &mut self,
            con: impl sqlx::Executor<'_, Database=sqlx::MySql>,
            update: #patch_struct_ident,
        ) -> sqlx::Result<()> {
            sqlx::query!(
                #query,
                #(update.#update_fields,)*
                self.#id_ident,
            )
            .execute(con)
            .await?;

            #(self.#update_fields = update.#update_fields;)*

            Ok(())
        }
    }
}

fn to_patch_method(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let vis = &entity.vis;
    let patchable_fields = entity
        .patchable_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();

    quote! {
        #vis fn to_patch(&self) -> #patch_struct_ident {
            #patch_struct_ident {
                #(#patchable_fields: self.#patchable_fields.clone()),*
            }
        }
    }
}
