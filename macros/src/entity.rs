use crate::attrs::{ConvertType, EntityAttr, FieldAttr};
use proc_macro2::Span;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::*;

#[derive(Clone)]
pub struct EntityField {
    pub get_one: Option<Ident>,
    pub get_optional: Option<Ident>,
    pub get_many: Option<Ident>,
    pub set: Option<Ident>,
    pub delete: Option<Ident>,
    pub convert: Option<ConvertType>,
    pub column_name: String,
    pub ident: Ident,
    pub ty: Type,

    pub default: bool,
    pub updatable: bool,
    pub patchable: bool,
    pub custom_type: bool,
}

impl EntityField {
    pub fn fmt_for_select(&self) -> String {
        let ident = self.ident.to_string().replace("r#", "");
        let column_name = self.column_name.to_string().replace("r#", "");

        if self.custom_type {
            return format!(r#"{} AS "{}: _""#, column_name, ident);
        }

        ident
    }
}

pub struct Entity {
    pub table_name: String,
    pub id: EntityField,
    pub ident: Ident,
    pub fields: Vec<EntityField>,
    pub vis: Visibility,

    pub get_all: Option<Ident>,
    pub delete: Option<Ident>,

    pub insert: Option<Ident>,
    pub patch: Option<Ident>,
}

impl Entity {
    pub fn insertable_fields(&self) -> impl Iterator<Item = &EntityField> {
        let id = self.id.ident.clone();
        self.fields
            .iter()
            .filter(move |field| !(id == field.ident || field.default))
    }

    pub fn patchable_fields(&self) -> impl Iterator<Item = &EntityField> {
        let id = self.id.ident.clone();
        self.fields
            .iter()
            .filter(move |field| field.ident != id && field.patchable)
    }

    pub fn updatable_fields(&self) -> impl Iterator<Item = &EntityField> {
        let id = self.id.ident.clone();
        self.fields
            .iter()
            .filter(move |field| field.ident != id && field.updatable)
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
            delete: None,
            set: None,
            convert: None,
            ty: field.ty.clone(),
            column_name: ident.to_string(),
            ident: ident.clone(),
            updatable: true,
            patchable: true,
            default: false,
            custom_type: false,
        };
        for attr in crate::attrs::parse_all::<FieldAttr>(&field.attrs)? {
            match attr {
                FieldAttr::Rename(r) => result.column_name = r,
                FieldAttr::GetOne(g) => {
                    let fn_name = g.unwrap_or_else(|| get_ident("get_by"));
                    result.get_one = Some(fn_name)
                }
                FieldAttr::GetOptional(g) => {
                    let fn_name = g.unwrap_or_else(|| get_ident("get_by"));
                    result.get_optional = Some(fn_name);
                }
                FieldAttr::GetMany(g) => {
                    let fn_name = g.unwrap_or_else(|| get_ident("get_by"));
                    result.get_many = Some(fn_name);
                }
                FieldAttr::Delete(g) => {
                    let fn_name = g.unwrap_or_else(|| get_ident("delete_by"));
                    result.delete = Some(fn_name)
                }
                FieldAttr::Set(s) => {
                    let fn_name = s.unwrap_or_else(|| get_ident("set"));
                    result.set = Some(fn_name);
                }
                FieldAttr::Updatable(updatable) => result.updatable = updatable,
                FieldAttr::Patchable(patchable) => result.patchable = patchable,
                FieldAttr::Default => {
                    result.default = true;
                }
                FieldAttr::CustomType => {
                    result.custom_type = true;
                }
                FieldAttr::Convert(function) => result.convert = Some(function),
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

        let mut table_name = None;
        let mut id = None;
        let mut insert = None;
        let mut patch = None;
        let mut get_all = None;
        let mut delete = None;
        for attr in crate::attrs::parse_all::<EntityAttr>(&input.attrs)? {
            match attr {
                EntityAttr::Table(name) => table_name.replace(name).map_or(Ok(()), duplicate)?,
                EntityAttr::Id(new_id) => id.replace(new_id).map_or(Ok(()), duplicate)?,
                EntityAttr::Insertable(struct_ident) => {
                    let struct_ident = struct_ident.unwrap_or_else(|| {
                        Ident::new(&format!("Insert{}", ident), Span::call_site())
                    });
                    insert.replace(struct_ident).map_or(Ok(()), duplicate)?
                }
                EntityAttr::Patchable(struct_ident) => {
                    let struct_ident = struct_ident.unwrap_or_else(|| {
                        Ident::new(&format!("Patch{}", ident), Span::call_site())
                    });
                    patch.replace(struct_ident).map_or(Ok(()), duplicate)?
                }
                EntityAttr::GetAll(fun) => {
                    let fun = fun.unwrap_or_else(|| Ident::new("get_all", Span::call_site()));
                    get_all.replace(fun).map_or(Ok(()), duplicate)?
                }
                EntityAttr::Deletable(fun) => {
                    let fun = fun.unwrap_or_else(|| Ident::new("delete", Span::call_site()));
                    delete.replace(fun).map_or(Ok(()), duplicate)?
                }
            }
        }
        let table_name = table_name.ok_or_else(|| missing_attr("table"))?;
        let id = id.ok_or_else(|| missing_attr("id"))?;

        let fields = get_fields(input.span(), &data_struct)?
            .into_iter()
            .map(EntityField::try_from)
            .collect::<Result<Vec<_>>>()?;

        Ok(Entity {
            table_name,
            id: fields
                .iter()
                .find(|field| field.ident == id)
                .expect("the struct does not have this field")
                .clone(),
            fields,
            ident,
            vis: input.vis,
            insert,
            patch,
            get_all,
            delete,
        })
    }
}

fn get_fields(span: Span, input: &DataStruct) -> Result<Vec<&Field>> {
    match &input.fields {
        Fields::Named(FieldsNamed { named, .. }) => Ok(named.iter().collect()),
        _ => Err(Error::new(
            span,
            "only structs with named fields can be used as entity",
        )),
    }
}

fn duplicate<T>(_: T) -> Result<()> {
    Err(Error::new(Span::call_site(), "duplicate attribute"))
}

fn missing_attr(name: &str) -> Error {
    Error::new(
        Span::call_site(),
        &format!(r#"missing #[ormx({} = "..")) attribute"#, name),
    )
}
