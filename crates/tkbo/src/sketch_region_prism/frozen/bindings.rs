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
) -> CaptureResult<BoundProjection<'a>> {
    super::domain::check_count(bindings.directed.len())?;
    admit_identifiers(bindings)?;
    let mut bound = unbound_projection(receipt, bindings)?;
    bound.names = bound
        .contexts
        .iter()
        .map(|c| name_context(&bound, c))
        .collect::<Result<_, _>>()?;
    check_bound_definitions(&bound)?;
    bound.use_names = bind_uses(&bound)?;
    Ok(bound)
}
fn admit_identifiers(bindings: &PrismPublicationBindings) -> CaptureResult<()> {
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
        return Err(identity("nonzero caller-authored identities", 0));
    }
    if bindings.sources.is_empty() || bindings.directed.is_empty() || bindings.originals.is_empty()
    {
        return Err(missing("authored member and directed-use bindings"));
    }
    Ok(())
}
fn check_definition_collisions(bound: &BoundProjection<'_>) -> CaptureResult<()> {
    for (index, key) in bound.names.iter().enumerate() {
        let first = &bound.contexts[index].occurrence;
        for (other, name) in bound.names.iter().enumerate().take(index) {
            if key == name && !first.is_same(&bound.contexts[other].occurrence) {
                return Err(identity(
                    "unique definition for authored key and sweep role",
                    2,
                ));
            }
        }
    }
    Ok(())
}
fn bind_uses(bound: &BoundProjection<'_>) -> CaptureResult<Vec<FrozenUseKey>> {
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
    check_use_coverage(bound, &names, consumed.len())?;
    Ok(names)
}
fn direct_binding(bound: &BoundProjection<'_>, context: &PrismBodyContext) -> CaptureResult<usize> {
    let found: Vec<_> = bound
        .bindings
        .directed
        .iter()
        .enumerate()
        .filter(|(_, b)| same_context(&b.actual, context))
        .map(|(i, _)| i)
        .collect();
    only_bound(
        &found,
        "one authored directed-use binding for actual context",
    )
    .copied()
}
fn ancestry_key(
    bound: &BoundProjection<'_>,
    names: &[FrozenUseKey],
    index: usize,
    authored: AuthoredPrismId,
) -> CaptureResult<FrozenUseKey> {
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

fn check_orientations(bound: &BoundProjection<'_>) -> CaptureResult<()> {
    use rustycad_tkg3d::top_abs::Orientation::{Forward, Reversed};
    for context in &bound.contexts {
        let actual = context.occurrence.orientation();
        if !matches!(actual, Forward | Reversed) {
            return Err(PrismFreezeCause::UnsupportedOrientation {
                actual,
                expected: "Forward or Reversed directed polynomial prism occurrence",
            });
        }
    }
    Ok(())
}

pub(super) fn identity(required: &'static str, actual: usize) -> PrismFreezeCause {
    PrismFreezeCause::Identity { required, actual }
}

fn unbound_projection<'a>(
    receipt: &'a PrismReceipt,
    bindings: &'a PrismPublicationBindings,
) -> CaptureResult<BoundProjection<'a>> {
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .ok_or(missing("diagnostic body"))?;
    let contexts = body_contexts(&diagnostic.body).map_err(freeze_context_cause)?;
    let bound = BoundProjection {
        receipt,
        diagnostic,
        bindings,
        contexts,
        names: Vec::new(),
        use_names: Vec::new(),
    };
    Ok(bound)
}

fn check_bound_definitions(bound: &BoundProjection<'_>) -> CaptureResult<()> {
    check_orientations(bound)?;
    super::source_bindings::check_sources(bound)?;
    super::source_bindings::check_originals(bound)?;
    check_definition_collisions(bound)?;
    Ok(())
}

fn check_use_coverage(
    bound: &BoundProjection<'_>,
    names: &[FrozenUseKey],
    consumed: usize,
) -> CaptureResult<()> {
    if consumed != bound.bindings.directed.len()
        || names.iter().collect::<BTreeSet<_>>().len() != names.len()
    {
        return Err(identity(
            "exactly one bound key per directed occurrence, no extras/collisions",
            consumed,
        ));
    }
    Ok(())
}

pub(super) fn only_bound<'a, T>(values: &'a [T], required: &'static str) -> CaptureResult<&'a T> {
    match values {
        [value] => Ok(value),
        _ => Err(identity(required, values.len())),
    }
}

pub(super) fn freeze_context_cause(cause: super::super::PrismCause) -> PrismFreezeCause {
    use super::super::PrismCause;
    match cause {
        PrismCause::Mapped { failure, .. } => PrismFreezeCause::Native(failure),
        PrismCause::CyclicBodyContext { .. } => super::admission::unsupported(
            "cyclic actual body context requires unbound Shape diagnostic",
        ),
        _ => super::admission::unsupported("unprojectable actual body-context cause"),
    }
}
