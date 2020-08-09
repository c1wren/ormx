extern crate proc_macro;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::convert::TryFrom;
use std::fmt;
use syn::export::Formatter;
use syn::*;

mod get;
mod insert;
mod parse;
mod set;
mod utils;

#[derive(Clone)]
pub(crate) struct EntityField {
    get_one: Option<Ident>,
    get_optional: Option<Ident>,
    get_many: Option<Ident>,
    set: Option<Ident>,
    generated: bool,
    primary_key: bool,

    db_ident: String,
    rust_ident: Ident,
    ty: Type,
}

pub(crate) struct Entity {
    table_name: String,
    ident: Ident,
    fields: Vec<EntityField>,
    primary_key: Option<EntityField>,
}

fn connection_type() -> TokenStream2 {
    #[cfg(feature = "sqlite")]
    return quote!(sqlx::SqliteConnection);
    #[cfg(feature = "mysql")]
    return quote!(sqlx::MySqlConnection);
    #[cfg(feature = "postgres")]
    return quote!(sqlx::PostgresConnection);
}

impl Entity {
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
            return Ok(quote!())
        }

        let primary_key = match &self.primary_key {
            Some(primary_key) => primary_key,
            None => return Err(Error::new(Span::call_site(), "#[set] requires a primary key"))
        };

        fields.into_iter()
            .map(|(field, set)| set::set(self, primary_key, field, set))
            .collect()
    }

    fn gen_insert(&self) -> TokenStream2 {
        let fields = self
            .fields
            .iter()
            .map(|EntityField { rust_ident, ty, .. }| quote!(#rust_ident));

        quote! {}
    }
}

fn derive_entity(input: DeriveInput) -> Result<TokenStream2> {
    let ty = &input.ident;
    let entity = Entity::try_from(&input)?;
    let getters = entity.getters();
    let setters = entity.setters()?;

    Ok(quote! {
        impl #ty {
            #getters
            #setters
        }
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

fn function_name(prefix: &str, field: &Ident, rename: &Option<Ident>) -> Ident {
    match rename {
        None => Ident::new(&format!("{}_{}", prefix, field), Span::call_site()),
        Some(accessor) => accessor.clone(),
    }
}
