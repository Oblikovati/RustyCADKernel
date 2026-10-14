// SPDX-License-Identifier: GPL-2.0-only
//! Resolve only through the one certified map; output order never creates names.
use super::{error::*, *};
use crate::sketch_region_prism::{
    frozen::*,
    owned_request::names::{body_map::ResolvedPrismNames, *},
};

pub(super) fn member(
    names: &ResolvedPrismNames,
    key: &FrozenPrismKey,
) -> BodyCheck<SemanticMemberKey> {
    names
        .member(key)
        .cloned()
        .map_err(|cause| failure(names, PrismBodyProjectionCause::SemanticMap(cause)))
}
pub(super) fn occurrence(
    names: &ResolvedPrismNames,
    key: &FrozenUseKey,
) -> BodyCheck<SemanticUseKey> {
    names
        .occurrence(key)
        .cloned()
        .map_err(|cause| failure(names, PrismBodyProjectionCause::SemanticMap(cause)))
}
pub(super) fn occurrences(
    names: &ResolvedPrismNames,
    keys: &[FrozenUseKey],
) -> BodyCheck<Vec<SemanticUseKey>> {
    keys.iter().map(|key| occurrence(names, key)).collect()
}
pub(super) fn output_slot(
    names: &ResolvedPrismNames,
    key: &FrozenPrismKey,
) -> BodyCheck<Option<SemanticNameSlot>> {
    let rows: Vec<_> = names
        .member_bindings()
        .iter()
        .filter(|row| &row.frozen == key)
        .collect();
    count(
        names,
        "one semantic output association per member",
        rows.len(),
        1,
    )?;
    Ok(rows[0].output_slot)
}
pub(super) fn at_member<T>(
    names: &ResolvedPrismNames,
    key: &FrozenPrismKey,
    result: BodyCheck<T>,
) -> BodyCheck<T> {
    result.map_err(|mut error| {
        error.entity = names
            .member(key)
            .ok()
            .cloned()
            .map(PrismBodyProjectionEntity::Member);
        error
    })
}
pub(super) fn at_use<T>(
    names: &ResolvedPrismNames,
    key: &FrozenUseKey,
    result: BodyCheck<T>,
) -> BodyCheck<T> {
    result.map_err(|mut error| {
        error.entity = names
            .occurrence(key)
            .ok()
            .cloned()
            .map(PrismBodyProjectionEntity::Occurrence);
        error
    })
}

pub(super) fn root_occurrence(
    names: &ResolvedPrismNames,
    uses: &[FrozenPrismUse],
) -> BodyCheck<SemanticUseKey> {
    let roots: Vec<_> = uses.iter().filter(|row| row.key.0.len() == 1).collect();
    count(names, "one complete solid root occurrence", roots.len(), 1)?;
    if roots[0].key.0[0].0.kind != 2 {
        return Err(failure(
            names,
            PrismBodyProjectionCause::OperandMismatch {
                required: "root member kind Solid (2)",
            },
        ));
    }
    occurrence(names, &roots[0].key)
}
