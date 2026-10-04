use super::algo_tools3d;
use crate::int_tools::{Context, Tools};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::{self, Explorer},
    topods::{Edge, Face, Shape, ShapeIterator, Solid, Vertex},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::gp::{Point2, Point3};
/// Classify a point with the context's cached solid classifier; `compute_state_point(point,&solid,1e-7,&context)?`.
pub fn compute_state_point(
    point: Point3,
    solid: &Solid,
    tolerance: f64,
    context: &Context,
) -> Result<State, Failure> {
    let cached = context.solid_classifier(solid)?;
    let mut classifier = cached.write();
    classifier.perform(&point, tolerance)?;
    Ok(classifier.state())
}
/// Classify a vertex's point without adding its tolerance; `compute_state_vertex(&vertex,&solid,1e-7,&context)?`.
pub fn compute_state_vertex(
    vertex: &Vertex,
    solid: &Solid,
    tolerance: f64,
    context: &Context,
) -> Result<State, Failure> {
    compute_state_point(BRepTool::pnt(vertex)?, solid, tolerance, context)
}
/// Classify a representative edge point, including native unbounded-range choices; `compute_state_edge(&edge,&solid,1e-7,&context)?`.
pub fn compute_state_edge(
    edge: &Edge,
    solid: &Solid,
    tolerance: f64,
    context: &Context,
) -> Result<State, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    if !curve.is_null() {
        return compute_state_point(
            curve.value(edge_sample(first, last))?,
            solid,
            tolerance,
            context,
        );
    }
    let vertex = topexp::first_vertex(edge, false);
    if vertex.is_null() {
        return Ok(State::Unknown);
    }
    compute_state_vertex(&vertex, solid, tolerance, context)
}
fn edge_sample(first: f64, last: f64) -> f64 {
    match (
        precision::is_negative_infinite(first),
        precision::is_positive_infinite(last),
    ) {
        (true, false) => last - 10.,
        (false, true) => first + 10.,
        (true, true) => 0.,
        (false, false) => Tools::intermediate_point(first, last),
    }
}
/// Classify an unshared edge or a point inside a fully shared face; `compute_state_face(&face,&solid,1e-7,&bounds,&context)?`.
pub fn compute_state_face(
    face: &Face,
    solid: &Solid,
    tolerance: f64,
    bounds: &IndexedMapOfShape,
    context: &Context,
) -> Result<State, Failure> {
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = Edge::unchecked(shape);
        if !BRepTool::degenerated(&edge)? && !bounds.contains(&edge) {
            return compute_state_edge(&edge, solid, tolerance, context);
        }
    }
    let (mut point, mut uv) = (Point3::default(), Point2::default());
    let status = face_sample(face, &mut point, &mut uv, context)?;
    if status != 0 {
        return Ok(State::Unknown);
    }
    compute_state_point(point, solid, tolerance, context)
}
fn face_sample(
    face: &Face,
    point: &mut Point3,
    uv: &mut Point2,
    context: &Context,
) -> Result<i32, Failure> {
    let mut status = algo_tools3d::point_in_face(face, point, uv, context)?;
    if status == 0 {
        return Ok(status);
    }
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = Edge::unchecked(shape);
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        status = algo_tools3d::point_near_edge(&edge, face, uv, point, context)?;
        if status == 0 {
            break;
        }
    }
    Ok(status)
}
/// Dispatch classification while retaining native container return behavior; `compute_state_by_one_point(&shape,&solid,1e-7,&context)?`.
pub fn compute_state_by_one_point(
    shape: &Shape,
    solid: &Solid,
    tolerance: f64,
    context: &Context,
) -> Result<State, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Vertex => {
            compute_state_vertex(&Vertex::unchecked(shape.clone()), solid, tolerance, context)
        }
        ShapeEnum::Edge => {
            compute_state_edge(&Edge::unchecked(shape.clone()), solid, tolerance, context)
        }
        ShapeEnum::Face => classify_face_with_solid_bounds(shape, solid, tolerance, context),
        _ => classify_container(shape, solid, tolerance, context),
    }
}
fn classify_face_with_solid_bounds(
    shape: &Shape,
    solid: &Solid,
    tolerance: f64,
    context: &Context,
) -> Result<State, Failure> {
    let mut bounds = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(solid, ShapeEnum::Edge, &mut bounds)?;
    compute_state_face(
        &Face::unchecked(shape.clone()),
        solid,
        tolerance,
        &bounds,
        context,
    )
}
fn classify_container(
    shape: &Shape,
    solid: &Solid,
    tolerance: f64,
    context: &Context,
) -> Result<State, Failure> {
    if let Some(child) = Iterator::next(&mut ShapeIterator::new(shape, true, true)) {
        // OCCT 3d097a03 ComputeStateByOnePoint discards this result; preserve its side effects.
        compute_state_by_one_point(&child, solid, tolerance, context)?;
    }
    Ok(State::Unknown)
}
