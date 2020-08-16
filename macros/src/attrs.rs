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
        use EntityAttr::*;

        let ident = input.parse::<Ident>()?;
        let attr = match &*ident.to_string() {
            "table" => Table(assign_string(ident.span(), &input)?),
            "id" => Id(assign(ident.span(), &input)?),
            "insertable" => Insertable(opt_assign(&input)?),
            "patchable" => Patchable(opt_assign(&input)?),
            "deletable" => Deletable(opt_assign(&input)?),
            "get_all" => GetAll(opt_assign(&input)?),
            other => {
                return Err(Error::new(
                    ident.span(),
                    &format!("unknown ormx attribute: `{}`", other),
                ))
            }
        };
        Ok(attr)
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
        use FieldAttr::*;

        let ident = input.parse::<Ident>()?;
        let attr = match &*ident.to_string() {
            "rename" => Rename(assign_string(ident.span(), &input)?),
            "set" => Set(opt_assign(&input)?),
            "get_one" => GetOne(opt_assign(&input)?),
            "get_optional" => GetOptional(opt_assign(&input)?),
            "get_many" => GetMany(opt_assign(&input)?),
            "delete" => Delete(opt_assign(&input)?),
            "generated" => Generated,
            "custom_type" => CustomType,
            "patchable" => Patchable(opt_assign_bool(&input)?.unwrap_or(true)),
            "updatable" => Updatable(opt_assign_bool(&input)?.unwrap_or(true)),
            other => {
                return Err(Error::new(
                    ident.span(),
                    &format!("unknown ormx attribute: `{}`", other),
                ))
            }
        };
        Ok(attr)
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

fn assign<V: Parse>(span: Span, input: &ParseStream) -> Result<V> {
    opt_assign(&input)?.ok_or_else(|| Error::new(span, "missing value"))
}

fn opt_assign<V: Parse>(input: &ParseStream) -> Result<Option<V>> {
    Ok(if input.peek(Token![=]) {
        input.parse::<Token![=]>()?;
        Some(input.parse()?)
    } else {
        None
    })
}

fn opt_assign_bool(input: &ParseStream) -> Result<Option<bool>> {
    let parsed = opt_assign::<LitBool>(&input)?.map(|lit| lit.value);
    Ok(parsed)
}

fn assign_string(span: Span, input: &ParseStream) -> Result<String> {
    assign::<LitStr>(span, input).map(|lit| lit.value())
}
