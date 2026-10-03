//! Exact FixAddPCurve cases from the pinned ShapeFix_Edge implementation.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface, GeomToroidalSurface};
use rustycad_tkmath::gp::{Axis3, Direction2, Point2, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

const ORACLE: &str = include_str!("../../../tools/oracle/tkshhealing_fix_add_pcurve.expected");

#[test]
fn cylinder_projection_and_face_overload_match_native() {
    let (edge, surface) = cylinder_edge(1.);
    let mut face = rustycad_tkbrep::topods::Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let fixer = ShapeFixEdge::new();
    let changed = fixer.fix_add_pcurve(&edge, &face, false, 1e-7).unwrap();
    assert_case(
        "cylinder-project",
        &fixer,
        &edge,
        &surface,
        &Location::default(),
        changed,
    );
}

#[test]
fn seam_adds_translated_second_cylinder_pcurve() {
    let (edge, surface) = cylinder_edge(1.);
    add_line_pcurve(&edge, &surface, Point2::new(0., 0.));
    let fixer = ShapeFixEdge::new();
    let changed = fixer
        .fix_add_pcurve_on_surface(&edge, &surface, &Location::default(), true, 1e-7)
        .unwrap();
    assert_case(
        "cylinder-second-seam",
        &fixer,
        &edge,
        &surface,
        &Location::default(),
        changed,
    );
}

#[test]
fn plane_short_circuits_and_missing_curve_sets_fail1() {
    let (edge, _) = cylinder_edge(1.);
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let fixer = ShapeFixEdge::new();
    let changed = fixer
        .fix_add_pcurve_on_surface(&edge, &plane, &Location::default(), false, 1e-7)
        .unwrap();
    assert_case(
        "plane-early-return",
        &fixer,
        &edge,
        &plane,
        &Location::default(),
        changed,
    );

    let mut missing = Edge::default();
    BRepBuilder::default().make_edge(&mut missing).unwrap();
    let (_, cylinder) = cylinder_edge(1.);
    let fixer = ShapeFixEdge::new();
    let changed = fixer
        .fix_add_pcurve_on_surface(&missing, &cylinder, &Location::default(), false, 1e-7)
        .unwrap();
    assert_case(
        "missing-3d-curve",
        &fixer,
        &missing,
        &cylinder,
        &Location::default(),
        changed,
    );
}

#[test]
fn located_surface_projects_world_space_curve() {
    let mut transform = Transform3::default();
    transform.set_translation(Vector3::new(3., 0., 0.));
    let location = Location::new(transform);
    let (edge, surface) = cylinder_edge(4.);
    let fixer = ShapeFixEdge::new();
    let changed = fixer
        .fix_add_pcurve_on_surface(&edge, &surface, &location, false, 1e-7)
        .unwrap();
    assert_case(
        "located-cylinder",
        &fixer,
        &edge,
        &surface,
        &location,
        changed,
    );
}

#[test]
fn torus_double_periodic_line_and_spline_use_native_translation() {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomToroidalSurface::new(Axis3::default(), 5., 2.).unwrap()).cast();
    let (line_edge, _) = cylinder_edge(7.);
    add_line_pcurve(&line_edge, &surface, Point2::new(0., 0.2));
    assert_torus_seam("torus-line-seam", &line_edge, &surface);

    let (spline_edge, _) = cylinder_edge(7.);
    add_spline_pcurve(&spline_edge, &surface);
    assert_torus_seam("torus-spline-seam", &spline_edge, &surface);
}

fn assert_torus_seam(name: &str, edge: &Edge, surface: &Handle<dyn GeomSurface>) {
    let fixer = ShapeFixEdge::new();
    let changed = fixer
        .fix_add_pcurve_on_surface(edge, surface, &Location::default(), true, 1e-7)
        .unwrap();
    assert_case(name, &fixer, edge, surface, &Location::default(), changed);
}

fn cylinder_edge(x: f64) -> (Edge, Handle<dyn GeomSurface>) {
    let edge = BRepLibMakeEdge::from_points(Point3::new(x, 0., 0.), Point3::new(x, 0., 1.))
        .unwrap()
        .edge()
        .unwrap();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast();
    (edge, surface)
}

fn add_line_pcurve(edge: &Edge, surface: &Handle<dyn GeomSurface>, point: Point2) {
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        point,
        Direction2::new(0., 1.).unwrap(),
    ))
    .cast();
    add_pcurve(edge, surface, &curve);
}

fn add_spline_pcurve(edge: &Edge, surface: &Handle<dyn GeomSurface>) {
    let poles = [Point2::new(0., 0.2), Point2::new(0., 1.)];
    let curve = Geom2dBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&[0., 1.], 1),
        &Array1::from_shared(&[2, 2], 1),
        1,
        false,
    )
    .unwrap();
    let curve: Handle<dyn Geom2dCurve> = Handle::new(curve).cast();
    add_pcurve(edge, surface, &curve);
}

fn add_pcurve(edge: &Edge, surface: &Handle<dyn GeomSurface>, curve: &Handle<dyn Geom2dCurve>) {
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve(edge, curve, surface, &Location::default(), 1e-7, None)
        .unwrap();
    builder
        .range_on_surface(edge, surface, &Location::default(), 0., 1.)
        .unwrap();
}

fn assert_case(
    name: &str,
    fixer: &ShapeFixEdge,
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    changed: bool,
) {
    let fields = expected(name);
    assert_eq!(changed, parse_bool(fields[1]));
    let analyzer = ShapeAnalysisEdge::new();
    assert_eq!(
        analyzer
            .has_pcurve_on_surface(edge, surface, location)
            .unwrap(),
        parse_bool(fields[2])
    );
    assert_eq!(
        analyzer
            .is_seam_on_surface(edge, surface, location)
            .unwrap(),
        parse_bool(fields[3])
    );
    assert_statuses(&fields, fixer);
    assert_expected_curves(&fields, edge);
}

fn assert_statuses(fields: &[&str], fixer: &ShapeFixEdge) {
    for (offset, index) in (1..=17).enumerate() {
        let status = ShapeExtendStatus::from_index(index).unwrap();
        assert_eq!(fixer.status_flag(status), parse_bool(fields[4 + offset]));
    }
}

fn assert_expected_curves(fields: &[&str], edge: &Edge) {
    assert_curve(fields, edge, 1, 21, "first");
    assert_curve(fields, edge, 2, 28, "second");
}

fn assert_curve(fields: &[&str], edge: &Edge, rank: i32, offset: usize, label: &str) {
    let expected_kind = fields[offset].parse::<i32>().unwrap();
    let actual = BRepTool::curve_on_surface_by_index(edge, rank).unwrap();
    assert_eq!(curve_kind(&actual.pcurve), expected_kind);
    if expected_kind == 0 {
        return;
    }
    let curve = actual.pcurve.get().unwrap();
    let first = actual.first;
    let last = actual.last;
    let start = curve.value(first).unwrap();
    let end = curve.value(last).unwrap();
    for (actual, field) in [first, last, start.x(), start.y(), end.x(), end.y()]
        .into_iter()
        .zip(&fields[offset + 1..offset + 7])
    {
        let expected = field.parse::<f64>().unwrap();
        assert!(
            (actual - expected).abs() < 1e-11,
            "native {label} p-curve: {actual} != {expected}"
        );
    }
}

fn curve_kind(curve: &Handle<dyn Geom2dCurve>) -> i32 {
    if curve.cast::<Geom2dLine>().get().is_some() {
        return 1;
    }
    if curve.cast::<Geom2dBSplineCurve>().get().is_some() {
        return 2;
    }
    0
}

fn expected(name: &str) -> Vec<&str> {
    ORACLE
        .lines()
        .find(|line| line.starts_with(name))
        .expect("native oracle case")
        .split_whitespace()
        .collect()
}

fn parse_bool(value: &str) -> bool {
    value == "1"
}
