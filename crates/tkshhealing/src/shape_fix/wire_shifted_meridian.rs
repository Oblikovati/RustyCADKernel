use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_analysis::adjust_to_period;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{Point2, Vector2};

struct MeridianCurves {
    previous: ParametricCurve,
    current: ParametricCurve,
    prior_singular: ParametricCurve,
    singular: ParametricCurve,
}

struct MeridianSamples {
    previous_end: Point2,
    current_start: Point2,
    prior_singular_end: Point2,
    singular_start: Point2,
    deep1: [Point2; 4],
    deep2: [Point2; 4],
}

struct ParametricCurve {
    curve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
}

/// Calculates the native meridian translation, or `None` if any required p-curve is missing.
pub(super) fn bimeridian_shift(
    edges: &[Edge],
    start: usize,
    stop: usize,
    face: &Face,
    closed: (bool, bool),
    periods: (f64, f64),
) -> Result<BimeridianShift, Failure> {
    let curves = collect_meridian_curves(edges, start, stop, face)?;
    let Some(curves) = curves else {
        return Ok(BimeridianShift::MissingCurve);
    };
    let shift = meridian_translation(curves, closed, periods)?;
    if shift == Vector2::default() {
        return Ok(BimeridianShift::NoCrossing);
    }
    Ok(BimeridianShift::Translate(shift))
}

pub(super) enum BimeridianShift {
    MissingCurve,
    NoCrossing,
    Translate(Vector2),
}

fn collect_meridian_curves(
    edges: &[Edge],
    start: usize,
    stop: usize,
    face: &Face,
) -> Result<Option<MeridianCurves>, Failure> {
    let (previous, current) = cyclic_neighbor_edges(edges, stop);
    let (prior_singular, singular) = cyclic_neighbor_edges(edges, start);
    let analysis = ShapeAnalysisEdge::new();
    let curves = MeridianCurves {
        previous: parametric_pcurve(&analysis, &previous, face)?,
        current: parametric_pcurve(&analysis, &current, face)?,
        prior_singular: parametric_pcurve(&analysis, &prior_singular, face)?,
        singular: parametric_pcurve(&analysis, &singular, face)?,
    };
    let present = [
        &curves.previous,
        &curves.current,
        &curves.prior_singular,
        &curves.singular,
    ]
    .iter()
    .all(|curve| !curve.curve.is_null());
    Ok(present.then_some(curves))
}

fn cyclic_neighbor_edges(edges: &[Edge], index: usize) -> (Edge, Edge) {
    (
        edges[(index + edges.len() - 1) % edges.len()].clone(),
        edges[index].clone(),
    )
}

fn parametric_pcurve(
    analysis: &ShapeAnalysisEdge,
    edge: &Edge,
    face: &Face,
) -> Result<ParametricCurve, Failure> {
    let (curve, first, last) = analysis.pcurve_on_face(edge, face, true)?;
    Ok(ParametricCurve { curve, first, last })
}

fn meridian_translation(
    curves: MeridianCurves,
    closed: (bool, bool),
    periods: (f64, f64),
) -> Result<Vector2, Failure> {
    let axis = usize::from(!closed.0);
    let period = if axis == 0 { periods.0 } else { periods.1 };
    calculate_bimeridian_shift(meridian_points(&curves)?, axis, period)
}

fn meridian_points(curves: &MeridianCurves) -> Result<MeridianSamples, Failure> {
    Ok(MeridianSamples {
        previous_end: curve_at(&curves.previous, curves.previous.last)?,
        current_start: curve_at(&curves.current, curves.current.first)?,
        prior_singular_end: curve_at(&curves.prior_singular, curves.prior_singular.last)?,
        singular_start: curve_at(&curves.singular, curves.singular.first)?,
        deep1: [
            curve_at(&curves.current, curves.current.last)?,
            curve_at(&curves.prior_singular, curves.prior_singular.first)?,
            curve_midpoint(&curves.current)?,
            curve_midpoint(&curves.prior_singular)?,
        ],
        deep2: [
            curve_at(&curves.previous, curves.previous.first)?,
            curve_at(&curves.singular, curves.singular.last)?,
            curve_midpoint(&curves.previous)?,
            curve_midpoint(&curves.singular)?,
        ],
    })
}

fn curve_at(curve: &ParametricCurve, parameter: f64) -> Result<Point2, Failure> {
    curve
        .curve
        .get()
        .ok_or_else(missing_pcurve)?
        .value(parameter)
}

fn curve_midpoint(curve: &ParametricCurve) -> Result<Point2, Failure> {
    curve_at(curve, 0.5 * (curve.first + curve.last))
}

fn calculate_bimeridian_shift(
    points: MeridianSamples,
    axis: usize,
    period: f64,
) -> Result<Vector2, Failure> {
    let rotation1 = cross_with_axis(points.previous_end, points.singular_start, axis);
    let rotation2 = cross_with_axis(points.prior_singular_end, points.current_start, axis);
    let displacement = scalar_component(points.singular_start, axis)
        - scalar_component(points.prior_singular_end, axis);
    let normal =
        scalar_component(points.current_start, axis) - scalar_component(points.previous_end, axis);
    if !meridian_crossing_is_valid(rotation1, rotation2, displacement, normal, period) {
        return Ok(Vector2::default());
    }
    let sign = if rotation2 > 0. { 1. } else { -1. };
    let deep1 = signed_minimum(
        sign,
        points.current_start,
        points.prior_singular_end,
        points.deep1,
        axis,
    );
    let deep2 = signed_maximum(
        sign,
        points.previous_end,
        points.singular_start,
        points.deep2,
        axis,
    );
    let epsilon = rustycad_tkernel::precision::p_confusion();
    let adjust = adjust_to_period(deep2 - deep1, epsilon, period + epsilon);
    let signed_adjust = if displacement > 0. { -adjust } else { adjust };
    Ok(axis_vector(axis, signed_adjust))
}

fn signed_minimum(
    sign: f64,
    first: Point2,
    second: Point2,
    middle: [Point2; 4],
    axis: usize,
) -> f64 {
    middle
        .into_iter()
        .chain([first, second])
        .map(|point| sign * scalar_component(point, axis))
        .fold(f64::INFINITY, f64::min)
}

fn signed_maximum(
    sign: f64,
    first: Point2,
    second: Point2,
    middle: [Point2; 4],
    axis: usize,
) -> f64 {
    middle
        .into_iter()
        .chain([first, second])
        .map(|point| sign * scalar_component(point, axis))
        .fold(f64::NEG_INFINITY, f64::max)
}

fn meridian_crossing_is_valid(
    rotation1: f64,
    rotation2: f64,
    displacement: f64,
    normal: f64,
    period: f64,
) -> bool {
    let precision = rustycad_tkernel::precision::p_confusion();
    rotation1 * rotation2 < -precision
        && displacement * normal < -precision
        && normal.abs() > 0.1 * period
        && displacement.abs() > 0.1 * period
        && rotation1 * displacement > precision
        && rotation2 * normal > precision
}

fn cross_with_axis(first: Point2, second: Point2, axis: usize) -> f64 {
    if axis == 0 {
        -(first.y() - second.y())
    } else {
        first.x() - second.x()
    }
}

fn scalar_component(point: Point2, axis: usize) -> f64 {
    if axis == 0 { point.x() } else { point.y() }
}

fn axis_vector(axis: usize, value: f64) -> Vector2 {
    if axis == 0 {
        Vector2::new(value, 0.)
    } else {
        Vector2::new(0., value)
    }
}

fn missing_pcurve() -> Failure {
    Failure::new("ShapeFix_Wire bi-meridian repair requires four p-curves")
}
