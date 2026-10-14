// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn inspect(
    join: &AnnularFrozenDomain,
    boundaries: &[AnnularBoundaryDirection; 2],
    jacobian: i8,
    cancelled: &mut impl FnMut() -> bool,
) -> AnnularDirectionCheck<Vec<AnnularFaceDirection>> {
    count(
        "four completely named annular face domains",
        join.faces().len(),
        4,
    )?;
    let mut rows = Vec::new();
    for face in join.faces() {
        operation::poll(cancelled)?;
        rows.push(inspect_face(join, face, boundaries, jacobian)?);
    }
    complete_rows(&rows)?;
    rows.sort_by(|a, b| a.named.chart.occurrence.cmp(&b.named.chart.occurrence));
    Ok(rows)
}
fn inspect_face(
    join: &AnnularFrozenDomain,
    face: &NamedAnnularFace,
    boundaries: &[AnnularBoundaryDirection; 2],
    jacobian: i8,
) -> AnnularDirectionCheck<AnnularFaceDirection> {
    let native = face_sign(face, boundaries)?;
    let facing = prism_outward_sign(face.chart.role, native, jacobian);
    let expected = signed_prism_orientation(facing);
    let actual = lookup::check_direction(join, &face.chart.occurrence, expected)?;
    let trims = trims::inspect(join, face, boundaries, facing)?;
    let parents = lookup::parents(join, &face.chart.occurrence)?;
    let mut named = face.clone();
    named
        .chart
        .trims
        .sort_by(|a, b| a.occurrence.cmp(&b.occurrence));
    Ok(AnnularFaceDirection {
        named,
        parents,
        actual,
        expected,
        trims,
    })
}
fn face_sign(
    face: &NamedAnnularFace,
    boundaries: &[AnnularBoundaryDirection; 2],
) -> AnnularDirectionCheck<i8> {
    match &face.domain {
        NamedAnnularDomain::Cap { .. }
            if matches!(
                face.chart.role,
                PrismImageRole::First | PrismImageRole::Last
            ) =>
        {
            Ok(1)
        }
        NamedAnnularDomain::Side { boundary } if face.chart.role == PrismImageRole::Generated => {
            let source = winding::boundary(boundaries, boundary.source)?;
            Ok(source.material_side * source.native_winding)
        }
        _ => Err(AnnularDirectionCause::Context {
            occurrence: Box::new(face.chart.occurrence.clone()),
            required: "certified cap or source-specific Generated side role",
        }),
    }
}

fn complete_rows(rows: &[AnnularFaceDirection]) -> AnnularDirectionCheck<()> {
    lookup::distinct(
        rows.iter().map(|row| &row.named.chart.occurrence).collect(),
        4,
    )?;
    lookup::distinct(
        rows.iter()
            .flat_map(|row| row.trims.iter().map(|t| &t.named.occurrence))
            .collect(),
        12,
    )?;
    Ok(())
}
