// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::OriginalMemberIdentity;

pub(super) fn original_multiset(
    bindings: &PrismPublicationBindings,
    originals: &[OriginalMemberIdentity],
) -> CaptureResult<Vec<FrozenOriginalKey>> {
    let mut keys = originals
        .iter()
        .map(|original| original_key(bindings, original))
        .collect::<CaptureResult<Vec<_>>>()?;
    keys.sort();
    Ok(keys)
}
/// Retain every current classed parent; `link_frozen_parent_dispositions(bound, projection)`.
pub(in crate::sketch_region_prism::frozen) fn current_dispositions(
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> CaptureResult<Vec<JoinedParentDispositions>> {
    let mut result = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        let rows: Vec<_> = bound
            .diagnostic
            .relations
            .iter()
            .filter(|row| row.output.is_equal(&context.occurrence))
            .collect();
        let row = *only_bound(&rows, "one actual complete history row for joined proof")?;
        let entry = JoinedParentDispositions {
            occurrence: bound.use_names[index].clone(),
            preserved: original_multiset(bound.bindings, &row.preserved)?,
            modified: original_multiset(bound.bindings, &row.modified)?,
            generated: original_multiset(bound.bindings, &row.generated)?,
        };
        check_projected_set(&entry, projection)?;
        result.push(entry);
    }
    result.sort_by(|a, b| a.occurrence.cmp(&b.occurrence));
    Ok(result)
}
fn check_projected_set(
    joined: &JoinedParentDispositions,
    projection: &FrozenPrismProjection,
) -> CaptureResult<()> {
    let rows: Vec<_> = projection
        .history()
        .derived
        .iter()
        .filter(|row| row.output == joined.occurrence)
        .collect();
    let frozen = *only_bound(&rows, "one frozen history row for joined proof")?;
    for (full, set) in [
        (&joined.preserved, &frozen.preserved),
        (&joined.modified, &frozen.modified),
        (&joined.generated, &frozen.generated),
    ] {
        let mut unique = full.clone();
        unique.dedup();
        if &unique != set {
            return Err(identity(
                "same complete parent dispositions before frozen set projection",
                0,
            ));
        }
    }
    Ok(())
}
/// Encode existing remaining history without a completeness claim; `append_frozen_remaining_history(bound, bytes)`.
pub(in crate::sketch_region_prism::frozen) fn append_remaining_history(
    bound: &BoundProjection<'_>,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    let copied = bound
        .receipt
        .copied
        .as_ref()
        .ok_or(missing("joined copied history"))?;
    for (stage, relations) in [
        ("copied", &copied.relations),
        ("retired", &bound.diagnostic.native_retired),
    ] {
        let mut rows = Vec::new();
        for relation in relations {
            let mut row = Vec::new();
            replay::text(
                &format!(
                    "{stage}/{:?}",
                    source_name(bound.bindings, &relation.output)?
                ),
                &mut row,
            );
            super::replay_sources::occurrence(
                &crate::sketch_region::PartitionOccurrenceIdentity::capture(&relation.output),
                &mut row,
            )?;
            append_relation(bound.bindings, relation, &mut row)?;
            rows.push(row);
        }
        super::replay_sources::sorted_blocks(rows, bytes);
    }
    append_rejected(bound, bytes)?;
    replay::text(
        &format!(
            "deleted={:?}",
            original_multiset(bound.bindings, &bound.receipt.input.input_deleted)?
        ),
        bytes,
    );
    Ok(())
}
fn append_rejected(bound: &BoundProjection<'_>, bytes: &mut Vec<u8>) -> CaptureResult<()> {
    let mut candidates = Vec::new();
    for rejected in &bound.receipt.input.rejected {
        let mut rows = Vec::new();
        for relation in rejected {
            let mut row = Vec::new();
            replay::text(
                &format!(
                    "{:?}",
                    relation
                        .output
                        .shape_type()
                        .map_err(PrismFreezeCause::Native)?
                ),
                &mut row,
            );
            append_relation(bound.bindings, relation, &mut row)?;
            rows.push(row);
        }
        let mut candidate = Vec::new();
        super::replay_sources::sorted_blocks(rows, &mut candidate);
        candidates.push(candidate);
    }
    super::replay_sources::sorted_blocks(candidates, bytes);
    Ok(())
}
fn append_relation(
    bindings: &PrismPublicationBindings,
    relation: &crate::sketch_region::ArrangementRelation,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    for values in [&relation.preserved, &relation.modified, &relation.generated] {
        replay::text(
            &format!("{:?}", original_multiset(bindings, values)?),
            bytes,
        );
    }
    Ok(())
}
