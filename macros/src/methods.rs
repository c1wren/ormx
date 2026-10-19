use crate::{Entity, EntityField};
use proc_macro2::{Ident, Span};

/// Mutation method names shared by code generation and getter collision checks.
pub struct MethodNames {
    pub main: Ident,
    pub with_context: Option<Ident>,
    pub no_trigger: Option<Ident>,
}

impl MethodNames {
    fn new(main: Ident, context: bool, no_trigger: Option<String>) -> Self {
        let base = main.to_string();
        let base = base.trim_start_matches("r#");
        Self {
            with_context: context.then(|| Ident::new(&format!("{base}_with_context"), main.span())),
            no_trigger: no_trigger.map(|name| Ident::new(&name, main.span())),
            main,
        }
    }

    fn into_names(self) -> impl Iterator<Item = Ident> {
        std::iter::once(self.main)
            .chain(self.with_context)
            .chain(self.no_trigger)
    }
}

pub fn update(entity: &Entity) -> Option<MethodNames> {
    entity.updatable_fields().next()?;
    Some(MethodNames::new(
        entity.update.clone(),
        entity.context_type.is_some(),
        (entity.before_update.is_some() || entity.after_update.is_some())
            .then(|| "no_trigger_update".into()),
    ))
}

pub fn setter(entity: &Entity, field: &EntityField) -> Option<MethodNames> {
    let name = field.set.as_ref()?;
    Some(MethodNames::new(
        name.clone(),
        entity.context_type.is_some(),
        (entity.before_update.is_some() || entity.after_update.is_some())
            .then(|| format!("no_trigger_{}", name.to_string().trim_start_matches("r#"))),
    ))
}

pub fn delete(entity: &Entity) -> Option<MethodNames> {
    entity.delete.as_ref()?;
    Some(MethodNames::new(
        Ident::new("delete", Span::call_site()),
        entity.context_type.is_some(),
        (entity.before_delete.is_some() || entity.after_delete.is_some())
            .then(|| "no_trigger_delete".into()),
    ))
}

pub fn patch(entity: &Entity) -> Option<MethodNames> {
    entity.patch.as_ref()?;
    Some(MethodNames::new(
        Ident::new("patch", Span::call_site()),
        entity.context_type.is_some(),
        (entity.before_patch.is_some() || entity.after_patch.is_some())
            .then(|| "no_trigger_patch".into()),
    ))
}

pub fn mutation_names(entity: &Entity) -> impl Iterator<Item = Ident> + '_ {
    [update(entity), delete(entity), patch(entity)]
        .into_iter()
        .flatten()
        .chain(
            entity
                .fields
                .iter()
                .filter_map(|field| setter(entity, field)),
        )
        .flat_map(MethodNames::into_names)
        .chain(
            entity
                .fields
                .iter()
                .filter_map(|field| field.delete.clone()),
        )
}

#[cfg(test)]
mod tests;
