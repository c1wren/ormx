extern crate proc_macro;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::convert::TryFrom;
use syn::*;

pub(crate) use parse::{Entity, EntityField};

mod delete;
mod get;
mod insert;
mod parse;
mod set;
mod update;
mod utils;

fn connection_type() -> TokenStream2 {
    #[cfg(feature = "sqlite")]
    return quote!(sqlx::SqliteConnection);
    #[cfg(feature = "mysql")]
    return quote!(sqlx::MySqlConnection);
    #[cfg(feature = "postgres")]
    return quote!(sqlx::PostgresConnection);
}

impl Entity {
    pub(crate) fn generated_fields(&self) -> impl Iterator<Item = &EntityField> {
        self.fields.iter().filter(|field| field.generated)
    }

    pub(crate) fn data_fields(&self) -> impl Iterator<Item = &EntityField> {
        self.fields.iter().filter(|field| !field.generated)
    }

    pub(crate) fn updatable_fields(&self) -> impl Iterator<Item = &EntityField> {
        self.fields.iter().filter(|field| field.updatable)
    }

    pub(crate) fn getters(&self) -> TokenStream2 {
        self.fields
            .iter()
            .flat_map(|field| {
                std::iter::empty()
                    .chain(
                        field
                            .get_one
                            .as_ref()
                            .map(|func| get::single(self, field, func)),
                    )
                    .chain(
                        field
                            .get_optional
                            .as_ref()
                            .map(|func| get::optional(self, field, func)),
                    )
                    .chain(
                        field
                            .get_many
                            .as_ref()
                            .map(|func| get::many(self, field, func)),
                    )
            })
            .collect()
    }

    fn setters(&self) -> Result<TokenStream2> {
        let fields = self
            .fields
            .iter()
            .flat_map(|field| match &field.set {
                Some(set) => Some((field, set)),
                None => None,
            })
            .collect::<Vec<_>>();

        if fields.is_empty() {
            return Ok(quote!());
        }

        let primary_key = match &self.primary_key {
            Some(primary_key) => primary_key,
            None => {
                return Err(Error::new(
                    Span::call_site(),
                    "#[set] requires a primary key",
                ))
            }
        };

        fields
            .into_iter()
            .map(|(field, set)| set::set(self, primary_key, field, set))
            .collect()
    }
}

fn derive_entity(input: DeriveInput) -> Result<TokenStream2> {
    let ty = input.ident.clone();
    let entity = Entity::try_from(input)?;
    let getters = entity.getters();
    let setters = entity.setters()?;
    let insert_struct = insert::insert_struct(&entity);
    let insert_fn = insert::insert_fn(&entity)?;
    let update_struct = update::update_struct(&entity);
    let update_fn = update::update_fn(&entity);
    let delete = delete::delete(&entity);

    Ok(quote! {
        impl #ty {
            #getters
            #setters
            #delete
            #insert_fn
            #update_fn
        }

        #insert_struct
        #update_struct
    })
}

#[proc_macro_derive(Entity, attributes(ormx))]
pub fn derive_entity_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_entity(input) {
        Ok(out) => out,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

