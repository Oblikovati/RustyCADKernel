//! Behavior tests of ShapeBuild_Edge and ShapeBuild_Vertex.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Transform2, Transform3, Vector3,
};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_build::{ShapeBuildEdge, ShapeBuildVertex};

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::default())).cast()
}

fn x_line() -> Handle<dyn GeomCurve> {
    let axis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    Handle::new(GeomLine::new(axis)).cast()
}

fn x_line_2d() -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Handle::new(Geom2dLine::new(axis)).cast()
}

/// The edge [0, 1] of the X axis, with its p-curve on the plane z = 0.
fn edge_on_plane() -> (Edge, Face) {
    let mut edge = Edge::default();
    ShapeBuildEdge
        .make_edge_on_curve_range(&mut edge, &x_line(), &Location::default(), 0.0, 1.0)
        .unwrap();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &plane(), 1e-7)
        .unwrap();
    BRepBuilder::default()
        .update_edge_pcurve_on_face(&edge, &x_line_2d(), &face, 0.0)
        .unwrap();
    (edge, face)
}

#[test]
fn copy_makes_a_new_edge_and_optionally_new_pcurves() {
    let (edge, face) = edge_on_plane();
    let shared = ShapeBuildEdge.copy(&edge, true).unwrap();
    let own = ShapeBuildEdge.copy(&edge, false).unwrap();
    assert!(!shared.is_same(&edge));
    let original = BRepTool::curve_on_face(&edge, &face).unwrap().pcurve;
    assert!(BRepTool::curve_on_face(&shared, &face).unwrap().pcurve == original);
    assert!(BRepTool::curve_on_face(&own, &face).unwrap().pcurve != original);
}

#[test]
fn copy_replace_vertices_takes_the_given_vertices() {
    let (edge, _) = edge_on_plane();
    let mut v = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut v, Point3::new(-1.0, 0.0, 0.0), 1e-3)
        .unwrap();
    let copy = ShapeBuildEdge
        .copy_replace_vertices(&edge, &v, &Vertex::default())
        .unwrap();
    let (first, last) = rustycad_tkbrep::topexp::vertices(&copy, false);
    assert!(first.is_same(&v));
    assert_eq!(BRepTool::pnt(&last).unwrap().x(), 1.0);
}

#[test]
fn ranges_and_pcurves_are_set_replaced_and_removed() {
    let (edge, face) = edge_on_plane();
    ShapeBuildEdge.set_range_3d(&edge, 0.25, 0.5).unwrap();
    assert_eq!(BRepTool::curve(&edge).unwrap().1, 0.25);
    let other = x_line_2d();
    ShapeBuildEdge.replace_pcurve(&edge, &other, &face).unwrap();
    assert!(BRepTool::curve_on_face(&edge, &face).unwrap().pcurve == other);
    ShapeBuildEdge.remove_curve_3d(&edge).unwrap();
    assert!(BRepTool::curve(&edge).unwrap().0.is_null());
    // The 3D curve is rebuilt from the p-curve on the plane.
    assert!(ShapeBuildEdge.build_curve_3d(&edge));
    assert!(!BRepTool::curve(&edge).unwrap().0.is_null());
    ShapeBuildEdge.remove_pcurve_on_face(&edge, &face).unwrap();
    // A plane face gives a p-curve computed on the fly, not stored.
    assert!(!BRepTool::curve_on_face(&edge, &face).unwrap().is_stored);
}

#[test]
fn transform_pcurve_scales_a_line_along_u() {
    let (curve, first, last) = ShapeBuildEdge
        .transform_pcurve(&x_line_2d(), Transform2::default(), 2.0, 0.0, 1.0)
        .unwrap();
    assert_eq!((first, last), (0.0, 2.0));
    assert_eq!(curve.value(2.0).unwrap().x(), 2.0);
}

#[test]
fn a_located_make_edge_moves_its_vertices() {
    let mut t = Transform3::default();
    t.set_translation(Vector3::new(0.0, 0.0, 5.0));
    let mut edge = Edge::default();
    ShapeBuildEdge
        .make_edge_on_curve_range(&mut edge, &x_line(), &Location::new(t), 0.0, 1.0)
        .unwrap();
    let (first, _) = rustycad_tkbrep::topexp::vertices(&edge, false);
    assert_eq!(BRepTool::pnt(&first).unwrap().z(), 5.0);
}

#[test]
fn combine_vertex_keeps_the_larger_sphere_or_spans_both() {
    let inner = ShapeBuildVertex
        .combine_points(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.5, 0.0, 0.0),
            1.0,
            0.1,
            1.0,
        )
        .unwrap();
    assert_eq!(BRepTool::vertex_tolerance(&inner).unwrap(), 1.0);
    let spanning = ShapeBuildVertex
        .combine_points(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            0.5,
            0.5,
            1.0,
        )
        .unwrap();
    assert_eq!(BRepTool::pnt(&spanning).unwrap().x(), 1.0);
    assert_eq!(BRepTool::vertex_tolerance(&spanning).unwrap(), 1.5);
}
