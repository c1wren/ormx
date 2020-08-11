use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use std::convert::TryFrom;
use std::fmt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;

#[derive(Clone)]
pub struct EntityField {
    pub get_one: Option<Ident>,
    pub get_optional: Option<Ident>,
    pub get_many: Option<Ident>,
    pub set: Option<Ident>,
    pub generated: bool,
    pub primary_key: bool,
    pub updatable: bool,
    pub column_name: String,
    pub ident: Ident,
    pub ty: Type,
}

pub struct Entity {
    pub table_name: String,
    pub ident: Ident,
    pub fields: Vec<EntityField>,
    pub primary_key: Option<EntityField>,
    pub visibility: Visibility,
}

pub type Accessor = Option<Ident>;

pub enum HelperAttr {
    Generated,
    TableName(String),
    Rename(String),
    GetOne(Accessor),
    GetOptional(Accessor),
    GetMany(Accessor),
    Set(Accessor),
    Update(bool),
    PrimaryKey,
}

impl ToTokens for EntityField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let ty = &self.ty;
        tokens.extend(quote!(pub #ident: #ty));
    }
}

impl fmt::Display for HelperAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#[ormx(")?;
        match self {
            HelperAttr::Generated => write!(f, "generated"),
            HelperAttr::TableName(table) => write!(f, "table = {:?}", table),
            HelperAttr::Rename(rename) => write!(f, "rename = {:?}", rename),
            HelperAttr::GetOne(None) => write!(f, "get_one"),
            HelperAttr::GetOne(Some(func)) => write!(f, "get_one = {}", func),
            HelperAttr::GetOptional(None) => write!(f, "get_optional"),
            HelperAttr::GetOptional(Some(func)) => write!(f, "get_optional = {}", func),
            HelperAttr::GetMany(None) => write!(f, "get_many"),
            HelperAttr::GetMany(Some(func)) => write!(f, "get_many = {}", func),
            HelperAttr::Set(None) => write!(f, "set"),
            HelperAttr::Set(Some(func)) => write!(f, "set = {}", func),
            HelperAttr::PrimaryKey => write!(f, "primary_key"),
            HelperAttr::Update(true) => write!(f, "update(include)"),
            HelperAttr::Update(false) => write!(f, "update(exclude)"),
        }?;
        write!(f, ")]")
    }
}

impl TryFrom<&Field> for EntityField {
    type Error = Error;

    fn try_from(field: &Field) -> Result<Self> {
        let ident = field.ident.clone().unwrap();
        let get_ident =
            |prefix: &str| Ident::new(&format!("{}_{}", prefix, ident), Span::call_site());

        let mut result = EntityField {
            get_one: None,
            get_optional: None,
            get_many: None,
            set: None,
            ty: field.ty.clone(),
            column_name: ident.to_string(),
            ident: ident.clone(),
            generated: false,
            primary_key: false,
            updatable: true,
        };
        for attr in HelperAttr::parse_all(&field.attrs)? {
            match attr {
                HelperAttr::Generated => {
                    result.generated = true;
                    result.updatable = false;
                }
                HelperAttr::PrimaryKey => {
                    result.primary_key = true;
                    result.updatable = false;
                }
                HelperAttr::Rename(r) => result.column_name = r,
                HelperAttr::GetOne(g) => {
                    let fn_name = g.unwrap_or_else(|| get_ident("get_by"));
                    result.get_one = Some(fn_name)
                }
                HelperAttr::GetOptional(g) => {
                    let fn_name = g.unwrap_or_else(|| get_ident("get_by"));
                    result.get_optional = Some(fn_name);
                }
                HelperAttr::GetMany(g) => {
                    let fn_name = g.unwrap_or_else(|| get_ident("get_by"));
                    result.get_many = Some(fn_name);
                }
                HelperAttr::Set(s) => {
                    let fn_name = s.unwrap_or_else(|| get_ident("set"));
                    result.set = Some(fn_name);
                }
                HelperAttr::Update(updatable) => {
                    result.updatable = updatable;
                }
                x => {
                    return Err(Error::new(
                        Span::call_site(),
                        format!("unexpected attribute {}", x),
                    ))
                }
            }
        }

        Ok(result)
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

        let fields = get_fields(input.span(), &data_struct)?;
        let fields = fields
            .into_iter()
            .map(EntityField::try_from)
            .collect::<Result<Vec<_>>>()?;

        let mut table_name = None;
        for attr in HelperAttr::parse_all(&input.attrs)? {
            match attr {
                HelperAttr::TableName(name) => {
                    table_name.replace(name).map_or(Ok(()), duplicate)?
                }
                _ => return Err(Error::new(Span::call_site(), "unexpected attribute")),
            }
        }
        let table_name = table_name
            .ok_or_else(|| Error::new(Span::call_site(), r#"missing #[ormx(table = "..")] attribute"#))?;

        let mut primary_key: Option<EntityField> = None;
        for field in fields.iter() {
            if field.primary_key {
                if primary_key.is_some() {
                    return Err(Error::new(Span::call_site(), "duplicate primary key"));
                }
                primary_key = Some(field.clone());
            }
        }

        Ok(Entity {
            table_name,
            fields,
            ident,
            primary_key,
            visibility: input.vis,
        })
    }
}

fn get_fields(span: Span, input: &DataStruct) -> Result<Vec<&Field>> {
    match &input.fields {
        Fields::Named(FieldsNamed { named, .. }) => Ok(named.iter().collect()),
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
            "primary_key" => Ok(Self::PrimaryKey),
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
            "update" => {
                let content;
                parenthesized!(content in input);
                let ident = content.parse::<Ident>()?;
                if !content.is_empty() {
                    panic!("unexpected {:?}", content);
                }
                match &*ident.to_string() {
                    "exclude" => Ok(HelperAttr::Update(false)),
                    "include" => Ok(HelperAttr::Update(true)),
                    other => {
                        return Err(Error::new(
                            Span::call_site(),
                            format!("expected 'exclude' or 'include', got '{}'", other),
                        ))
                    }
                }
            }
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

fn duplicate<T>(_: T) -> Result<()> {
    Err(Error::new(Span::call_site(), "duplicate attribute"))
}
