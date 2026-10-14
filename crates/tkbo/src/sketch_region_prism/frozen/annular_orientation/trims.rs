// SPDX-License-Identifier: GPL-2.0-only
//! Cap rims use their certified outer/inner wire; repeated First/Last roles never identify a rim.
use super::*;
pub(super) fn inspect(
    join: &AnnularFrozenDomain,
    face: &NamedAnnularFace,
    boundaries: &[AnnularBoundaryDirection; 2],
    facing: i8,
) -> AnnularDirectionCheck<Vec<AnnularTrimDirection>> {
    let expected = if matches!(face.domain, NamedAnnularDomain::Cap { .. }) {
        2
    } else {
        4
    };
    count(
        "complete annular face trim census",
        face.chart.trims.len(),
        expected,
    )?;
    let mut rows = face
        .chart
        .trims
        .iter()
        .map(|trim| inspect_trim(join, face, trim, boundaries, facing))
        .collect::<AnnularDirectionCheck<Vec<_>>>()?;
    rows.sort_by(|a, b| a.named.occurrence.cmp(&b.named.occurrence));
    Ok(rows)
}
fn inspect_trim(
    join: &AnnularFrozenDomain,
    face: &NamedAnnularFace,
    trim: &JoinedPrismTrim,
    boundaries: &[AnnularBoundaryDirection; 2],
    facing: i8,
) -> AnnularDirectionCheck<AnnularTrimDirection> {
    let source = source_for_trim(face, trim)?;
    let boundary = winding::boundary(boundaries, source)?;
    let expected = expected_trim_direction(
        face.chart.role,
        trim.role,
        boundary.material_side * boundary.native_winding,
        facing,
    );
    let actual = lookup::check_direction(join, &trim.occurrence, expected)?;
    let parents = lookup::parents(join, &trim.occurrence)?;
    Ok(AnnularTrimDirection {
        named: trim.clone(),
        boundary: source,
        parents,
        actual,
        expected,
    })
}
/// Select the certified source by complete wire ancestry; `annular_trim_source(face, trim)`.
pub(in crate::sketch_region_prism) fn source_for_trim(
    face: &NamedAnnularFace,
    trim: &JoinedPrismTrim,
) -> AnnularDirectionCheck<RegionSourceId> {
    let wires: Vec<_> = match &face.domain {
        NamedAnnularDomain::Cap { outer, inner } => vec![outer, inner],
        NamedAnnularDomain::Side { boundary } => vec![boundary],
    };
    let matches: Vec<_> = wires
        .into_iter()
        .filter(|wire| lookup::within_wire(&trim.occurrence, &wire.occurrence))
        .collect();
    count(
        "one certified source wire for exact trim ancestry",
        matches.len(),
        1,
    )?;
    Ok(matches[0].source)
}
