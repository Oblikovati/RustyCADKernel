// SPDX-License-Identifier: GPL-2.0-only
//! Reject unused/colliding authored members; physical definitions and directed uses are distinct.
use super::{
    bindings::{BoundProjection, identity, missing},
    identities::{same_definition, source_name},
    types::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;

pub(super) fn check_sources(bound: &BoundProjection<'_>) -> CaptureResult<()> {
    let copied = bound
        .receipt
        .copied
        .as_ref()
        .ok_or(missing("copied source binding domain"))?;
    if source_name(bound.bindings, &copied.base)? != bound.bindings.selected_region {
        return Err(identity("selected-region ID bound to copied base", 0));
    }
    check_used_sources(&bound.bindings.sources, &copied.relations)?;
    check_source_collisions(&bound.bindings.sources)
}
fn check_source_collisions(bindings: &[PrismSourceBinding]) -> CaptureResult<()> {
    for (index, binding) in bindings.iter().enumerate() {
        for other in &bindings[..index] {
            if (binding.member == other.member) != same_definition(&binding.actual, &other.actual) {
                return Err(identity(
                    "one authored source ID per physical copied member",
                    2,
                ));
            }
        }
    }
    Ok(())
}
pub(super) fn check_originals(bound: &BoundProjection<'_>) -> CaptureResult<()> {
    let mut parents = bound.receipt.input.input_deleted.clone();
    for row in &bound.diagnostic.relations {
        parents.extend(super::super::history::all_parents(row));
    }
    for row in bound.receipt.input.rejected.iter().flatten() {
        parents.extend(super::super::history::all_parents(row));
    }
    for binding in &bound.bindings.originals {
        if !parents.contains(&binding.actual) {
            return Err(identity(
                "every authored original binding used by operation history",
                0,
            ));
        }
    }
    Ok(())
}

fn check_used_sources(
    bindings: &[PrismSourceBinding],
    relations: &[crate::sketch_region::ArrangementRelation],
) -> CaptureResult<()> {
    for binding in bindings {
        if !relations.iter().any(|row| {
            same_definition(
                &binding.actual,
                &PartitionOccurrenceIdentity::capture(&row.output),
            )
        }) {
            return Err(identity(
                "every authored source binding used by copied selection",
                0,
            ));
        }
    }
    Ok(())
}
