//! Closed-path connector construction for `BRepOffset_Offset::Init`.
use super::{offset_rails::PipeFaceOptions, offset_rails_geometry};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Vertex},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_lib::{build_curve3d, same_parameter_edge, same_range};

pub(super) fn make_closed_connector(
    builder: &BRepBuilder,
    face: &Face,
    curves: [&Handle<dyn Geom2dCurve>; 2],
    endpoints: [Vertex; 2],
    range: (f64, f64),
    path_tolerance: f64,
    options: &PipeFaceOptions<'_>,
) -> Result<Edge, Failure> {
    let edge_match = offset_rails_geometry::supplied_connector(
        options.supplied_connectors[0],
        [&endpoints[0], &endpoints[1]],
    )?;
    let supplied = edge_match.is_some();
    let (edge, reversed) = closed_edge(builder, edge_match, [&endpoints[0], &endpoints[1]])?;
    let curves = offset_rails_geometry::reverse_connector_curves(curves, reversed)?;
    let range = offset_rails_geometry::reverse_connector_range(range, reversed);
    attach_closed_pcurves(builder, &edge, face, curves, range, path_tolerance)?;
    finish_closed_connector(builder, &edge, endpoints, supplied, path_tolerance, options)?;
    Ok(edge)
}

fn closed_edge(
    builder: &BRepBuilder,
    matched: Option<(Edge, bool)>,
    endpoints: [&Vertex; 2],
) -> Result<(Edge, bool), Failure> {
    if let Some((edge, reversed)) = matched {
        return Ok((edge, reversed));
    }
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    offset_rails_geometry::add_connector_vertices(builder, &mut edge, endpoints)?;
    Ok((edge, false))
}

fn attach_closed_pcurves(
    builder: &BRepBuilder,
    edge: &Edge,
    face: &Face,
    curves: [Handle<dyn Geom2dCurve>; 2],
    range: (f64, f64),
    tolerance: f64,
) -> Result<(), Failure> {
    builder.update_edge_pcurves_on_face(edge, (&curves[0], &curves[1]), face, tolerance)?;
    builder.range_on_face(edge, face, range.0, range.1)
}

fn finish_closed_connector(
    builder: &BRepBuilder,
    edge: &Edge,
    endpoints: [Vertex; 2],
    supplied: bool,
    path_tolerance: f64,
    options: &PipeFaceOptions<'_>,
) -> Result<(), Failure> {
    if endpoints[0].is_same(&endpoints[1]) {
        return builder.degenerated(edge, true);
    }
    if supplied {
        return build_supplied_connector(builder, edge, path_tolerance, options.tolerance);
    }
    offset_rails_geometry::compute_native_connector_curve(edge, options.surface)
}

fn build_supplied_connector(
    builder: &BRepBuilder,
    edge: &Edge,
    path_tolerance: f64,
    tolerance: f64,
) -> Result<(), Failure> {
    build_curve3d(edge, path_tolerance, Continuity::C1, 11, 30)?;
    builder.same_range(edge, false)?;
    builder.same_parameter(edge, false)?;
    same_range(edge, path_tolerance)?;
    same_parameter_edge(edge, tolerance)
}
