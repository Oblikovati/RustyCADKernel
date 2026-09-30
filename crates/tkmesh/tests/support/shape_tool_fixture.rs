//! Native BRep construction used identically by ShapeTool tests and differential probes.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Vertex, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCircle, GeomCurve, GeomLine, GeomPlane, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3};
pub fn tool_topology(kind: i32, tolerance: f64, span: f64) -> (Edge, Face) {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    if kind != 0 {
        let curve: Handle<dyn GeomCurve> = if kind == 3 {
            Handle::new(GeomCircle::new(Axis2::default(), 1.).unwrap()).cast()
        } else {
            Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast()
        };
        builder.update_edge_curve(&edge, &curve, 2e-7).unwrap();
        builder.range(&edge, 0., span, false).unwrap();
    }
    if kind >= 2 {
        tool_vertices(&builder, &mut edge, kind, tolerance);
    }
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    builder.add(&mut wire, &edge).unwrap();
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    builder
        .make_face_on_surface(&mut face, &surface, 3e-7)
        .unwrap();
    builder.add(&mut face, &wire).unwrap();
    (edge, face)
}
fn tool_vertices(builder: &BRepBuilder, edge: &mut Edge, kind: i32, tolerance: f64) {
    let mut first = Vertex::default();
    builder
        .make_vertex_at(&mut first, Point3::default(), tolerance)
        .unwrap();
    builder
        .add(edge, &first.oriented(Orientation::Forward))
        .unwrap();
    let mut last = first.clone();
    if kind == 2 {
        builder
            .make_vertex_at(&mut last, Point3::new(1., 0., 0.), tolerance)
            .unwrap();
    }
    builder
        .add(edge, &last.oriented(Orientation::Reversed))
        .unwrap();
}
