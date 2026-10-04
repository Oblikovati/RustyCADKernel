use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{ShapeIterator, Vertex},
    toptools::IndexedMapOfShape,
};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::{
    gp::{Axis1, Point3, Transform3, Vector3},
    toploc::Location,
};
#[test]
fn native_point_curve_discrepancies_and_locations_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_point_curve_tolerance.expected").lines()
    {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (edge, vertices) = fixture([0.0001, 0.01, 0.1][row[0] as usize], row[3] as usize);
        let avoided = avoidance(&edge, row[2] as usize);
        bop_tools::correct_point_on_curve(
            &edge,
            &avoided,
            [0.0101, 0.1, 1.][row[1] as usize],
            row[4] != 0.,
        )
        .unwrap();
        for (vertex, expected) in vertices.iter().zip(&row[5..]) {
            let actual = BRepTool::vertex_tolerance(vertex).unwrap();
            assert!((actual - expected).abs() < 1e-14, "{line}: {actual}");
        }
    }
}
fn fixture(shift: f64, located: usize) -> (Edge, Vec<Vertex>) {
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge(&mut edge).unwrap();
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    builder.update_edge_curve(&edge, &curve, 0.001).unwrap();
    builder.range(&edge, 0., 1., false).unwrap();
    let mut vertices = Vec::new();
    for index in 0..3 {
        vertices.push(append_vertex(&mut edge, index, shift, located));
    }
    if located == 1 {
        edge.set_location(&translation(), true).unwrap();
    }
    (edge, vertices)
}
fn append_vertex(edge: &mut Edge, index: usize, shift: f64, located: usize) -> Vertex {
    let parameter = [0., 1., 0.5][index];
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(parameter, shift, 0.), 1e-7)
        .unwrap();
    vertex.set_orientation(
        [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
        ][index],
    );
    if located == 2 && index == 0 {
        vertex.set_location(&translation(), true).unwrap();
    }
    builder.add(edge, &vertex).unwrap();
    if index == 2 {
        builder
            .update_vertex_on_edge(&vertex, parameter, edge, 1e-7)
            .unwrap();
    }
    vertex
}
fn translation() -> Location {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(0., 0., 0.25));
    Location::new(transform)
}
fn avoidance(edge: &Edge, mode: usize) -> IndexedMapOfShape {
    let mut avoided = IndexedMapOfShape::default();
    if mode == 1 {
        avoided.add(edge.clone().into_shape());
    }
    if mode == 2 {
        avoided.add(Iterator::next(&mut ShapeIterator::new(edge, true, true)).unwrap());
    }
    avoided
}
