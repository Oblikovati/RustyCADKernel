//! Adjacent intersection inputs and local deviation calculations.
use super::wire_intersection_fix::{CrossingAction, CrossingState};
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::{Direction3, Line3, Point3};

fn ordered_parameters(point: &IntersectionPoint, reverse: bool) -> (f64, f64) {
    if reverse {
        (point.param_on_second(), point.param_on_first())
    } else {
        (point.param_on_first(), point.param_on_second())
    }
}

pub(super) fn crossing_action(
    state: &mut CrossingState,
    index: i32,
    uv: &Sequence<IntersectionPoint>,
    xyz: &Sequence<Point3>,
    errors: &Sequence<f64>,
    item: usize,
) -> Result<Option<CrossingAction>, Failure> {
    let intersection = uv.value(item)?;
    let parameters = ordered_parameters(intersection, index == 1);
    if !state.accept_intersection(parameters) {
        return Ok(None);
    }
    let point = *xyz.value(item)?;
    let radius = *errors.value(item)?;
    let tolerance = 1.0001 * (state.point.distance(&point) + radius);
    Ok(Some(CrossingAction {
        point,
        radius,
        param1: parameters.0,
        param2: parameters.1,
        tolerance,
        improved: false,
    }))
}

pub(super) fn local_deviation(
    edge: &Edge,
    face: &Face,
    crossing: Point3,
    shared: Point3,
    first: f64,
    last: f64,
) -> Result<f64, Failure> {
    let (curve, low, high) = BRepTool::curve(edge)?;
    if curve.is_null() {
        return Ok(f64::MAX);
    }
    let params = remap_trimmed_line_parameters(edge, face, (first, last), (low, high))?;
    let line = Line3::new(
        crossing,
        Direction3::new(
            shared.x() - crossing.x(),
            shared.y() - crossing.y(),
            shared.z() - crossing.z(),
        )?,
    );
    sample_line_deviation(&curve, line, params)
}

fn remap_trimmed_line_parameters(
    edge: &Edge,
    face: &Face,
    parameters: (f64, f64),
    range: (f64, f64),
) -> Result<(f64, f64), Failure> {
    let (pcurve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let trimmed_curve = pcurve.cast::<Geom2dTrimmedCurve>();
    let Some(trimmed) = trimmed_curve.get() else {
        return Ok(parameters);
    };
    if trimmed.basis_curve().cast::<Geom2dLine>().is_null() {
        return Ok(parameters);
    }
    let scale = (range.1 - range.0) / (last - first);
    Ok((
        range.0 + (parameters.0 - first) * scale,
        range.0 + (parameters.1 - first) * scale,
    ))
}

fn sample_line_deviation(
    curve: &Handle<dyn GeomCurve>,
    line: Line3,
    range: (f64, f64),
) -> Result<f64, Failure> {
    let mut maximum: f64 = 0.0;
    let step = (range.1 - range.0) / 10.0;
    for sample in 1..10 {
        maximum = maximum.max(line.distance(curve.value(range.0 + f64::from(sample) * step)?));
    }
    Ok(maximum)
}
