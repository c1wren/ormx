use crate::{Entity, EntityField, HelperAttr};
use proc_macro2::{Span, TokenTree};
use std::convert::TryFrom;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Token;
use syn::*;

fn duplicate<T>(_: T) -> Result<()> {
    Err(Error::new(Span::call_site(), "duplicate attribute"))
}

impl TryFrom<Field> for EntityField {
    type Error = Error;

    fn try_from(field: Field) -> Result<Self> {
        let mut generated = false;
        let mut rename = None;
        let mut get_one = None;
        let mut get_opt = None;
        let mut get_many = None;
        let mut set = None;
        for attr in HelperAttr::parse_all(&field.attrs)? {
            match attr {
                HelperAttr::Generated => generated = true,
                HelperAttr::Rename(r) => {
                    rename.replace(r).map_or(Ok(()), duplicate)?;
                }
                HelperAttr::GetOne(g) => {
                    get_one.replace(g).map_or(Ok(()), duplicate)?;
                }
                HelperAttr::GetOptional(g) => {
                    get_opt.replace(g).map_or(Ok(()), duplicate)?;
                }
                HelperAttr::GetMany(g) => {
                    get_many.replace(g).map_or(Ok(()), duplicate)?;
                }
                HelperAttr::Set(s) => {
                    set.replace(s).map_or(Ok(()), duplicate)?;
                }
                _ => return Err(Error::new(Span::call_site(), "unexpected attribute")),
            }
        }

        let rust_ident = field.ident.unwrap();
        Ok(EntityField {
            get_one: None,
            get_optional: None,
            get_many: None,
            set: None,
            ty: field.ty,
            db_ident: rename.unwrap_or_else(|| rust_ident.to_string()),
            rust_ident,
        })
    }
}

impl TryFrom<DeriveInput> for Entity {
    type Error = Error;

    fn try_from(input: DeriveInput) -> Result<Self> {
        let ident = input.ident.clone();
        let data_struct = match &input.data {
            Data::Struct(s) => s,
            _ => {
                return Err(Error::new(
                    input.span(),
                    "only structs can be used as entity",
                ))
            }
        };

        let fields = get_fields(input.span(), &data_struct)?
            .cloned()
            .map(EntityField::try_from)
            .collect::<Result<Vec<_>>>()?;

        let mut table_name = None;
        for attr in HelperAttr::parse_all(&input.attrs)? {
            match attr {
                HelperAttr::TableName(name) => {
                    table_name.replace(name).map_or(Ok(()), duplicate)?
                }
                other => return Err(Error::new(Span::call_site(), "unexpected attribute")),
            }
        }
        let table_name = table_name
            .ok_or_else(|| Error::new(Span::call_site(), r#"missing #[table = ".."] attribute"#))?;

        Ok(Entity {
            table_name,
            fields,
            ident,
        })
    }
}

fn get_fields<'a>(
    span: Span,
    input: &'a DataStruct,
) -> Result<impl Iterator<Item = &'a Field> + 'a> {
    match &input.fields {
        Fields::Named(FieldsNamed { named, .. }) => Ok(named.iter()),
        _ => {
            return Err(Error::new(
                span,
                "only structs with named fields can be used as entity",
            ))
        }
    }
}

impl HelperAttr {
    fn parse_all(attrs: &[Attribute]) -> Result<Vec<HelperAttr>> {
        let all = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("ormx"))
            .map(|attr| {
                attr.parse_args_with(Punctuated::<HelperAttr, Token![,]>::parse_separated_nonempty)
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(all)
    }
}

impl Parse for HelperAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match &*ident.to_string() {
            "generated" => Ok(Self::Generated),
            "table" => parse_assign::<LitStr>(ident.span(), &input)
                .map(|lit| lit.value())
                .map(Self::TableName),
            "rename" => parse_assign::<LitStr>(ident.span(), &input)
                .map(|lit| lit.value())
                .map(Self::Rename),
            "set" => parse_optional_assign::<Ident>(&input).map(Self::Set),
            "get_one" => parse_optional_assign::<Ident>(&input).map(Self::GetOne),
            "get_optional" => parse_optional_assign::<Ident>(&input).map(Self::GetOptional),
            "get_many" => parse_optional_assign::<Ident>(&input).map(Self::GetMany),
            other => Err(Error::new(
                ident.span(),
                &format!("unknown attribute key `{}`", other),
            )),
        }
    }
}

fn parse_assign<V: Parse>(span: Span, input: &ParseStream) -> Result<V> {
    parse_optional_assign(&input)?.ok_or_else(|| Error::new(span, "missing value"))
}

fn parse_optional_assign<V: Parse>(input: &ParseStream) -> Result<Option<V>> {
    Ok(if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        Some(input.parse()?)
    } else {
        None
    })
}
