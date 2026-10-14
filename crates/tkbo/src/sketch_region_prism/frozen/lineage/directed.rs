// SPDX-License-Identifier: GPL-2.0-only
//! Actual mapped ancestry proves correspondence; supplied use names only label proven contexts.
use super::*;
use std::collections::BTreeSet;

pub(super) struct BoundCopyContexts {
    pub selected: Vec<PrismBodyContext>,
    pub copied: Vec<PrismBodyContext>,
    pub selected_keys: Vec<CopyUseKey>,
    pub copied_keys: Vec<CopyUseKey>,
}
pub(super) fn bind_contexts(inputs: &LineageInputs<'_>) -> LineageCheck<BoundCopyContexts> {
    let selected = names::contexts(inputs, CopyLineageStage::Selected)?;
    let copied = names::contexts(inputs, CopyLineageStage::Copied)?;
    let selected_keys = context_keys(
        inputs,
        &selected,
        CopyLineageStage::Selected,
        &inputs.authored.selected_uses,
    )?;
    let copied_keys = context_keys(
        inputs,
        &copied,
        CopyLineageStage::Copied,
        &inputs.authored.copied_uses,
    )?;
    Ok(BoundCopyContexts {
        selected,
        copied,
        selected_keys,
        copied_keys,
    })
}
fn context_keys(
    inputs: &LineageInputs<'_>,
    contexts: &[PrismBodyContext],
    stage: CopyLineageStage,
    bindings: &[PrismDirectedBinding],
) -> LineageCheck<Vec<CopyUseKey>> {
    super::super::domain::check_count(contexts.len())
        .map_err(|e| inputs.frozen(CopyLineagePhase::Directed, e))?;
    count(
        inputs,
        "one authored binding for each actual directed context",
        bindings.len(),
        contexts.len(),
    )?;
    let mut keys = Vec::new();
    for context in contexts {
        let key = key_for_context(inputs, contexts, bindings, context, stage)?;
        keys.push(key);
    }
    count(
        inputs,
        "unique authored full directed-use keys",
        keys.iter().collect::<BTreeSet<_>>().len(),
        keys.len(),
    )?;
    Ok(keys)
}
fn key_for_context(
    inputs: &LineageInputs<'_>,
    contexts: &[PrismBodyContext],
    bindings: &[PrismDirectedBinding],
    context: &PrismBodyContext,
    stage: CopyLineageStage,
) -> LineageCheck<CopyUseKey> {
    let mut key = Vec::new();
    for length in 0..=context.child_slots.len() {
        let parent = ancestor_at_path(inputs, contexts, &context.child_slots[..length])?;
        let authored = authored_use(inputs, bindings, parent)?;
        key.push((names::member(inputs, &parent.occurrence, stage)?, authored));
    }
    Ok(CopyUseKey(key))
}
fn ancestor_at_path<'a>(
    inputs: &LineageInputs<'_>,
    contexts: &'a [PrismBodyContext],
    path: &[usize],
) -> LineageCheck<&'a PrismBodyContext> {
    let parents: Vec<_> = contexts.iter().filter(|c| c.child_slots == path).collect();
    count(
        inputs,
        "one actual ancestor at directed path",
        parents.len(),
        1,
    )?;
    Ok(parents[0])
}
fn authored_use(
    inputs: &LineageInputs<'_>,
    bindings: &[PrismDirectedBinding],
    parent: &PrismBodyContext,
) -> LineageCheck<AuthoredPrismId> {
    let found: Vec<_> = bindings
        .iter()
        .filter(|b| super::super::identities::same_context(&b.actual, parent))
        .collect();
    count(
        inputs,
        "one authored binding for actual directed ancestry",
        found.len(),
        1,
    )?;
    count(
        inputs,
        "nonzero authored use",
        usize::from(found[0].authored_use.0 != 0),
        1,
    )?;
    Ok(found[0].authored_use)
}

pub(super) fn pair_contexts(
    inputs: &LineageInputs<'_>,
    bound: &BoundCopyContexts,
) -> LineageCheck<Vec<CopyDirectedObservation>> {
    let mut result = Vec::new();
    let mut consumed = BTreeSet::new();
    for (index, selected) in bound.selected.iter().enumerate() {
        let matched = matched_context(inputs, bound, selected)?;
        consumed.insert(matched);
        result.push(CopyDirectedObservation {
            selected: bound.selected_keys[index].clone(),
            copied: bound.copied_keys[matched].clone(),
        });
    }
    count(
        inputs,
        "one-to-one directed correspondence",
        consumed.len(),
        bound.copied.len(),
    )?;
    result.sort();
    Ok(result)
}
fn matched_context(
    inputs: &LineageInputs<'_>,
    bound: &BoundCopyContexts,
    selected: &PrismBodyContext,
) -> LineageCheck<usize> {
    let matches: Vec<_> = bound
        .copied
        .iter()
        .enumerate()
        .filter(|(_, copied)| matches_context(inputs, selected, copied))
        .map(|(i, _)| i)
        .collect();
    let key = names::member(inputs, &selected.occurrence, CopyLineageStage::Selected)?;
    count(
        inputs,
        "unique copied directed context from mapped ancestors",
        matches.len(),
        1,
    )
    .map_err(|e| names::attributed(e, key))?;
    Ok(matches[0])
}
fn matches_context(
    inputs: &LineageInputs<'_>,
    selected: &PrismBodyContext,
    copied: &PrismBodyContext,
) -> bool {
    let observations = inputs
        .receipt
        .copied
        .as_ref()
        .map(|copy| copy.observations.as_slice())
        .unwrap_or_default();
    matches_selected_copy_context(observations, selected, copied)
}
/// Reuse retained physical images and directed ancestry before naming.
/// Example: `matches_selected_copy_context(observations, selected, copied)`.
/// The caller still proves unique complete correspondence and all member-history obligations.
pub(in crate::sketch_region_prism) fn matches_selected_copy_context(
    observations: &[PrismCopyObservation],
    selected: &PrismBodyContext,
    copied: &PrismBodyContext,
) -> bool {
    if selected.ancestors.len() != copied.ancestors.len() {
        return false;
    }
    selected
        .ancestors
        .iter()
        .chain([&selected.occurrence])
        .zip(copied.ancestors.iter().chain([&copied.occurrence]))
        .all(|(first, second)| mapped_member(observations, first, second))
}
fn mapped_member(observations: &[PrismCopyObservation], selected: &Shape, copied: &Shape) -> bool {
    if selected.location() != copied.location() || selected.orientation() != copied.orientation() {
        return false;
    }
    observations
        .iter()
        .any(|o| o.selected.output.is_same(selected) && o.copied_image.is_same(copied))
}
pub(super) fn owned_contexts(
    inputs: &LineageInputs<'_>,
    bound: &BoundCopyContexts,
) -> LineageCheck<Vec<CopyContextObservation>> {
    let mut result = Vec::new();
    for (contexts, keys, stage) in [
        (
            &bound.selected,
            &bound.selected_keys,
            CopyLineageStage::Selected,
        ),
        (&bound.copied, &bound.copied_keys, CopyLineageStage::Copied),
    ] {
        for (index, context) in contexts.iter().enumerate() {
            result.push(CopyContextObservation {
                key: keys[index].clone(),
                occurrence: names::occurrence(inputs, &context.occurrence, stage)?,
                children: child_keys(contexts, keys, &context.child_slots),
            });
        }
    }
    result.sort();
    Ok(result)
}
fn child_keys(
    contexts: &[PrismBodyContext],
    keys: &[CopyUseKey],
    path: &[usize],
) -> Vec<CopyUseKey> {
    contexts
        .iter()
        .enumerate()
        .filter(|(_, c)| c.child_slots.len() == path.len() + 1 && c.child_slots.starts_with(path))
        .map(|(index, _)| keys[index].clone())
        .collect()
}
fn count(
    inputs: &LineageInputs<'_>,
    required: &'static str,
    actual: usize,
    expected: usize,
) -> LineageCheck<()> {
    inputs.require(CopyLineagePhase::Directed, required, actual, expected)
}
