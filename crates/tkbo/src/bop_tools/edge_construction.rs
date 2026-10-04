use super::d_tolerance;
use crate::int_tools::Curve;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Shape, ShapeIterator, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
/// Empty-copy edge geometry while sharing its vertex children; `copy_edge(&edge)?`.
pub fn copy_edge(edge: &Edge) -> Result<Edge, Failure> {
    let mut result = forward_empty_copy(edge)?;
    for child in ShapeIterator::new(edge, false, true) {
        BRepBuilder::default().add(&mut result, &child)?;
    }
    result.set_orientation(edge.orientation());
    Ok(Edge::unchecked(result))
}
/// Copy edge geometry and install oriented endpoints at sorted bounds; `make_split_edge(&edge,&first,0.,&last,1.,&mut result)?`.
pub fn make_split_edge(
    edge: &Edge,
    first: &Vertex,
    first_parameter: f64,
    last: &Vertex,
    last_parameter: f64,
    result: &mut Edge,
) -> Result<(), Failure> {
    let mut shape = forward_empty_copy(edge)?;
    let increasing = first_parameter < last_parameter;
    let orientation = endpoint_orientation(increasing);
    append_endpoint(&mut shape, first, orientation)?;
    append_endpoint(
        &mut shape,
        last,
        rustycad_tkg3d::top_abs::reverse(orientation),
    )?;
    let range = if increasing {
        (first_parameter, last_parameter)
    } else {
        (last_parameter, first_parameter)
    };
    let new = Edge::unchecked(shape);
    BRepBuilder::default().range(&new, range.0, range.1, false)?;
    *result = new;
    result.set_orientation(edge.orientation());
    Ok(())
}
fn endpoint_orientation(increasing: bool) -> Orientation {
    if increasing {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}
fn forward_empty_copy(edge: &Edge) -> Result<Shape, Failure> {
    let mut source = edge.clone();
    source.set_orientation(Orientation::Forward);
    source.empty_copied()
}
fn append_endpoint(
    edge: &mut Shape,
    vertex: &Vertex,
    orientation: Orientation,
) -> Result<(), Failure> {
    if vertex.is_null() {
        return Ok(());
    }
    let mut vertex = vertex.clone();
    vertex.set_orientation(orientation);
    BRepBuilder::default().add(edge, &vertex)
}
/// Build a section edge and restore the input parameter order verbatim; `make_section_edge(&curve,&first,0.,&last,1.,&mut edge)?`.
pub fn make_section_edge(
    curve: &Curve,
    first: &Vertex,
    first_parameter: f64,
    last: &Vertex,
    last_parameter: f64,
    result: &mut Edge,
) -> Result<(), Failure> {
    let mut maker = BRepBuilderAPIMakeEdge::from_curve_vertices_params(
        curve.curve(),
        first,
        last,
        first_parameter,
        last_parameter,
    )?;
    let edge = Edge::unchecked(maker.shape()?.clone());
    BRepBuilder::default().range(&edge, first_parameter, last_parameter, false)?;
    *result = edge;
    Ok(())
}
/// Raise endpoint tolerances before building the section edge; `make_edge(&curve,&first,0.,&last,1.,1e-6,&mut edge)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_edge(
    curve: &Curve,
    first: &Vertex,
    first_parameter: f64,
    last: &Vertex,
    last_parameter: f64,
    tolerance: f64,
    result: &mut Edge,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let needed = tolerance + d_tolerance();
    builder.update_vertex_tolerance(first, needed)?;
    builder.update_vertex_tolerance(last, needed)?;
    make_section_edge(curve, first, first_parameter, last, last_parameter, result)?;
    builder.update_edge_tolerance(result, tolerance)
}
