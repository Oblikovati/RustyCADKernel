// SPDX-License-Identifier: GPL-2.0-only
//! Keep every original parent and mapped class; no empty-parent fallback authorizes capture.
use super::{
    bindings::{BoundProjection, missing},
    identities::original_key,
    types::*,
};
use crate::sketch_region::OriginalMemberIdentity;

pub(super) fn capture_history(
    bound: &BoundProjection<'_>,
) -> Result<FrozenPrismHistory, PrismFreezeCause> {
    if !bound.receipt.input.unrepresented.is_empty() || !bound.diagnostic.native_retired.is_empty()
    {
        return Err(PrismFreezeCause::UnsupportedDiagnostic {
            stage: None,
            actual: "unrepresented input or uncomposed intermediate retirement",
        });
    }
    let mut derived = Vec::new();
    for index in 0..bound.contexts.len() {
        derived.push(capture_relation(bound, index)?);
    }
    derived.sort_by(|a, b| a.output.cmp(&b.output));
    let deleted = original_keys(bound.bindings, &bound.receipt.input.input_deleted)?;
    Ok(FrozenPrismHistory { derived, deleted })
}
fn capture_relation(
    bound: &BoundProjection<'_>,
    index: usize,
) -> Result<FrozenDerivedRelation, PrismFreezeCause> {
    let actual = &bound.contexts[index].occurrence;
    let rows: Vec<_> = bound
        .diagnostic
        .relations
        .iter()
        .filter(|r| r.output.is_equal(actual))
        .collect();
    let [row] = rows.as_slice() else {
        return Err(missing("one complete actual occurrence history row"));
    };
    let result = FrozenDerivedRelation {
        output: bound.use_names[index].clone(),
        preserved: original_keys(bound.bindings, &row.preserved)?,
        modified: original_keys(bound.bindings, &row.modified)?,
        generated: original_keys(bound.bindings, &row.generated)?,
    };
    if result.preserved.is_empty() && result.modified.is_empty() && result.generated.is_empty() {
        return Err(missing("all original occurrence parents"));
    }
    Ok(result)
}
pub(super) fn original_keys(
    bindings: &PrismPublicationBindings,
    originals: &[OriginalMemberIdentity],
) -> Result<Vec<FrozenOriginalKey>, PrismFreezeCause> {
    let mut result = originals
        .iter()
        .map(|o| original_key(bindings, o))
        .collect::<Result<Vec<_>, _>>()?;
    result.sort();
    result.dedup();
    Ok(result)
}
pub(super) fn occurrence_keys(
    bound: &BoundProjection<'_>,
    shape: &rustycad_tkbrep::topods::Shape,
) -> Result<Vec<FrozenUseKey>, PrismFreezeCause> {
    let keys: Vec<_> = bound
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, c)| c.occurrence.is_equal(shape))
        .map(|(i, _)| bound.use_names[i].clone())
        .collect();
    if keys.is_empty() {
        return Err(missing("actual attributed diagnostic occurrence"));
    }
    Ok(keys)
}
