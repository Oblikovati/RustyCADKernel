// SPDX-License-Identifier: GPL-2.0-only
//! Full authored observations join existing immutable rows; role alone never chooses a side.
use super::*;
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_prism::surface_charts::CapturedPrismSurface;
use crate::sketch_region_source_conformance::CapturedPlaneSupport;

pub(super) fn frozen_face<'a>(
    join: &'a AnnularFrozenDomain,
    named: &NamedAnnularFace,
) -> MeasureCheck<&'a FrozenPrismFace> {
    let found: Vec<_> = join
        .projection()
        .faces()
        .iter()
        .filter(|f| f.key == named.chart.definition)
        .collect();
    count("one complete named face definition", found.len(), 1)?;
    Ok(found[0])
}
pub(super) fn named_cap(
    join: &AnnularFrozenDomain,
    role: PrismImageRole,
) -> MeasureCheck<&NamedAnnularFace> {
    let found: Vec<_> = join
        .faces()
        .iter()
        .filter(|f| f.chart.role == role && matches!(f.domain, NamedAnnularDomain::Cap { .. }))
        .collect();
    count("one named cap at requested sweep end", found.len(), 1)?;
    Ok(found[0])
}
pub(super) fn named_side(
    join: &AnnularFrozenDomain,
    source: RegionSourceId,
) -> MeasureCheck<&NamedAnnularFace> {
    let found: Vec<_> = join.faces().iter().filter(|f| matches!(&f.domain, NamedAnnularDomain::Side { boundary } if boundary.source == source)).collect();
    count(
        "one named side for complete boundary source",
        found.len(),
        1,
    )?;
    Ok(found[0])
}
pub(super) fn direction(
    boundaries: &[AnnularBoundaryDirection; 2],
    source: RegionSourceId,
) -> MeasureCheck<AnnularBoundaryDirection> {
    let found: Vec<_> = boundaries
        .iter()
        .filter(|row| row.source == source)
        .collect();
    count(
        "one admitted increasing-parameter winding for source",
        found.len(),
        1,
    )?;
    Ok(*found[0])
}
pub(super) fn support(face: &FrozenPrismFace) -> MeasureCheck<&CapturedPlaneSupport> {
    match &face.surface {
        CapturedPrismSurface::Plane(support) => Ok(support),
        _ => Err(operand("actual cap plane support")),
    }
}
pub(super) fn trim<'a>(
    face: &'a FrozenPrismFace,
    key: &FrozenUseKey,
) -> MeasureCheck<&'a FrozenPrismTrim> {
    let found: Vec<_> = face
        .trims
        .iter()
        .filter(|trim| &trim.occurrence == key)
        .collect();
    count("one exact directed trim in named cap", found.len(), 1)?;
    Ok(found[0])
}
pub(super) fn normalized(
    curve: MetricCurve<'_, 2>,
    boundary: AnnularBoundaryDirection,
) -> OrientedMetricCurve<'_> {
    OrientedMetricCurve {
        curve,
        reversed: boundary.native_winding * boundary.material_side < 0,
    }
}
