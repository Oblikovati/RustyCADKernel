use super::point_on_edge;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::{ncollection::List, precision, standard::Failure};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib;
/// Return the native Boolean tolerance margin; `assert_eq!(d_tolerance(),1e-12)`.
pub const fn d_tolerance() -> f64 {
    1e-12
}
/// Intersect a vertex tolerance sphere with a point sphere; `compute_vertex_point(&vertex,point,tolerance)?`.
pub fn compute_vertex_point(
    vertex: &Vertex,
    point: Point3,
    tolerance: f64,
) -> Result<i32, Failure> {
    let sum = BRepTool::vertex_tolerance(vertex)? + tolerance + precision::confusion();
    Ok(i32::from(
        BRepTool::pnt(vertex)?.square_distance(&point) > sum * sum,
    ))
}
/// Intersect vertex spheres using at least Confusion of fuzz; `compute_vertices(&first,&second,0.)?`.
pub fn compute_vertices(first: &Vertex, second: &Vertex, fuzz: f64) -> Result<i32, Failure> {
    let fuzz = if fuzz > precision::confusion() {
        fuzz
    } else {
        precision::confusion()
    };
    let sum = BRepTool::vertex_tolerance(first)? + BRepTool::vertex_tolerance(second)? + fuzz;
    Ok(i32::from(
        BRepTool::pnt(first)?.square_distance(&BRepTool::pnt(second)?) > sum * sum,
    ))
}
/// Reuse a singleton or bound multiple vertices, preserving output for an empty list; `make_vertex(&vertices,&mut output)?`.
pub fn make_vertex(vertices: &List<Shape>, output: &mut Vertex) -> Result<(), Failure> {
    if vertices.extent() == 1 {
        *output = Vertex::unchecked(vertices.first().expect("singleton vertex list").clone());
    } else if let Some((point, tolerance)) = brep_lib::bounding_vertex(vertices)? {
        make_new_vertex_point(point, tolerance, output)?;
    }
    Ok(())
}
/// Construct a vertex at a point with the given tolerance; `make_new_vertex_point(point,1e-7,&mut output)?`.
pub fn make_new_vertex_point(
    point: Point3,
    tolerance: f64,
    output: &mut Vertex,
) -> Result<(), Failure> {
    BRepBuilder::default().make_vertex_at(output, point, tolerance)
}
/// Construct a midpoint vertex covering both input spheres; `make_new_vertex_pair(&first,&second,&mut output)?`.
pub fn make_new_vertex_pair(
    first: &Vertex,
    second: &Vertex,
    output: &mut Vertex,
) -> Result<(), Failure> {
    midpoint_vertex(
        (BRepTool::pnt(first)?, BRepTool::vertex_tolerance(first)?),
        (BRepTool::pnt(second)?, BRepTool::vertex_tolerance(second)?),
        output,
    )
}
/// Construct a midpoint vertex from edge evaluations; `make_new_vertex_edges(&first,0.,&second,1.,&mut output)?`.
pub fn make_new_vertex_edges(
    first: &Edge,
    first_parameter: f64,
    second: &Edge,
    second_parameter: f64,
    output: &mut Vertex,
) -> Result<(), Failure> {
    let mut first_point = Point3::default();
    let mut second_point = Point3::default();
    point_on_edge(first, first_parameter, &mut first_point)?;
    point_on_edge(second, second_parameter, &mut second_point)?;
    midpoint_vertex(
        (first_point, BRepTool::edge_tolerance(first)?),
        (second_point, BRepTool::edge_tolerance(second)?),
        output,
    )
}
/// Construct an edge point with summed edge/face tolerances and DTolerance; `make_new_vertex_edge_face(&edge,0.,&face,&mut output)?`.
pub fn make_new_vertex_edge_face(
    edge: &Edge,
    parameter: f64,
    face: &Face,
    output: &mut Vertex,
) -> Result<(), Failure> {
    let mut point = Point3::default();
    point_on_edge(edge, parameter, &mut point)?;
    let tolerance =
        BRepTool::edge_tolerance(edge)? + BRepTool::face_tolerance(face)? + d_tolerance();
    make_new_vertex_point(point, tolerance, output)
}
fn midpoint_vertex(
    first: (Point3, f64),
    second: (Point3, f64),
    output: &mut Vertex,
) -> Result<(), Failure> {
    let max = if first.1 > second.1 {
        first.1
    } else {
        second.1
    };
    let tolerance = max + 0.5 * first.0.distance(&second.0);
    let point = Point3::new(
        0.5 * (first.0.x() + second.0.x()),
        0.5 * (first.0.y() + second.0.y()),
        0.5 * (first.0.z() + second.0.z()),
    );
    make_new_vertex_point(point, tolerance, output)
}
