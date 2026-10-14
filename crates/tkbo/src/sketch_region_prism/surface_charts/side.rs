// SPDX-License-Identifier: GPL-2.0-only
//! The full extrusion identity requires both its actual basis and its stored direction/trim height.
use super::{identity::*, types::*};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

pub(super) fn prove_side(
    chart: &CapturedSurfaceChart,
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let source = side_source(chart)?;
    prove_basis(source, &chart.actual, context)?;
    let height = side_height(&chart.trims)?;
    prove_extrusion_direction(chart, height, context)?;
    super::side_trims::prove_side_trims(chart, source, height, context)?;
    context.proof.side_height = Some(height);
    Ok(())
}
fn side_source(
    chart: &CapturedSurfaceChart,
) -> Result<&CapturedSpaceRepresentation, PrismSurfaceCause> {
    let CapturedChartSource::Side(source) = &chart.source else {
        return Err(missing_side());
    };
    require_range(source.interval, [0., 1.])?;
    Ok(source)
}
fn prove_basis(
    source: &CapturedSpaceRepresentation,
    actual: &CapturedPrismSurface,
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let CapturedPrismSurface::Extrusion {
        basis, placement, ..
    } = actual
    else {
        return Err(missing_side());
    };
    let expected = chain(source.curve.borrow(), source.placement, &[]);
    let observed = chain(basis.borrow(), *placement, &[]);
    context.equal(expected, observed, PrismChartStage::SideBasis)
}
fn missing_side() -> PrismSurfaceCause {
    PrismSurfaceCause::Unsupported {
        representation: "Bézier edge to stored extrusion surface",
        actual: "different captured source/surface kind".to_owned(),
    }
}
fn side_height(trims: &[CapturedPrismTrim]) -> Result<f64, PrismSurfaceCause> {
    require_rectangle_roles(trims)?;
    let last = trims
        .iter()
        .find(|t| t.role == PrismTrimRole::Last)
        .unwrap();
    let CapturedMetricCurve::Line { origin, .. } = last.pcurve else {
        return Err(missing_side());
    };
    let height = -origin[1];
    if !height.is_finite() || height <= 0. {
        return Err(PrismSurfaceCause::InvalidHeight {
            actual: height,
            expected: "finite strictly positive height",
        });
    }
    Ok(height)
}
fn require_rectangle_roles(trims: &[CapturedPrismTrim]) -> Result<(), PrismSurfaceCause> {
    require_count("side trim occurrences", trims.len(), 4)?;
    for role in [
        PrismTrimRole::First,
        PrismTrimRole::Last,
        PrismTrimRole::Start,
        PrismTrimRole::End,
    ] {
        require_count(
            "rectangle side role",
            trims.iter().filter(|t| t.role == role).count(),
            1,
        )?;
    }
    Ok(())
}
fn prove_extrusion_direction(
    chart: &CapturedSurfaceChart,
    height: f64,
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let (direction, placement) = side_direction(chart);
    let mut scale = spatial_identity();
    scale.scale = -height;
    let placements = [linear_part(placement)];
    let expected = chain(line([0.; 3], chart.extrusion), spatial_identity(), &[]);
    let actual = chain(line([0.; 3], direction), scale, &placements);
    context.equal(expected, actual, PrismChartStage::SideDirection)
}
pub(super) fn side_direction(chart: &CapturedSurfaceChart) -> ([f64; 3], MetricEmbedding<3>) {
    let CapturedPrismSurface::Extrusion {
        direction,
        placement,
        ..
    } = chart.actual
    else {
        unreachable!("side kind checked before trim proof")
    };
    (direction, placement)
}
