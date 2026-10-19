use super::*;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{ImplItem, ItemImpl};

const MODES: [(LockMode, &str, &str); 4] = [
    (LockMode::Update, "update", "FOR UPDATE"),
    (LockMode::NoKeyUpdate, "no_key_update", "FOR NO KEY UPDATE"),
    (LockMode::Share, "share", "FOR SHARE"),
    (LockMode::KeyShare, "key_share", "FOR KEY SHARE"),
];
const KINDS: [(GetterKind, &str, &str, &str); 4] = [
    (GetterKind::One, "get_one", "Self", "fetch_one"),
    (
        GetterKind::Optional,
        "get_optional",
        "Option<Self>",
        "fetch_optional",
    ),
    (GetterKind::Many, "get_many", "Vec<Self>", "fetch_all"),
    (GetterKind::All, "get_all", "Vec<Self>", "fetch_all"),
];

fn entity(source: &str) -> Entity {
    Entity::try_from(syn::parse_str::<syn::DeriveInput>(source).unwrap()).unwrap()
}

fn compact(tokens: impl ToTokens) -> String {
    tokens
        .to_token_stream()
        .to_string()
        .split_whitespace()
        .collect()
}

fn error(source: &str) -> String {
    crate::derive_entity(syn::parse_str::<syn::DeriveInput>(source).unwrap())
        .expect_err("invalid attributes must be rejected")
        .to_string()
}

#[test]
fn every_locking_getter_has_independent_names_signatures_sql_and_fetch_behavior() {
    for (mode, suffix, clause) in MODES {
        for (kind, prefix, value_type, fetch) in KINDS {
            for custom_name in [false, true] {
                for custom_error in [false, true] {
                    let attr = format!("{prefix}_for_{suffix}");
                    let attr = if custom_name {
                        format!("{attr} = \"chosen\"")
                    } else {
                        attr
                    };
                    let (entity_attr, field_attr) = if kind == GetterKind::All {
                        (format!(", {attr}"), String::new())
                    } else {
                        (String::new(), format!(", {attr}"))
                    };
                    let error_attr = if custom_error {
                        ", error_type = \"TestError\""
                    } else {
                        ""
                    };
                    let record = entity(&format!(
                        "#[ormx(table = \"records\"{entity_attr}{error_attr})]
                         pub(crate) struct Record {{ #[ormx(key{field_attr})] id: i32, value: String }}"
                    ));
                    let methods = getters(&record).unwrap();
                    let implementation: ItemImpl =
                        syn::parse2(quote!(impl Record { #methods })).unwrap();
                    assert_eq!(
                        implementation.items.len(),
                        1,
                        "must not generate companion getters"
                    );
                    let ImplItem::Fn(method) = &implementation.items[0] else {
                        panic!("expected method")
                    };
                    let expected_name = if custom_name {
                        "chosen".into()
                    } else if kind == GetterKind::All {
                        format!("get_all_for_{suffix}")
                    } else {
                        format!("get_by_id_for_{suffix}")
                    };
                    assert_eq!(method.sig.ident, expected_name);
                    assert_eq!(compact(&method.vis), "pub(crate)");
                    assert!(method.sig.asyncness.is_some());
                    assert_eq!(
                        compact(&method.sig.inputs[0]),
                        "tx:&mutsqlx::Transaction<'_,sqlx::Postgres>"
                    );
                    assert_eq!(
                        method.sig.inputs.len(),
                        if kind == GetterKind::All { 1 } else { 2 }
                    );
                    if kind != GetterKind::All {
                        assert_eq!(compact(&method.sig.inputs[1]), "val:&i32");
                    }
                    let error_type = if custom_error {
                        "TestError"
                    } else {
                        "sqlx::Error"
                    };
                    assert_eq!(
                        compact(&method.sig.output),
                        format!("->Result<{value_type},{error_type}>")
                    );
                    let body = compact(&method.block);
                    assert!(body.contains(&format!(".{fetch}(&mut**tx).await")));
                    assert_eq!(body.matches("sqlx::query_as!").count(), 1);
                    assert_eq!(body.contains("Ok("), custom_error);
                    assert!(
                        !body.contains(".begin(")
                            && !body.contains(".commit(")
                            && !body.contains(".rollback(")
                    );

                    let field = (kind != GetterKind::All).then_some(&record.fields[0]);
                    let (sql, binds) =
                        build_select_query(&record, field, kind, Some(mode)).unwrap();
                    let mut expected = "SELECT id, value FROM records".to_owned();
                    if field.is_some() {
                        expected.push_str(" WHERE id = $1");
                    }
                    if kind.is_collection() {
                        expected.push_str(" ORDER BY records.id ASC");
                    }
                    expected.push_str(&format!(" {clause}"));
                    assert_eq!(sql, expected);
                    assert!(binds.is_empty());
                    assert!(method.block.to_token_stream().to_string().contains(
                        &syn::LitStr::new(&sql, Span::call_site())
                            .to_token_stream()
                            .to_string()
                    ));
                }
            }
        }
    }
}

#[test]
fn duplicate_locking_attributes_are_rejected_even_with_different_names() {
    for (_, suffix, _) in MODES {
        for (kind, prefix, _, _) in KINDS {
            let attrs = format!("{prefix}_for_{suffix}, {prefix}_for_{suffix} = \"other\"");
            let source = if kind == GetterKind::All {
                format!("#[ormx(table = \"records\", {attrs})] struct Record {{ #[ormx(key)] id: i32 }}")
            } else {
                format!("#[ormx(table = \"records\")] struct Record {{ #[ormx(key, {attrs})] id: i32 }}")
            };
            assert_eq!(
                error(&source),
                format!("duplicate ormx attribute `{prefix}_for_{suffix}`")
            );
        }
    }
}

#[test]
fn conflicting_getter_and_mutation_names_are_rejected() {
    for source in [
        "#[ormx(table = \"records\", get_all = \"chosen\", get_all_for_update = \"chosen\")] struct Record { #[ormx(key)] id: i32 }",
        "#[ormx(table = \"records\")] struct Record { #[ormx(key, get_one = \"chosen\", get_one_for_share = \"chosen\")] id: i32 }",
        "#[ormx(table = \"records\")] struct Record { #[ormx(key, get_one_for_share = \"chosen\")] id: i32, #[ormx(get_many_for_share = \"chosen\")] value: String }",
        "#[ormx(table = \"records\")] struct Record { #[ormx(key, get_one_for_share, get_optional_for_share)] id: i32 }",
        "#[ormx(table = \"records\")] struct Record { #[ormx(key, get_one_for_share = \"update\")] id: i32, value: String }",
        "#[ormx(table = \"records\", context_type = \"Context\")] struct Record { #[ormx(key, get_one_for_share = \"update_with_context\")] id: i32, value: String }",
        "#[ormx(table = \"records\", deletable)] struct Record { #[ormx(key, get_one_for_share = \"delete\")] id: i32 }",
        "#[ormx(table = \"records\", patchable)] struct Record { #[ormx(key, get_one_for_share = \"patch\")] id: i32, value: String }",
        "#[ormx(table = \"records\")] struct Record { #[ormx(key, get_one_for_share = \"chosen\")] id: i32, #[ormx(set = \"chosen\")] value: String }",
        "#[ormx(table = \"records\")] struct Record { #[ormx(key, get_one = \"r#chosen\", get_one_for_share = \"chosen\")] id: i32 }",
    ] {
        assert!(error(source).starts_with("conflicting generated getter name"), "{source}");
    }
}

#[test]
fn wrong_attribute_placement_and_invalid_names_are_rejected() {
    assert!(error(
        "#[ormx(table = \"records\", get_one_for_update)] struct Record { #[ormx(key)] id: i32 }"
    )
    .contains("unknown ormx attribute"));
    assert!(error(
        "#[ormx(table = \"records\")] struct Record { #[ormx(key, get_all_for_update)] id: i32 }"
    )
    .contains("unknown ormx attribute"));
    assert!(error("#[ormx(table = \"records\")] struct Record { #[ormx(key, get_one_for_update = \"bad-name\")] id: i32 }").contains("unexpected token"));
}

#[test]
fn collection_order_uses_physical_composite_keys_in_declaration_order() {
    let record = entity(
        r#"
        #[ormx(table = "public.records", get_all_for_update)]
        struct Record {
            #[ormx(key, rename = "second_key")] first: i32,
            #[ormx(key, rename = "first_key", custom_type)] second: Kind,
            #[ormx(get_many_for_share)] group_id: i32,
        }
    "#,
    );
    for (kind, field, mode, clause) in [
        (
            GetterKind::Many,
            Some(&record.fields[2]),
            LockMode::Share,
            " WHERE group_id = $1",
        ),
        (GetterKind::All, None, LockMode::Update, ""),
    ] {
        let (sql, _) = build_select_query(&record, field, kind, Some(mode)).unwrap();
        assert_eq!(sql, format!(
            "SELECT second_key AS first, first_key AS \"second: _\", group_id FROM public.records{clause} ORDER BY public.records.second_key ASC, public.records.first_key ASC {}", mode.sql()
        ));
    }
}

#[test]
fn transforms_keep_projection_aliases_and_bind_offsets_in_every_mode() {
    let record = entity(
        r#"
        #[ormx(table = "records")]
        struct Record {
            #[ormx(key, rename = "record_id")] id: i32,
            #[ormx(transform_get = "decrypt($field, $param1, $param2)", transform_get_params = "keys")] secret: String,
            #[ormx(custom_type, transform_get = "decode($field, $param1)", transform_get_params = "decoder")] kind: Option<Kind>,
        }
    "#,
    );
    for (mode, _, _) in MODES {
        for (kind, _, _, _) in KINDS {
            let field = (kind != GetterKind::All).then_some(&record.fields[0]);
            let (sql, binds) = build_select_query(&record, field, kind, Some(mode)).unwrap();
            let offset = usize::from(field.is_some());
            assert!(sql.starts_with(&format!(
                "SELECT record_id AS id, decrypt(secret, ${}, ${}) AS \"secret!\", decode(kind, ${}) AS \"kind?: _\" FROM records",
                offset + 1, offset + 2, offset + 3
            )));
            if field.is_some() {
                assert!(sql.contains(" WHERE record_id = $1"));
            }
            assert_eq!(binds.iter().map(|b| b.count).collect::<Vec<_>>(), [2, 1]);
            assert_eq!(
                bind_transform_params(&binds)
                    .expressions
                    .into_iter()
                    .map(compact)
                    .collect::<Vec<_>>(),
                [
                    "__ormx_transform_params_0.0",
                    "__ormx_transform_params_0.1",
                    "__ormx_transform_params_1.0"
                ]
            );
        }
    }
}

#[test]
fn both_lookup_conversion_forms_are_preserved() {
    for (attr, expected) in [
        ("convert_as = \"i32\"", "*valasi32"),
        ("convert = \"convert_id\"", "convert_id(&val)"),
    ] {
        let record = entity(&format!(
            "#[ormx(table = \"records\")] struct Record {{ #[ormx(key, custom_type, get_one_for_update, {attr})] id: Kind }}"
        ));
        let generated = compact(getters(&record).unwrap());
        assert!(generated.contains(&format!(",{expected})")), "{generated}");
        assert!(generated.contains("idAS\\\"id:_\\\""));
    }
}

#[test]
fn raw_identifiers_use_valid_default_names_and_physical_columns() {
    let record = entity(
        r#"
        #[ormx(table = "records", get_all_for_share)]
        struct Record { #[ormx(key, get_one_for_update)] r#type: i32 }
    "#,
    );
    assert_eq!(
        record.fields[0].locking_getters[0].name,
        "get_by_type_for_update"
    );
    let (sql, _) = build_select_query(
        &record,
        Some(&record.fields[0]),
        GetterKind::One,
        Some(LockMode::Update),
    )
    .unwrap();
    assert_eq!(sql, "SELECT type FROM records WHERE type = $1 FOR UPDATE");
    let (sql, _) =
        build_select_query(&record, None, GetterKind::All, Some(LockMode::Share)).unwrap();
    assert_eq!(
        sql,
        "SELECT type FROM records ORDER BY records.type ASC FOR SHARE"
    );
}

#[test]
fn unlocked_getters_keep_their_existing_sql_and_connection_contract() {
    let record = entity(
        r#"
        #[ormx(table = "records", get_all)]
        struct Record {
            #[ormx(key, get_one = "one", get_optional = "optional", get_many = "many")] id: i32,
            value: String,
        }
    "#,
    );
    for GetterDeclaration {
        field, kind, lock, ..
    } in declarations(&record)
    {
        assert!(lock.is_none());
        let (sql, _) = build_select_query(&record, field, kind, lock).unwrap();
        assert_eq!(
            sql,
            if kind == GetterKind::All {
                "SELECT id, value FROM records"
            } else {
                "SELECT id, value FROM records WHERE id = $1"
            }
        );
    }
    let methods = getters(&record).unwrap();
    let implementation: ItemImpl = syn::parse2(quote!(impl Record { #methods })).unwrap();
    assert_eq!(implementation.items.len(), 4);
    for item in implementation.items {
        let ImplItem::Fn(method) = item else {
            panic!("expected getter")
        };
        assert_eq!(
            compact(&method.sig.inputs[0]),
            "conn:&mutsqlx::PgConnection"
        );
        assert!(!compact(&method.block).contains("Transaction"));
    }
}
