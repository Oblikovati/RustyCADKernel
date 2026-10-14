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
    if !bound.receipt.input.unrepresented.is_empty() {
        return Err(PrismFreezeCause::UnsupportedDiagnostic {
            stage: None,
            actual: "unrepresented input history",
        });
    }
    let mut derived = Vec::new();
    for index in 0..bound.contexts.len() {
        derived.push(capture_relation(bound, index)?);
    }
    derived.sort_by(|a, b| a.output.cmp(&b.output));
    let deleted = original_keys(bound.bindings, &bound.receipt.input.input_deleted)?;
    Ok(FrozenPrismHistory {
        originals: capture_originals(bound)?,
        rejected_unbound: capture_rejected(bound)?,
        retired_intermediate_unbound: capture_retired(bound)?,
        derived,
        deleted,
    })
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

pub(super) fn freeze_placement(
    location: &rustycad_tkmath::toploc::Location,
) -> Result<rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3>, PrismFreezeCause>
{
    crate::sketch_region_geometry_capture::captured_placement(location.transformation())
        .map_err(|cause| PrismFreezeCause::Surface(cause.into()))
}
fn capture_originals(
    bound: &BoundProjection<'_>,
) -> Result<Vec<FrozenOriginalOccurrence>, PrismFreezeCause> {
    let mut result = Vec::new();
    for binding in &bound.bindings.originals {
        result.push(FrozenOriginalOccurrence {
            key: original_key(bound.bindings, &binding.actual)?,
            orientation: binding.actual.orientation,
            placement: freeze_placement(&binding.actual.location)?,
        });
    }
    result.sort_by(|a, b| a.key.cmp(&b.key));
    if result.windows(2).any(|pair| pair[0].key == pair[1].key) {
        return Err(missing(
            "one authored original key per actual original occurrence",
        ));
    }
    Ok(result)
}

fn capture_rejected(
    bound: &BoundProjection<'_>,
) -> Result<Vec<Vec<FrozenRejectedRelation>>, PrismFreezeCause> {
    let mut result = Vec::new();
    for candidate in &bound.receipt.input.rejected {
        let mut relations = candidate
            .iter()
            .map(|row| rejected_relation(bound, row))
            .collect::<Result<Vec<_>, _>>()?;
        relations.sort();
        result.push(relations);
    }
    result.sort();
    Ok(result)
}
fn rejected_relation(
    bound: &BoundProjection<'_>,
    row: &crate::sketch_region::ArrangementRelation,
) -> Result<FrozenRejectedRelation, PrismFreezeCause> {
    let parents = [
        original_keys(bound.bindings, &row.preserved)?,
        original_keys(bound.bindings, &row.modified)?,
        original_keys(bound.bindings, &row.generated)?,
    ];
    if parents.iter().all(Vec::is_empty) {
        return Err(missing(
            "all original parents for rejected selection relation",
        ));
    }
    Ok(FrozenRejectedRelation {
        output_kind: row.output.shape_type().map_err(PrismFreezeCause::Native)? as u8,
        parents,
    })
}

fn capture_retired(
    bound: &BoundProjection<'_>,
) -> Result<Vec<FrozenRetiredIntermediate>, PrismFreezeCause> {
    let mut result = Vec::new();
    for row in &bound.diagnostic.native_retired {
        result.push(FrozenRetiredIntermediate {
            source: super::identities::source_name(bound.bindings, &row.output)?,
            relation: rejected_relation(bound, row)?,
            orientation: row.output.orientation(),
            placement: freeze_placement(row.output.location())?,
        });
    }
    result.sort_by_key(|row| (row.source, row.orientation as u8));
    Ok(result)
}
