use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, LitBool, LitStr, Result, Token};

pub enum EntityAttr {
    Table(String),
    Id(Ident),
    Insertable(Option<Ident>),
    Patchable(Option<Ident>),
    GetAll(Option<Ident>),
    Deletable(Option<Ident>),
}

impl Parse for EntityAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match &*ident.to_string() {
            "table" => parse_assign::<LitStr>(ident.span(), &input)
                .map(|lit| lit.value())
                .map(Self::Table),
            "id" => parse_assign::<Ident>(ident.span(), &input).map(Self::Id),
            "insertable" => parse_optional_assign::<Ident>(&input).map(Self::Insertable),
            "patchable" => parse_optional_assign::<Ident>(&input).map(Self::Patchable),
            "deletable" => parse_optional_assign::<Ident>(&input).map(Self::Deletable),
            "get_all" => parse_optional_assign::<Ident>(&input).map(Self::GetAll),
            other => Err(Error::new(
                ident.span(),
                &format!("unknown ormx attribute: `{}`", other),
            )),
        }
    }
}

pub enum FieldAttr {
    Rename(String),
    GetOne(Option<Ident>),
    GetOptional(Option<Ident>),
    GetMany(Option<Ident>),
    Delete(Option<Ident>),
    Set(Option<Ident>),
    Updatable(bool),
    Patchable(bool),
    Generated,
    CustomType,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        match &*ident.to_string() {
            "rename" => parse_assign::<LitStr>(ident.span(), &input)
                .map(|lit| lit.value())
                .map(Self::Rename),
            "set" => parse_optional_assign::<Ident>(&input).map(Self::Set),
            "get_one" => parse_optional_assign::<Ident>(&input).map(Self::GetOne),
            "get_optional" => parse_optional_assign::<Ident>(&input).map(Self::GetOptional),
            "get_many" => parse_optional_assign::<Ident>(&input).map(Self::GetMany),
            "delete" => parse_optional_assign::<Ident>(&input).map(Self::Delete),
            "generated" => Ok(Self::Generated),
            "custom_type" => Ok(Self::CustomType),
            "patchable" => Ok(Self::Patchable(
                parse_optional_assign::<LitBool>(&input)?
                    .map(|lit| lit.value)
                    .unwrap_or(true),
            )),
            "updatable" => Ok(Self::Updatable(
                parse_optional_assign::<LitBool>(&input)?
                    .map(|lit| lit.value)
                    .unwrap_or(true),
            )),
            other => Err(Error::new(
                ident.span(),
                &format!("unknown ormx attribute: `{}`", other),
            )),
        }
    }
}

pub fn parse_all<P: Parse>(attrs: &[Attribute]) -> Result<Vec<P>> {
    let all = attrs
        .iter()
        .filter(|attr| attr.path.is_ident("ormx"))
        .map(|attr| attr.parse_args_with(Punctuated::<P, Token![,]>::parse_separated_nonempty))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(all)
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
