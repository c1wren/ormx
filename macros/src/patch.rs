use crate::{Entity, EntityField};
use proc_macro2::{Span, TokenStream};
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
        .map(|EntityField { ident, ty, .. }| quote!(#vis #ident: Option<#ty>));

    let setters = entity
        .patchable_fields()
        .map(|EntityField { ident, ty, .. }| {
            let setter = Ident::new(&format!("set_{}", ident), Span::call_site());
            quote!(fn #setter(mut self, value: #ty) -> Self {
                self.#ident = Some(value);
                self
            })
        });

    quote! {
        #[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
        #vis struct #patch_struct_ident {
            #(#fields),*
        }

        impl #patch_struct_ident {
            #(#setters)*
        }
    }
}

fn methods(entity: &Entity, patch_struct_ident: &Ident) -> TokenStream {
    let patchable_fields = entity
        .patchable_fields()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let id_ty = &entity.id.ty;
    let id_ident = &entity.id.ident;
    let entity_ident = &entity.ident;
    let table_name = &entity.table_name;
    let vis = &entity.vis;

    let column_building = entity.patchable_fields().map(|field| {
        let ident = &field.ident;
        quote!(
            if self.#ident.is_some() {
                columns.push(format!("{} = ${}", stringify!(#ident), count));
                count += 1;
            }
        )
    });

    let binding = entity.patchable_fields().map(|field| {
        let ident = &field.ident;
        let value_getter = if let Some(convert_fn) = &field.convert {
            quote! { #convert_fn(value) }
        } else {
            quote! { value }
        };
        quote!(
            if let Some(value) = self.#ident.as_ref() {
                query = query.bind(#value_getter)
            }
        )
    });

    quote! {
        impl #patch_struct_ident {
            #vis async fn patch(
                &self,
                con: impl sqlx::Executor<'_, Database=sqlx::Postgres>,
                id: &#id_ty,
            ) -> sqlx::Result<()> {
                let mut columns = vec![];
                let mut count = 2;

                #(#column_building)*

                let columns = columns.join(", ");

                let sql = format!("UPDATE {} SET {} WHERE id = $1", #table_name, columns);

                let mut query = sqlx::query(&sql).bind(id);
                #(#binding)*

                query.execute(con).await?;

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

                #(if let Some(new_value) = update.#patchable_fields {
                    self.#patchable_fields = new_value;
                })*

                Ok(())
            }
        }
    }
}
