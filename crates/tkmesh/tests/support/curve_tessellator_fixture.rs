//! Deliberately mismatched representations exercise native pcurve refinement and bounds.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::{ncollection::Array2, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomBezierSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomSurface},
    top_abs::Orientation,
};
use rustycad_tkmath::{
    gp::{Axis1, Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkmesh::{
    brep_mesh_data::DiscreteModel,
    imesh_data::{EdgeHandle, FaceHandle, MeshModel},
};
pub fn refinement_fixture(kind: i32, located: bool, internal: bool) -> (EdgeHandle, FaceHandle) {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    let surface = refinement_surface(kind);
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let mut edge = refinement_edge(&face, kind);
    if internal {
        add_internal_vertex(&mut edge);
    }
    let location = Location::new(Transform3::translation_vector(Vector3::new(
        if located { 7.0 } else { 0.0 },
        0.0,
        0.0,
    )));
    let model = DiscreteModel::new(&Shape::default());
    let discrete_face = model
        .add_face(&rustycad_tkbrep::topods::face(&face.located(&location, false).unwrap()).unwrap())
        .unwrap();
    let discrete_edge = model
        .add_edge(&rustycad_tkbrep::topods::edge(&edge.located(&location, false).unwrap()).unwrap())
        .unwrap();
    discrete_edge.add_pcurve(discrete_face.downgrade(), Orientation::Forward);
    (discrete_edge, discrete_face)
}
fn refinement_surface(kind: i32) -> Handle<dyn GeomSurface> {
    if kind == 0 {
        return Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap()).cast();
    }
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    for row in 1..=2 {
        for column in 1..=2 {
            poles[(row, column)] = Point3::new(f64::from(row - 1), f64::from(column - 1), 0.0);
        }
    }
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
fn refinement_edge(face: &Face, kind: i32) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast();
    builder
        .make_edge_with_curve(&mut edge, &curve, 1e-7)
        .unwrap();
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(0.0, 0.5),
        Direction2::X,
    )))
    .cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, face, 1e-7)
        .unwrap();
    builder
        .range(&edge, if kind == 0 { 0.0 } else { -1.0 }, 2.0, false)
        .unwrap();
    edge
}
fn add_internal_vertex(edge: &mut Edge) {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(0.37, 0.0, 0.0), 1e-7)
        .unwrap();
    vertex.set_orientation(Orientation::Internal);
    builder.add(edge, &vertex).unwrap();
    builder
        .update_vertex_on_edge(&vertex, 0.37, edge, 1e-7)
        .unwrap();
}
