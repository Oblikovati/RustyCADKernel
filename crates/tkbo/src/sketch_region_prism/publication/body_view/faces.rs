// SPDX-License-Identifier: GPL-2.0-only
//! Read the actual trimmed surface and preserve all directed seam uses; proof sources remain private.
use super::{error::*, lookup, rows::*};
use crate::sketch_region_prism::{
    frozen::{FrozenPrismFace, FrozenPrismTrim},
    owned_request::names::body_map::ResolvedPrismNames,
};

pub(super) fn project(
    row: &FrozenPrismFace,
    names: &ResolvedPrismNames,
) -> BodyCheck<PrismBodyFace> {
    let trims = row
        .trims
        .iter()
        .map(|trim| project_trim(trim, names))
        .collect::<BodyCheck<_>>()?;
    Ok(PrismBodyFace {
        key: lookup::member(names, &row.key)?,
        surface: row.surface.clone(),
        trims,
    })
}
pub(super) fn project_trim(
    row: &FrozenPrismTrim,
    names: &ResolvedPrismNames,
) -> BodyCheck<PrismBodyTrim> {
    Ok(PrismBodyTrim {
        occurrence: lookup::occurrence(names, &row.occurrence)?,
        role: row.role,
        pcurve: row.pcurve.clone(),
        native_interval: row.interval,
        space: row.space.clone(),
    })
}
