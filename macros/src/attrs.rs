use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Error, ExprPath, LitBool, LitStr, Result, Token};

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
            "id" => Id(assign_ident(ident.span(), &input)?),
            "insertable" => Insertable(opt_assign_ident(&input)?),
            "patchable" => Patchable(opt_assign_ident(&input)?),
            "deletable" => Deletable(opt_assign_ident(&input)?),
            "get_all" => GetAll(opt_assign_ident(&input)?),
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
    Convert(ExprPath),
    Generated,
    CustomType,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        use FieldAttr::*;

        let ident = input.parse::<Ident>()?;
        let attr = match &*ident.to_string() {
            "rename" => Rename(assign_string(ident.span(), &input)?),
            "set" => Set(opt_assign_ident(&input)?),
            "get_one" => GetOne(opt_assign_ident(&input)?),
            "get_optional" => GetOptional(opt_assign_ident(&input)?),
            "get_many" => GetMany(opt_assign_ident(&input)?),
            "delete" => Delete(opt_assign_ident(&input)?),
            "generated" => Generated,
            "custom_type" => CustomType,
            "patchable" => Patchable(opt_assign_bool(&input)?.unwrap_or(true)),
            "updatable" => Updatable(opt_assign_bool(&input)?.unwrap_or(true)),
            "convert" => Convert(assign_expr_path(ident.span(), &input)?),
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

fn assign_ident(span: Span, input: &ParseStream) -> Result<Ident> {
    opt_assign_ident(&input)?.ok_or_else(|| Error::new(span, "missing value"))
}

fn opt_assign_ident(input: &ParseStream) -> Result<Option<Ident>> {
    if let Some(lit_str) = opt_assign(&input)? {
        let tokens = spanned_tokens(&lit_str)?;
        Ok(Some(syn::parse2(tokens)?))
    } else {
        Ok(None)
    }
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

fn assign_expr_path(span: Span, input: &ParseStream) -> Result<ExprPath> {
    let lit_str = assign::<LitStr>(span, input)?;
    let tokens = spanned_tokens(&lit_str)?;
    syn::parse2(tokens)
}

fn spanned_tokens(s: &syn::LitStr) -> Result<TokenStream> {
    let stream = syn::parse_str(&s.value())?;
    Ok(respan_token_stream(stream, s.span()))
}

fn respan_token_stream(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| respan_token_tree(token, span))
        .collect()
}

fn respan_token_tree(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(g) = &mut token {
        *g = Group::new(g.delimiter(), respan_token_stream(g.stream(), span));
    }
    token.set_span(span);
    token
}
