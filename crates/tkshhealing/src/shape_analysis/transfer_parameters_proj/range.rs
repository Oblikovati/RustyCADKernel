//! Native range transfer updates each curve representation before the final SameRange flag.
use super::*;
use crate::shape_build::ShapeBuildEdge;
use rustycad_tkbrep::brep::{BRepGCurve, access::edge_tshape, update_curves::as_gcurve};
use rustycad_tkernel::standard::real::epsilon;
mod correction;
mod representation;
struct TransferRangeContext {
    points: [Point3; 2],
    parameters: [f64; 2],
    fractions: [f64; 2],
    linear_ends: [bool; 2],
    edge_location: Location,
    precision: f64,
    from_planar: bool,
}
pub(super) fn transfer(
    tool: &ShapeAnalysisTransferParametersProj,
    state: &ProjectionTransferState,
    edge: &Edge,
    previous: f64,
    current: f64,
    is_2d: bool,
) -> Result<(), Failure> {
    let source = tool
        .transfer_parameter_data()
        .with(|fields| fields.edge.clone());
    ShapeBuildEdge.copy_ranges(edge, &source, 0., 1.)?;
    let Some(context) = prepare_context(tool, state, previous, current, is_2d)? else {
        return BRepBuilder::default().same_range(edge, false);
    };
    let mut same_range = true;
    for representation in edge_tshape(edge, "TransferParametersProj::TransferRange")?.curves() {
        let curve = as_gcurve(&representation);
        if curve.is_null() {
            continue;
        }
        if !representation::transfer(&context, &curve)? {
            same_range = false;
        }
    }
    BRepBuilder::default().same_range(edge, same_range)
}
fn prepare_context(
    tool: &ShapeAnalysisTransferParametersProj,
    state: &ProjectionTransferState,
    previous: f64,
    current: f64,
    is_2d: bool,
) -> Result<Option<TransferRangeContext>, Failure> {
    let parameters = if previous < current {
        [previous, current]
    } else {
        [current, previous]
    };
    let point1 = source_point(state, parameters[0], is_2d)?;
    if infinite_point(point1) {
        return Ok(None);
    }
    let point2 = source_point(state, parameters[1], is_2d)?;
    if infinite_point(point2) {
        return Ok(None);
    }
    build_context(tool, state, [point1, point2], parameters, is_2d)
}
fn build_context(
    tool: &ShapeAnalysisTransferParametersProj,
    state: &ProjectionTransferState,
    points: [Point3; 2],
    parameters: [f64; 2],
    is_2d: bool,
) -> Result<Option<TransferRangeContext>, Failure> {
    let bounds = tool.target_bounds(state, is_2d)?;
    let fractions = range_fractions(parameters, bounds);
    let edge_location = tool
        .transfer_parameter_data()
        .with(|fields| fields.edge.location().clone());
    Ok(Some(TransferRangeContext {
        points,
        parameters,
        fractions,
        linear_ends: [
            fractions[0] < precision::p_confusion(),
            1. - fractions[1] < precision::p_confusion(),
        ],
        edge_location,
        precision: state.precision,
        from_planar: is_2d,
    }))
}
fn source_point(
    state: &ProjectionTransferState,
    parameter: f64,
    is_2d: bool,
) -> Result<Point3, Failure> {
    if is_2d {
        return Ok(state
            .adaptor
            .value(parameter)?
            .transformed(state.location.transformation()));
    }
    state.spatial.value(parameter)
}
fn infinite_point(point: Point3) -> bool {
    precision::is_infinite(point.x())
        || precision::is_infinite(point.y())
        || precision::is_infinite(point.z())
}
fn range_fractions(parameters: [f64; 2], bounds: (f64, f64)) -> [f64; 2] {
    let span = bounds.1 - bounds.0;
    if span > epsilon(bounds.1) {
        return [
            (parameters[0] - bounds.0) / span,
            (parameters[1] - bounds.0) / span,
        ];
    }
    [0., 1.]
}
