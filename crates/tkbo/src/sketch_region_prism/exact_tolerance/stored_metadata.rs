// SPDX-License-Identifier: GPL-2.0-only
//! Read owned raw scalars once per definition, retaining every directed use.
use super::{endpoint_uses::definition, tolerance_types::*};
use crate::sketch_region_prism::{frozen::*, tolerance_inspection::*};

pub(super) fn observe_node(
    projection: &FrozenPrismProjection,
    node: &FrozenPrismNode,
    request: PrismToleranceRequest,
) -> Result<JoinedToleranceMetadata, JoinedToleranceCause> {
    let (raw, effective) = raw_effective(node)?;
    let observation =
        request
            .observe(raw, effective)
            .map_err(|cause| JoinedToleranceCause::MetadataScalar {
                entity: node.key,
                cause,
            })?;
    Ok(JoinedToleranceMetadata {
        entity: node.key,
        contexts: metadata_contexts(projection, node.key),
        observation,
    })
}
fn raw_effective(node: &FrozenPrismNode) -> Result<(f64, f64), JoinedToleranceCause> {
    let raw = required(node.raw_tolerance, node.key, "raw stored entity tolerance")?;
    let effective = required(
        node.effective_tolerance,
        node.key,
        "effective compatibility accessor",
    )?;
    Ok((raw, effective))
}
fn metadata_contexts(
    projection: &FrozenPrismProjection,
    entity: FrozenPrismKey,
) -> Vec<FrozenUseKey> {
    projection
        .uses()
        .iter()
        .filter(|row| definition(&row.key) == Some(entity))
        .map(|row| row.key.clone())
        .collect()
}
fn required(
    value: Option<f64>,
    entity: FrozenPrismKey,
    quantity: &'static str,
) -> Result<f64, JoinedToleranceCause> {
    value.ok_or(JoinedToleranceCause::MissingMetadata { entity, quantity })
}
pub(super) fn refuse_caps(
    metadata: &[JoinedToleranceMetadata],
) -> Result<(), JoinedToleranceCause> {
    let entities: Vec<_> = metadata
        .iter()
        .filter(|row| row.observation.decision == StoredCapDecision::Refused)
        .map(|row| row.entity)
        .collect();
    if !entities.is_empty() {
        return Err(JoinedToleranceCause::RefusedCap { entities });
    }
    Ok(())
}
