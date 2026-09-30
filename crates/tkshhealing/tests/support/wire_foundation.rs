//! Geometric square wires with deliberate order, orientation, and missing representations.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Vertex, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3, Vector3};
use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;
pub fn order_fixture(
    order: i32,
    reverse: i32,
    missing: i32,
) -> (Wire, Face, Handle<ShapeExtendWireData>) {
    let builder = BRepBuilder::default();
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(2., 3., 0.),
        Point3::new(0., 3., 0.),
    ];
    let vertices: Vec<Vertex> = points
        .iter()
        .map(|&point| {
            let mut vertex = Vertex::default();
            builder.make_vertex_at(&mut vertex, point, 1e-7).unwrap();
            vertex
        })
        .collect();
    let edges: Vec<Edge> = (0..4)
        .map(|index| order_edge(&builder, &points, &vertices, index, missing))
        .collect();
    let wire_data = Handle::new(ShapeExtendWireData::new());
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    let indices = match order {
        0 => [0, 1, 2, 3],
        1 => [2, 0, 3, 1],
        2 => [1, 2, 3, 0],
        _ => [3, 2, 1, 0],
    };
    for index in indices {
        let mut edge = edges[index].clone();
        if reverse & (1 << index) != 0 {
            edge.reverse();
        }
        wire_data.add_edge(&edge, 0).unwrap();
        builder.add(&mut wire, &edge).unwrap();
    }
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    builder.add(&mut face, &wire).unwrap();
    (wire, face, wire_data)
}
fn order_edge(
    builder: &BRepBuilder,
    points: &[Point3; 4],
    vertices: &[Vertex],
    index: usize,
    missing: i32,
) -> Edge {
    let last = (index + 1) % 4;
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    if missing != 1 || index != 1 {
        let direction =
            Direction3::from_vector(Vector3::between(points[index], points[last])).unwrap();
        let curve: Handle<dyn GeomCurve> =
            Handle::new(GeomLine::new(Axis1::new(points[index], direction))).cast();
        builder.update_edge_curve(&edge, &curve, 1e-7).unwrap();
        builder
            .range(&edge, 0., points[index].distance(&points[last]), false)
            .unwrap();
    }
    if missing != 2 || index != 1 {
        builder
            .add(&mut edge, &vertices[index].oriented(Orientation::Forward))
            .unwrap();
    }
    builder
        .add(&mut edge, &vertices[last].oriented(Orientation::Reversed))
        .unwrap();
    edge
}
