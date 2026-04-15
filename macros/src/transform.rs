use crate::EntityField;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{Error, ExprPath, Result};

#[derive(Clone)]
pub struct TransformBinding {
    pub params_fn: ExprPath,
    pub count: usize,
}

pub fn count_transform_params(sql_template: &str) -> usize {
    let mut max_param = 0;
    let bytes = sql_template.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'$' && sql_template[index..].starts_with("$param") {
            index += "$param".len();
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if start != index {
                let number = sql_template[start..index].parse::<usize>().unwrap_or(0);
                max_param = max_param.max(number);
            }
            continue;
        }
        index += 1;
    }

    max_param
}

pub fn process_transform(
    sql_template: &str,
    field_ref: &str,
    param_offset: usize,
) -> Result<(String, usize)> {
    let max_param = count_transform_params(sql_template);
    let mut sql = sql_template.replace("$field", field_ref);

    for param_number in (1..=max_param).rev() {
        let from = format!("$param{param_number}");
        let to = format!("${}", param_offset + param_number);
        sql = sql.replace(&from, &to);
    }

    if sql.contains("$param") {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "invalid transform parameter placeholder",
        ));
    }

    Ok((sql, max_param))
}

pub fn transform_bind_expressions(bindings: &[TransformBinding]) -> Vec<TokenStream> {
    bindings
        .iter()
        .flat_map(|binding| {
            let params_fn = &binding.params_fn;
            (0..binding.count).map(move |i| {
                let index = syn::Index::from(i);
                quote! { #params_fn().#index }
            })
        })
        .collect()
}

pub fn build_select_clause(
    fields: &[EntityField],
    initial_offset: usize,
) -> Result<(String, Vec<TransformBinding>, usize)> {
    let mut bindings = Vec::new();
    let mut param_offset = initial_offset;
    let mut columns = Vec::with_capacity(fields.len());

    for field in fields {
        if let Some(transform_get) = &field.transform_get {
            let column_name = field.column_name.replace("r#", "");
            let (expr, count) = process_transform(transform_get, &column_name, param_offset)?;
            param_offset += count;

            if let Some(params_fn) = &field.transform_get_params {
                bindings.push(TransformBinding {
                    params_fn: params_fn.clone(),
                    count,
                });
            }

            columns.push(format!("{} AS {}", expr, field.fmt_expression_alias()));
        } else {
            columns.push(field.fmt_for_select());
        }
    }

    Ok((columns.join(", "), bindings, param_offset))
}

pub fn runtime_transform_expr(
    template: &str,
    field_expr: TokenStream,
    param_offset_expr: TokenStream,
    max_param: usize,
) -> TokenStream {
    let template_lit = Literal::string(template);
    let replacements = (1..=max_param).map(|param_number| {
        let needle = Literal::string(&format!("$param{param_number}"));
        quote! {
            expr = expr.replace(#needle, &format!("${}", #param_offset_expr + #param_number));
        }
    });

    quote! {{
        let mut expr = #template_lit.replace("$field", &#field_expr);
        #(#replacements)*
        expr
    }}
}
