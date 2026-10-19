use std::collections::BTreeSet;

use super::*;
use quote::quote;
use syn::{File, ImplItem, Item};

#[test]
fn registered_names_match_emitted_entity_methods() {
    for context in [false, true] {
        for hooks in ["", "before", "after", "both"] {
            let mut attrs = String::new();
            if context {
                attrs.push_str(", context_type = \"Context\"");
            }
            for action in ["update", "patch", "delete"] {
                for timing in ["before", "after"] {
                    if hooks == timing || hooks == "both" {
                        attrs.push_str(&format!(", {timing}_{action} = \"hook\""));
                    }
                }
            }
            let input: syn::DeriveInput = syn::parse_str(&format!(
                "#[ormx(table = \"records\", update = \"save\", patchable, deletable{attrs})]
                 struct Record {{ #[ormx(key)] id: i32,
                     #[ormx(set = \"rename\", delete = \"delete_by_value\")] value: String }}"
            ))
            .unwrap();
            let entity = Entity::try_from(input).unwrap();
            let setters = crate::set::setters(&entity);
            let update = crate::update::update(&entity);
            let delete = crate::delete::delete(&entity);
            let patch = crate::patch::patch(&entity);
            let generated: File = syn::parse2(quote! {
                impl Record { #setters #update #delete }
                #patch
            })
            .unwrap();
            let mut emitted = BTreeSet::new();
            for item in generated.items {
                let Item::Impl(implementation) = item else {
                    continue;
                };
                if !matches!(implementation.self_ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("Record"))
                {
                    continue;
                }
                for item in implementation.items {
                    if let ImplItem::Fn(method) = item {
                        assert!(emitted.insert(method.sig.ident.to_string()));
                    }
                }
            }
            let registered = mutation_names(&entity)
                .map(|name| name.to_string())
                .collect::<BTreeSet<_>>();
            assert_eq!(registered, emitted, "context={context}, hooks={hooks}");
            assert!(registered.contains("save"));
            assert_eq!(registered.contains("save_with_context"), context);
            assert_eq!(registered.contains("no_trigger_update"), !hooks.is_empty());
            assert!(!registered.contains("no_trigger_save"));
        }
    }
}

#[test]
fn all_key_entities_do_not_reserve_update_methods() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[ormx(table = "records")]
        struct Record { #[ormx(key, get_one_for_update = "update")] id: i32 }
    };
    let entity = Entity::try_from(input).unwrap();
    assert!(mutation_names(&entity).next().is_none());
    assert!(!crate::get::getters(&entity).unwrap().is_empty());
}
