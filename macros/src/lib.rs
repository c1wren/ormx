extern crate proc_macro;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::convert::TryFrom;
use syn::*;

pub(crate) use entity::{Entity, EntityField};

mod attrs;
mod delete;
mod entity;
mod get;
mod insert;
mod patch;
mod set;
mod update;

fn derive_entity(input: DeriveInput) -> Result<TokenStream2> {
    let ty = input.ident.clone();
    let entity = Entity::try_from(input)?;

    let getters = get::getters(&entity);
    let setters = set::setters(&entity);
    let insert = insert::insert(&entity);
    let patch = patch::patch(&entity);
    let delete = delete::delete(&entity);
    let update = update::update(&entity);

    Ok(quote! {
        impl #ty {
            #getters
            #setters
            #update
            #delete
        }

        #insert
        #patch
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
