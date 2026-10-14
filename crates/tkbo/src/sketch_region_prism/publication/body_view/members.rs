// SPDX-License-Identifier: GPL-2.0-only
use super::{error::*, lookup, rows::*};
use crate::sketch_region_prism::{
    exact_tolerance::JoinedToleranceMetadata, frozen::FrozenPrismNode,
    owned_request::names::body_map::ResolvedPrismNames,
    tolerance_inspection::StoredToleranceObservation,
};

pub(super) fn project(
    node: &FrozenPrismNode,
    names: &ResolvedPrismNames,
    metadata: &[JoinedToleranceMetadata],
) -> BodyCheck<PrismBodyMember> {
    let tolerance = metadata_for(node, names, metadata);
    Ok(PrismBodyMember {
        key: lookup::member(names, &node.key)?,
        output_slot: lookup::output_slot(names, &node.key)?,
        rounded_world_point: node.point,
        curve: node.curve.clone(),
        tolerance: lookup::at_member(names, &node.key, tolerance)?,
    })
}
fn metadata_for(
    node: &FrozenPrismNode,
    names: &ResolvedPrismNames,
    metadata: &[JoinedToleranceMetadata],
) -> BodyCheck<Option<StoredToleranceObservation>> {
    let rows: Vec<_> = metadata
        .iter()
        .filter(|row| row.entity == node.key)
        .collect();
    let expected = usize::from(matches!(node.key.kind, 4 | 6 | 7));
    metadata_count(names, rows.len(), expected)?;
    if expected == 0 {
        return Ok(None);
    }
    check_scalars(node, names, rows[0].observation)?;
    Ok(Some(rows[0].observation))
}
fn check_scalars(
    node: &FrozenPrismNode,
    names: &ResolvedPrismNames,
    actual: StoredToleranceObservation,
) -> BodyCheck<()> {
    if node.raw_tolerance.map(f64::to_bits) == Some(actual.raw.to_bits())
        && node.effective_tolerance.map(f64::to_bits) == Some(actual.effective.to_bits())
    {
        return Ok(());
    }
    Err(failure(
        names,
        PrismBodyProjectionCause::OperandMismatch {
            required: "raw/effective tolerance bits matching accepted metadata observation",
        },
    ))
}

fn metadata_count(names: &ResolvedPrismNames, actual: usize, expected: usize) -> BodyCheck<()> {
    count(
        names,
        "one tolerance observation per face/edge/vertex; none for containers",
        actual,
        expected,
    )
}
