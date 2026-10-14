// SPDX-License-Identifier: GPL-2.0-only
//! Reuse single-boundary pole/plane/extrusion admission per role; names must be distinct and nonzero.
use super::super::admission::{finite_plane, validate_poles};
use super::*;

/// Admission refusal; owned single-boundary causes keep their original typed operands.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum AnnularAdmissionCause {
    Owned(OwnedPrismAdmissionCause),
    Names { quantity: &'static str },
}
/// Admit one V1 annular request before any construction; `admit(&request)?`.
pub(super) fn admit(request: &OwnedAnnularPrismRequest) -> Result<(), AnnularAdmissionCause> {
    if request.version != 1 {
        let (actual, expected) = (request.version, 1);
        let cause = OwnedPrismAdmissionCause::UnsupportedVersion { actual, expected };
        return Err(AnnularAdmissionCause::Owned(cause));
    }
    for (boundary, _) in request.boundaries() {
        validate_poles(boundary).map_err(AnnularAdmissionCause::Owned)?;
    }
    finite_plane(request.declared_plane).map_err(AnnularAdmissionCause::Owned)?;
    admit_extrusion(request.extrusion)?;
    admit_names(request)
}
fn admit_extrusion(actual: [f64; 3]) -> Result<(), AnnularAdmissionCause> {
    if actual.iter().all(|x| x.is_finite()) && actual != [0.; 3] {
        return Ok(());
    }
    let cause = OwnedPrismAdmissionCause::InvalidExtrusion { actual };
    Err(AnnularAdmissionCause::Owned(cause))
}
fn admit_names(request: &OwnedAnnularPrismRequest) -> Result<(), AnnularAdmissionCause> {
    let names = request.regions.all();
    distinct("nonzero distinct region and wire names", &names)?;
    let sources = [request.outer.source.0, request.inner.source.0];
    distinct(
        "nonzero distinct outer and inner boundary sources",
        &sources,
    )?;
    distinct("nonzero operation identity", &[request.operation.0])
}
fn distinct(quantity: &'static str, values: &[u64]) -> Result<(), AnnularAdmissionCause> {
    let unique: std::collections::BTreeSet<_> = values.iter().filter(|v| **v != 0).collect();
    if unique.len() == values.len() {
        return Ok(());
    }
    Err(AnnularAdmissionCause::Names { quantity })
}
