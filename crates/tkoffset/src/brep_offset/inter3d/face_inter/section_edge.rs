//! Construction of a stored section edge from an IntTools face-face curve.
use super::not_implemented;
use super::orientation::{apply_side, orient_section};
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::same_parameter_edge;

pub(super) fn make_section_edge(
    curve: &Curve,
    first: &Face,
    second: &Face,
    side: State,
) -> Result<(Shape, Shape), Failure> {
    let (range, first_point, last_point) = curve_bounds(curve)?;
    let edge = make_edge_geometry(curve, range, first_point, last_point)?;
    add_face_pcurves(&edge, curve, first, second)?;
    let (first_orientation, second_orientation) = orient_section(curve, first, second, range)?;
    same_parameter_edge(&edge, precision::confusion())?;
    Ok((
        edge.oriented(apply_side(first_orientation, side)),
        edge.oriented(apply_side(second_orientation, side)),
    ))
}

fn curve_bounds(curve: &Curve) -> Result<((f64, f64), Point3, Point3), Failure> {
    let mut first = 0.0;
    let mut last = 0.0;
    let mut start = Point3::default();
    let mut finish = Point3::default();
    if !curve.bounds(&mut first, &mut last, &mut start, &mut finish)? {
        return Err(not_implemented("unbounded plane section curve"));
    }
    Ok(((first, last), start, finish))
}

fn make_edge_geometry(
    curve: &Curve,
    range: (f64, f64),
    first_point: Point3,
    last_point: Point3,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, curve.curve(), curve.tolerance())?;
    add_boundary_vertex(&builder, &mut edge, first_point, false)?;
    add_boundary_vertex(&builder, &mut edge, last_point, true)?;
    builder.range(&edge, range.0, range.1, false)?;
    Ok(edge)
}

fn add_boundary_vertex(
    builder: &BRepBuilder,
    edge: &mut Edge,
    point: Point3,
    reversed: bool,
) -> Result<(), Failure> {
    let mut vertex = Vertex::default();
    builder.make_vertex_at(&mut vertex, point, precision::confusion())?;
    let occurrence = if reversed {
        vertex.reversed()
    } else {
        vertex.into_shape()
    };
    builder.add(edge, &occurrence)
}

fn add_face_pcurves(
    edge: &Edge,
    curve: &Curve,
    first: &Face,
    second: &Face,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let first_curve = required_pcurve(curve.first_curve2d(), "first")?;
    let second_curve = required_pcurve(curve.second_curve2d(), "second")?;
    builder.update_edge_pcurve_on_face(edge, first_curve, first, curve.tolerance())?;
    builder.update_edge_pcurve_on_face(edge, second_curve, second, curve.tolerance())
}

fn required_pcurve<'a>(
    pcurve: &'a Handle<dyn Geom2dCurve>,
    label: &str,
) -> Result<&'a Handle<dyn Geom2dCurve>, Failure> {
    if pcurve.get().is_none() {
        return Err(Failure::with_message(
            "Standard_NotImplemented",
            &format!("plane section has null {label} pcurve; expected both face pcurves"),
        ));
    }
    Ok(pcurve)
}
