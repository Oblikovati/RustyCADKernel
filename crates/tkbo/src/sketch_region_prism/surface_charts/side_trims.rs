// SPDX-License-Identifier: GPL-2.0-only
use super::{cap::prove_space_translation, identity::*, types::*};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;

pub(super) fn prove_side_trims(
    chart: &CapturedSurfaceChart,
    source: &CapturedSpaceRepresentation,
    height: f64,
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    for trim in &chart.trims {
        prove_one_trim(trim, chart, source, height, context)?;
        context.proof.trim_roles.push(trim.role);
    }
    Ok(())
}
fn prove_one_trim(
    trim: &CapturedPrismTrim,
    chart: &CapturedSurfaceChart,
    source: &CapturedSpaceRepresentation,
    height: f64,
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    match trim.expected {
        CapturedTrimExpectation::SideGenerating => {
            prove_generating(trim, chart, source, height, context)
        }
        CapturedTrimExpectation::SideDirecting { parameter } => {
            prove_directing(trim, chart, source, (height, parameter), context)
        }
        _ => Err(PrismSurfaceCause::MissingHistory {
            entity: "side trim expectation",
        }),
    }
}
fn prove_generating(
    trim: &CapturedPrismTrim,
    chart: &CapturedSurfaceChart,
    source: &CapturedSpaceRepresentation,
    height: f64,
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let last = trim.role == PrismTrimRole::Last;
    let ordinate = if last { -height } else { 0. };
    let offset = if last { chart.extrusion } else { [0.; 3] };
    prove_line_trim(trim, ([0., ordinate], [1., 0.]), [0., 1.], context)?;
    prove_space_translation(source, &trim.space, offset, context)
}
fn prove_directing(
    trim: &CapturedPrismTrim,
    chart: &CapturedSurfaceChart,
    source: &CapturedSpaceRepresentation,
    (height, parameter): (f64, f64),
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    prove_line_trim(trim, ([parameter, 0.], [0., -1.]), [0., height], context)?;
    let (origin, direction) = directing_line(trim)?;
    prove_directing_origin(source, &trim.space, (parameter, origin), context)?;
    prove_directing_direction(trim, chart, direction, context)
}
fn prove_directing_direction(
    trim: &CapturedPrismTrim,
    chart: &CapturedSurfaceChart,
    direction: [f64; 3],
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let (surface_direction, placement) = super::side::side_direction(chart);
    let expected = chain(
        line([0.; 3], surface_direction.map(|v| -v)),
        linear_part(placement),
        &[],
    );
    let actual = chain(
        line([0.; 3], direction),
        linear_part(trim.space.placement),
        &[],
    );
    context.equal(expected, actual, PrismChartStage::TrimSpaceCurve)
}
fn directing_line(trim: &CapturedPrismTrim) -> Result<([f64; 3], [f64; 3]), PrismSurfaceCause> {
    match trim.space.curve {
        CapturedMetricCurve::Line { origin, direction } => Ok((origin, direction)),
        _ => Err(PrismSurfaceCause::Unsupported {
            representation: "directing curve",
            actual: "nonlinear".to_owned(),
        }),
    }
}
fn prove_directing_origin(
    source: &CapturedSpaceRepresentation,
    actual: &CapturedSpaceRepresentation,
    (parameter, origin): (f64, [f64; 3]),
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    let pole = source_endpoint_pole(source, parameter)?;
    let expected = chain(line(pole, [0.; 3]), source.placement, &[]);
    let observed = chain(line(origin, [0.; 3]), actual.placement, &[]);
    context.equal(expected, observed, PrismChartStage::TrimSpaceCurve)
}
fn source_endpoint_pole(
    source: &CapturedSpaceRepresentation,
    parameter: f64,
) -> Result<[f64; 3], PrismSurfaceCause> {
    let CapturedMetricCurve::Bezier { poles, .. } = &source.curve else {
        return Err(PrismSurfaceCause::MissingHistory {
            entity: "Bézier endpoint",
        });
    };
    match parameter {
        0. => Ok(poles[0]),
        1. => Ok(*poles.last().unwrap()),
        _ => Err(PrismSurfaceCause::Range {
            actual: [parameter, parameter],
            expected: [0., 1.],
        }),
    }
}
fn prove_line_trim(
    trim: &CapturedPrismTrim,
    (origin, direction): ([f64; 2], [f64; 2]),
    interval: [f64; 2],
    context: &mut ProofContext<'_>,
) -> Result<(), PrismSurfaceCause> {
    require_range(trim.interval, interval)?;
    require_range(trim.space.interval, interval)?;
    let expected = chain(line(origin, direction), planar_identity(), &[]);
    let observed = chain(trim.pcurve.borrow(), planar_identity(), &[]);
    context.equal(expected, observed, PrismChartStage::TrimPCurve)
}
