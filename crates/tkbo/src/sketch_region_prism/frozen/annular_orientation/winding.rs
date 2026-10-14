// SPDX-License-Identifier: GPL-2.0-only
//! The retained classifier includes traversal; undo it to recover increasing-parameter winding.
use super::*;
use rustycad_tkmath::predicates::roots::bezier_line::BezierWireLocation;
/// Recover increasing-parameter winding from the same certified annular source.
/// `annular_boundary_directions(join)` retains traversal and material-side provenance separately.
pub(in crate::sketch_region_prism) fn annular_boundary_directions(
    join: &AnnularFrozenDomain,
) -> AnnularDirectionCheck<[AnnularBoundaryDirection; 2]> {
    let nesting = join.embedding().binding().region().nesting();
    let outer = &nesting.outer_at_inner_closure;
    let inner = &nesting.inner_interior;
    Ok([
        derive(
            nesting.outer,
            outer.source,
            outer.reversed,
            outer.result.location(),
            1,
        )?,
        derive(
            nesting.inner,
            inner.source,
            inner.reversed,
            inner.result.location(),
            -1,
        )?,
    ])
}
fn derive(
    source: RegionSourceId,
    observed: RegionSourceId,
    reversed: bool,
    location: &BezierWireLocation,
    material_side: i8,
) -> AnnularDirectionCheck<AnnularBoundaryDirection> {
    count(
        "certified winding source role",
        usize::from(observed == source),
        1,
    )?;
    let actual = match location {
        BezierWireLocation::Inside { winding } => Some(*winding),
        _ => None,
    };
    let traversed_winding = actual
        .filter(|value| matches!(value, -1 | 1))
        .ok_or(AnnularDirectionCause::Winding { source, actual })?;
    let traversal = if reversed { -1 } else { 1 };
    Ok(AnnularBoundaryDirection {
        source,
        traversed_winding,
        reversed,
        native_winding: traversal * traversed_winding as i8,
        material_side,
    })
}
pub(super) fn boundary(
    rows: &[AnnularBoundaryDirection; 2],
    source: RegionSourceId,
) -> AnnularDirectionCheck<AnnularBoundaryDirection> {
    let found: Vec<_> = rows.iter().filter(|row| row.source == source).collect();
    count("one certified source-bound winding", found.len(), 1)?;
    Ok(*found[0])
}
#[cfg(test)]
#[path = "winding_tests.rs"]
mod tests;
