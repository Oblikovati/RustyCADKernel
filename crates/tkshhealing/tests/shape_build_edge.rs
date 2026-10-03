//! Behavior tests of ShapeBuild_Edge and ShapeBuild_Vertex.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Transform2, Transform3, Vector3,
};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_build::{ShapeBuildEdge, ShapeBuildVertex};
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

const REASSIGN_ORACLE: &str =
    include_str!("../../../tools/oracle/tkshhealing_reassign_pcurve.expected");

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
fn native_fix_gap_edge_copy_replaces_vertices_and_pcurve_range() {
    let (edge, face) = edge_on_plane();
    let (first, last) = replacement_vertices();
    let pcurve = x_line_2d();
    let fixed = ShapeBuildEdge
        .copy_replace_vertices_with_pcurve_range(&edge, &first, &last, &pcurve, &face, (0.25, 0.75))
        .unwrap();
    assert!(!fixed.is_same(&edge));
    assert_eq!(
        BRepTool::range_on_face(&fixed, &face).unwrap(),
        (0.25, 0.75)
    );
    assert!(BRepTool::curve_on_face(&fixed, &face).unwrap().pcurve == pcurve);
    assert_eq!(BRepTool::range_on_face(&edge, &face).unwrap(), (0.0, 1.0));
    assert!(!BRepTool::same_range(&fixed).unwrap());
    let (fixed_first, fixed_last) = rustycad_tkbrep::topexp::vertices(&fixed, false);
    assert!(fixed_first.is_same(&first));
    assert!(fixed_last.is_same(&last));
}

#[test]
fn split_at_parameter_reuses_crossing_vertex_and_preserves_ranges() {
    let (edge, face) = edge_on_plane();
    let mut crossing = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut crossing, Point3::new(0.5, 0.0, 0.0), 1e-7)
        .unwrap();
    let (first, second) = ShapeBuildEdge
        .split_at_parameter(&edge, &face, 0.5, &crossing)
        .unwrap();
    assert_eq!(BRepTool::range_on_face(&first, &face).unwrap(), (0.0, 0.5));
    assert_eq!(BRepTool::range_on_face(&second, &face).unwrap(), (0.5, 1.0));
    let (_, first_end) = rustycad_tkbrep::topexp::vertices(&first, false);
    let (second_start, _) = rustycad_tkbrep::topexp::vertices(&second, false);
    assert!(first_end.is_same(&crossing));
    assert!(second_start.is_same(&crossing));
    assert_eq!(
        BRepTool::curve_on_face(&first, &face)
            .unwrap()
            .pcurve
            .value(0.5)
            .unwrap()
            .x(),
        0.5
    );
    assert_eq!(
        BRepTool::curve_on_face(&second, &face)
            .unwrap()
            .pcurve
            .value(0.5)
            .unwrap()
            .x(),
        0.5
    );
}

fn replacement_vertices() -> (Vertex, Vertex) {
    let builder = BRepBuilder::default();
    let mut first = Vertex::default();
    let mut last = Vertex::default();
    builder
        .make_vertex_at(&mut first, Point3::new(0.25, 0.0, 0.0), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, Point3::new(0.75, 0.0, 0.0), 1e-7)
        .unwrap();
    (first, last)
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

#[test]
fn reassign_pcurve_moves_single_and_plane_generated_curves() {
    let (edge, old_face, new_face) = planar_reassign_fixture(true);
    assert_reassign_case("ordinary", &edge, &old_face, &new_face);

    let (edge, old_face, new_face) = planar_reassign_fixture(false);
    assert_reassign_case("plane-generated", &edge, &old_face, &new_face);
}

#[test]
fn reassign_pcurve_splits_old_seam_and_extends_existing_target_curve() {
    let (edge, old_face, new_face) = cylindrical_seam_reassign_fixture();
    assert_reassign_case("seam-into-seam", &edge, &old_face, &new_face);
}

#[test]
fn reassign_pcurve_preserves_reversed_edge_seam_order() {
    let (mut edge, old_face, new_face) = cylindrical_seam_reassign_fixture();
    edge.reverse();
    assert_reassign_case("seam-into-seam-reversed", &edge, &old_face, &new_face);
}

#[test]
fn reassign_pcurve_returns_false_when_source_has_no_pcurve() {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    let old_face = cylinder_face(1.);
    let new_face = plane_face(0.);
    assert_reassign_case("missing-source", &edge, &old_face, &new_face);
}

fn planar_reassign_fixture(has_stored_curve: bool) -> (Edge, Face, Face) {
    let edge = BRepLibMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    let (old_face, new_face) = (plane_face(0.), plane_face(1.));
    if has_stored_curve {
        let builder = BRepBuilder::default();
        builder
            .update_edge_pcurve_on_face(&edge, &x_line_2d(), &old_face, 1e-7)
            .unwrap();
        builder.range_on_face(&edge, &old_face, 0., 1.).unwrap();
    }
    (edge, old_face, new_face)
}

fn plane_face(z: f64) -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(
        GeomPlane::from_point(Point3::new(0., 0., z), Direction3::new(0., 0., 1.).unwrap())
            .unwrap(),
    )
    .cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn cylindrical_seam_reassign_fixture() -> (Edge, Face, Face) {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    let (old_face, new_face) = (cylinder_face(1.), cylinder_face(2.));
    let first = line_pcurve(Point2::new(0., 0.));
    let second = line_pcurve(Point2::new(std::f64::consts::TAU, 0.));
    BRepBuilder::default()
        .update_edge_pcurves_on_face(&edge, (&first, &second), &old_face, 1e-7)
        .unwrap();
    BRepBuilder::default()
        .range_on_face(&edge, &old_face, 0., 1.)
        .unwrap();
    set_face_line_pcurve(&edge, &new_face, Point2::new(0.3, 0.));
    (edge, old_face, new_face)
}

fn cylinder_face(radius: f64) -> Face {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), radius).unwrap()).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn line_pcurve(point: Point2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(
        point,
        Direction2::new(0., 1.).unwrap(),
    ))
    .cast()
}

fn set_face_line_pcurve(edge: &Edge, face: &Face, point: Point2) {
    BRepBuilder::default()
        .update_edge_pcurve_on_face(edge, &line_pcurve(point), face, 1e-7)
        .unwrap();
    BRepBuilder::default()
        .range_on_face(edge, face, 0., 1.)
        .unwrap();
}

fn assert_reassign_case(name: &str, edge: &Edge, old_face: &Face, new_face: &Face) {
    let expected: Vec<&str> = REASSIGN_ORACLE
        .lines()
        .find(|line| line.starts_with(name))
        .unwrap()
        .split_whitespace()
        .collect();
    assert_eq!(
        ShapeBuildEdge
            .reassign_pcurve(edge, old_face, new_face)
            .unwrap(),
        expected[1] == "1"
    );
    assert_face_curve(&expected, edge, old_face, 2, 4, 6);
    assert_face_curve(&expected, edge, new_face, 3, 5, 12);
}

fn assert_face_curve(
    expected: &[&str],
    edge: &Edge,
    face: &Face,
    stored_index: usize,
    closed_index: usize,
    range_index: usize,
) {
    let curve = BRepTool::curve_on_face(edge, face).unwrap();
    let closed = BRepTool::is_closed_on_face(edge, face).unwrap();
    assert_eq!(curve.is_stored, expected[stored_index] == "1");
    assert_eq!(closed, expected[closed_index] == "1");
    if curve.pcurve.is_null() {
        return;
    }
    assert_pcurve_values(
        &curve.pcurve,
        curve.first,
        curve.last,
        expected,
        range_index,
    );
}

fn assert_pcurve_values(
    pcurve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    expected: &[&str],
    range_index: usize,
) {
    let pcurve = pcurve.get().unwrap();
    let first_point = pcurve.value(first).unwrap();
    let last_point = pcurve.value(last).unwrap();
    let actual = [
        first,
        last,
        first_point.x(),
        first_point.y(),
        last_point.x(),
        last_point.y(),
    ];
    for (index, (value, field)) in actual
        .into_iter()
        .zip(&expected[range_index..range_index + 6])
        .enumerate()
    {
        let oracle_value = field.parse::<f64>().unwrap();
        assert!(
            (value - oracle_value).abs() < 1e-11,
            "native p-curve mismatch at {index}: actual={value} expected={oracle_value} row={expected:?}"
        );
    }
}
