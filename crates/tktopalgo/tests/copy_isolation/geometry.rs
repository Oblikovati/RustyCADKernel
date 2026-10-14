use super::fixture::SharedCopyFixture;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg3d::geom::{GeomLine, GeomPlane};
use rustycad_tkmath::gp::{Direction3, Plane3, Point2, Point3};

#[derive(Debug, PartialEq)]
pub(super) struct AnalyticSnapshot {
    pub curve_origin: Point3,
    pub plane_coefficients: [f64; 4],
    pub pcurve_origin: Point2,
    pub edge_tolerance: f64,
    pub face_tolerance: f64,
    pub hierarchy_flags: [[bool; 8]; 4],
}

fn stored_line(edge: &Edge) -> Handle<GeomLine> {
    let representation = BRepTool::curve_with_location(edge).unwrap();
    assert!(!representation.curve.is_null());
    representation.curve.cast()
}

fn stored_plane(face: &Face) -> Handle<GeomPlane> {
    let (surface, _) = BRepTool::surface_with_location(face).unwrap();
    assert!(!surface.is_null());
    surface.cast()
}

fn stored_pcurve(edge: &Edge, face: &Face) -> Handle<Geom2dLine> {
    let representation = BRepTool::curve_on_face(edge, face).unwrap();
    assert!(!representation.pcurve.is_null());
    representation.pcurve.cast()
}

pub(super) fn shape_flags(shape: &Shape) -> [bool; 8] {
    [
        shape.free().unwrap(),
        shape.locked().unwrap(),
        shape.modified().unwrap(),
        shape.checked().unwrap(),
        shape.orientable().unwrap(),
        shape.closed().unwrap(),
        shape.infinite().unwrap(),
        shape.convex().unwrap(),
    ]
}

pub(super) fn analytic_snapshot(fixture: &SharedCopyFixture) -> AnalyticSnapshot {
    let line = stored_line(&fixture.edge);
    let plane = stored_plane(&fixture.faces[0]);
    let pcurve = stored_pcurve(&fixture.edge, &fixture.faces[0]);
    AnalyticSnapshot {
        curve_origin: line.get().unwrap().lin().location(),
        plane_coefficients: plane.get().unwrap().coefficients().unwrap(),
        pcurve_origin: pcurve.get().unwrap().location(),
        edge_tolerance: BRepTool::edge_tolerance(&fixture.edge).unwrap(),
        face_tolerance: BRepTool::face_tolerance(&fixture.faces[0]).unwrap(),
        hierarchy_flags: hierarchy_flags(fixture),
    }
}

pub(super) fn mutate_analytic_storage(fixture: &SharedCopyFixture, coordinate: f64) {
    let point = Point3::new(coordinate, 0.0, 0.0);
    stored_line(&fixture.edge)
        .get()
        .unwrap()
        .set_location(point);
    mutate_plane(&fixture.faces[0], coordinate);
    stored_pcurve(&fixture.edge, &fixture.faces[0])
        .get()
        .unwrap()
        .set_location(Point2::new(coordinate, 0.0));
    mutate_tolerances(fixture, coordinate);
    mutate_flags(&fixture.edge);
    mutate_flags(&fixture.root);
    mutate_flags(&fixture.faces[0]);
}

fn mutate_tolerances(fixture: &SharedCopyFixture, coordinate: f64) {
    let builder = BRepBuilder::default();
    builder
        .update_edge_tolerance(&fixture.edge, coordinate)
        .unwrap();
    builder
        .update_face_tolerance(&fixture.faces[0], coordinate)
        .unwrap();
}

fn mutate_flags(shape: &Shape) {
    shape.set_closed(true).unwrap();
    shape.set_free(true).unwrap();
    shape.set_checked(true).unwrap();
    shape.set_orientable(false).unwrap();
    shape.set_infinite(true).unwrap();
    shape.set_convex(true).unwrap();
}

fn mutate_plane(face: &Face, coordinate: f64) {
    let origin = Point3::new(0.0, 0.0, coordinate);
    let plane = Plane3::new(origin, Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    stored_plane(face).get().unwrap().set_pln(plane);
}

pub(super) fn assert_analytic_baseline(fixture: &SharedCopyFixture) {
    let actual = analytic_snapshot(fixture);
    assert_eq!(actual.curve_origin, Point3::default());
    assert_eq!(actual.pcurve_origin, Point2::default());
    assert_eq!(actual.plane_coefficients, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(actual.edge_tolerance, 0.02);
    assert_eq!(actual.face_tolerance, 0.01);
}

fn hierarchy_flags(fixture: &SharedCopyFixture) -> [[bool; 8]; 4] {
    [
        shape_flags(&fixture.root),
        shape_flags(&fixture.edge),
        shape_flags(&fixture.faces[0]),
        shape_flags(&fixture.faces[1]),
    ]
}
