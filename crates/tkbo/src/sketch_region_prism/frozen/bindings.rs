// SPDX-License-Identifier: GPL-2.0-only
//! Directed ancestry is supplied explicitly; traversal positions only locate a supplied binding.
use super::super::{PrismBodyContext, PrismDiagnostic, PrismReceipt, contexts::body_contexts};
use super::{identities::*, types::*};
use std::collections::BTreeSet;

pub(super) struct BoundProjection<'a> {
    pub receipt: &'a PrismReceipt,
    pub diagnostic: &'a PrismDiagnostic,
    pub bindings: &'a PrismPublicationBindings,
    pub contexts: Vec<PrismBodyContext>,
    pub names: Vec<FrozenPrismKey>,
    pub use_names: Vec<FrozenUseKey>,
}
pub(super) fn bind_projection<'a>(
    receipt: &'a PrismReceipt,
    bindings: &'a PrismPublicationBindings,
) -> Result<BoundProjection<'a>, PrismFreezeCause> {
    admit_identifiers(bindings)?;
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .ok_or(missing("diagnostic body"))?;
    let contexts =
        body_contexts(&diagnostic.body).map_err(|_| missing("acyclic complete body contexts"))?;
    let mut bound = BoundProjection {
        receipt,
        diagnostic,
        bindings,
        contexts,
        names: Vec::new(),
        use_names: Vec::new(),
    };
    bound.names = bound
        .contexts
        .iter()
        .map(|c| name_context(&bound, c))
        .collect::<Result<_, _>>()?;
    check_definition_collisions(&bound)?;
    bound.use_names = bind_uses(&bound)?;
    Ok(bound)
}
fn admit_identifiers(bindings: &PrismPublicationBindings) -> Result<(), PrismFreezeCause> {
    let mut ids = vec![bindings.operation, bindings.selected_region];
    ids.extend(
        bindings
            .originals
            .iter()
            .flat_map(|b| [b.member, b.directed_use]),
    );
    ids.extend(bindings.sources.iter().map(|b| b.member));
    ids.extend(bindings.directed.iter().map(|b| b.authored_use));
    if ids.iter().any(|id| id.0 == 0) {
        return Err(PrismFreezeCause::Identity {
            required: "nonzero caller-authored identities",
            actual: 0,
        });
    }
    if bindings.sources.is_empty() || bindings.directed.is_empty() || bindings.originals.is_empty()
    {
        return Err(missing("authored member and directed-use bindings"));
    }
    Ok(())
}
fn check_definition_collisions(bound: &BoundProjection<'_>) -> Result<(), PrismFreezeCause> {
    for (index, key) in bound.names.iter().enumerate() {
        let first = &bound.contexts[index].occurrence;
        for (other, name) in bound.names.iter().enumerate().take(index) {
            if key == name && !first.is_same(&bound.contexts[other].occurrence) {
                return Err(PrismFreezeCause::Identity {
                    required: "unique definition for authored key and sweep role",
                    actual: 2,
                });
            }
        }
    }
    Ok(())
}
fn bind_uses(bound: &BoundProjection<'_>) -> Result<Vec<FrozenUseKey>, PrismFreezeCause> {
    let mut names = Vec::new();
    let mut consumed = BTreeSet::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        let binding = direct_binding(bound, context)?;
        consumed.insert(binding);
        names.push(ancestry_key(
            bound,
            &names,
            index,
            bound.bindings.directed[binding].authored_use,
        )?);
    }
    if consumed.len() != bound.bindings.directed.len()
        || names.iter().collect::<BTreeSet<_>>().len() != names.len()
    {
        return Err(PrismFreezeCause::Identity {
            required: "exactly one bound key per directed occurrence, no extras/collisions",
            actual: consumed.len(),
        });
    }
    Ok(names)
}
fn direct_binding(
    bound: &BoundProjection<'_>,
    context: &PrismBodyContext,
) -> Result<usize, PrismFreezeCause> {
    let found: Vec<_> = bound
        .bindings
        .directed
        .iter()
        .enumerate()
        .filter(|(_, b)| same_context(&b.actual, context))
        .map(|(i, _)| i)
        .collect();
    match found.as_slice() {
        [index] => Ok(*index),
        _ => Err(PrismFreezeCause::Identity {
            required: "one authored directed-use binding for actual context",
            actual: found.len(),
        }),
    }
}
fn ancestry_key(
    bound: &BoundProjection<'_>,
    names: &[FrozenUseKey],
    index: usize,
    authored: AuthoredPrismId,
) -> Result<FrozenUseKey, PrismFreezeCause> {
    let context = &bound.contexts[index];
    let mut key = Vec::new();
    if !context.child_slots.is_empty() {
        let path = &context.child_slots[..context.child_slots.len() - 1];
        let parent = bound.contexts[..index]
            .iter()
            .position(|c| c.child_slots == path)
            .ok_or(missing("bound ancestor"))?;
        key = names[parent].0.clone();
    }
    key.push((bound.names[index], authored));
    Ok(FrozenUseKey(key))
}
pub(super) fn missing(required: &'static str) -> PrismFreezeCause {
    PrismFreezeCause::Missing {
        required,
        actual: 0,
    }
}
