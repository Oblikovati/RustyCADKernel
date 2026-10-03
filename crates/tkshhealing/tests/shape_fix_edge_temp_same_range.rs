//! Native-backed TempSameRange coverage for periodic, Bezier, and seam p-curves.
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::update_curves::as_gcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepCurveRepresentation, BRepTool};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction2, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

#[test]
fn pconfusion_controls_whether_pcurve_geometry_is_reparameterized() {
    let surface = plane_surface();
    for (name, factor) in [("pconfusion-under", 0.5), ("pconfusion-over", 2.0)] {
        let edge = line_range_edge(
            &surface,
            rustycad_tkernel::precision::p_confusion() * factor,
        );
        repair_range(&edge);
        assert_native_fixture(name, &edge, &surface);
    }
}

#[test]
fn periodic_pcurve_range_is_shifted_into_edge_parameters() {
    let surface = cylinder_surface();
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(Axis2d::default(), 1.0, true).unwrap()).cast();
    let edge = edge_with_pcurve(&surface, &pcurve, (7.0, 8.0));
    repair_range(&edge);
    assert_native_fixture("periodic", &edge, &surface);
}

#[test]
fn bezier_pcurve_segments_before_range_repair() {
    let surface = plane_surface();
    let pcurve = bezier_pcurve();
    let edge = edge_with_pcurve(&surface, &pcurve, (0.25, 0.75));
    repair_range(&edge);
    assert_native_fixture("bezier", &edge, &surface);
}

#[test]
fn periodic_first_seam_curve_shifts_second_curve_source_range() {
    let surface = cylinder_surface();
    let first: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(Axis2d::default(), 1.0, true).unwrap()).cast();
    let second: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(std::f64::consts::TAU, 0.0),
        Direction2::new(0.0, 1.0).unwrap(),
    )))
    .cast();
    let edge = seam_edge(&surface, &first, &second);
    repair_range(&edge);
    assert_native_fixture("seam", &edge, &surface);
}

fn repair_range(edge: &Edge) {
    let fixer = ShapeFixEdge::new();
    assert!(!fixer.fix_same_parameter(edge, 0.0).unwrap());
    assert!(BRepTool::same_range(edge).unwrap());
    assert!(BRepTool::same_parameter(edge).unwrap());
}

fn edge_with_pcurve(
    surface: &Handle<dyn GeomSurface>,
    pcurve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
) -> Edge {
    let edge = line_edge();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve(&edge, pcurve, surface, &Location::default(), 1.0e-7, None)
        .unwrap();
    builder
        .range_on_surface(&edge, surface, &Location::default(), range.0, range.1)
        .unwrap();
    builder.range(&edge, 0.0, 1.0, true).unwrap();
    builder.same_range(&edge, false).unwrap();
    edge
}

fn line_range_edge(surface: &Handle<dyn GeomSurface>, delta: f64) -> Edge {
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    edge_with_pcurve(surface, &curve, (delta, 1.0 + delta))
}

fn seam_edge(
    surface: &Handle<dyn GeomSurface>,
    first: &Handle<dyn Geom2dCurve>,
    second: &Handle<dyn Geom2dCurve>,
) -> Edge {
    let edge = line_edge();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurves(
            &edge,
            (first, second),
            surface,
            &Location::default(),
            1.0e-7,
            None,
        )
        .unwrap();
    builder
        .range_on_surface(&edge, surface, &Location::default(), 7.0, 8.0)
        .unwrap();
    builder.range(&edge, 0.0, 1.0, true).unwrap();
    builder.same_range(&edge, false).unwrap();
    edge
}

fn assert_native_fixture(name: &str, edge: &Edge, surface: &Handle<dyn GeomSurface>) {
    let fields = native_fixture_row(name);
    let found = BRepTool::curve_on_surface(edge, surface, &Location::default()).unwrap();
    assert_fixture_range(&fields, found.first, found.last, edge);
    assert_fixture_curve(&fields, &found.pcurve);
    assert_fixture_seam(&fields, edge);
    assert_fixture_flags(&fields, edge);
}

fn assert_fixture_range(fields: &[String], first: f64, last: f64, edge: &Edge) {
    assert_eq!(
        BRepTool::range(edge).unwrap(),
        (parse_number(&fields[1]), parse_number(&fields[2]))
    );
    assert_eq!(
        (first, last),
        (parse_number(&fields[3]), parse_number(&fields[4]))
    );
}

fn assert_fixture_curve(fields: &[String], curve: &Handle<dyn Geom2dCurve>) {
    assert_eq!(curve.get().unwrap().dynamic_type().name(), fields[5]);
    assert_point(curve, parse_point(fields, 6), 0.0);
    assert_point(curve, parse_point(fields, 8), 1.0);
}

fn assert_fixture_seam(fields: &[String], edge: &Edge) {
    if fields[10] != "1" {
        return;
    }
    let representation = closed_representation(edge);
    let second = as_gcurve(&representation).get().unwrap().pcurve2().unwrap();
    assert_eq!(second.get().unwrap().dynamic_type().name(), fields[11]);
    assert_eq!(
        BRepTool::range(edge).unwrap(),
        (parse_number(&fields[1]), parse_number(&fields[2]))
    );
    assert_point(&second, parse_point(fields, 14), parse_number(&fields[12]));
    assert_point(&second, parse_point(fields, 16), parse_number(&fields[13]));
}

fn assert_fixture_flags(fields: &[String], edge: &Edge) {
    assert_eq!(
        BRepTool::same_range(edge).unwrap(),
        fields[fields.len() - 2] == "1"
    );
    assert_eq!(
        BRepTool::same_parameter(edge).unwrap(),
        fields[fields.len() - 1] == "1"
    );
}

fn assert_point(curve: &Handle<dyn Geom2dCurve>, expected: Point2, parameter: f64) {
    assert!(curve.value(parameter).unwrap().distance(expected) < 1.0e-12);
}

fn closed_representation(edge: &Edge) -> Handle<dyn BRepCurveRepresentation> {
    let curves = edge_tshape(&edge.clone().into_shape(), "TempSameRange fixture")
        .unwrap()
        .curves();
    curves
        .into_iter()
        .find(|curve| {
            curve
                .get()
                .is_some_and(BRepCurveRepresentation::is_curve_on_closed_surface)
        })
        .unwrap()
}

fn native_fixture_row(name: &str) -> Vec<String> {
    include_str!("../../../tools/oracle/tkshhealing_temp_same_range.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn parse_number(field: &str) -> f64 {
    field.parse().unwrap()
}

fn parse_point(fields: &[String], index: usize) -> Point2 {
    Point2::new(
        parse_number(&fields[index]),
        parse_number(&fields[index + 1]),
    )
}

fn line_edge() -> Edge {
    BRepLibMakeEdge::from_points(Point3::default(), Point3::new(1.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap()
}

fn cylinder_surface() -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap()).cast()
}

fn plane_surface() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::default())).cast()
}

fn bezier_pcurve() -> Handle<dyn Geom2dCurve> {
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, 3).unwrap();
    for (index, point) in [
        Point2::new(0.0, 0.0),
        Point2::new(0.5, 1.0),
        Point2::new(1.0, 0.0),
    ]
    .into_iter()
    .enumerate()
    {
        poles.set_value(index as i32 + 1, point).unwrap();
    }
    Handle::new(Geom2dBezierCurve::new(&poles).unwrap()).cast()
}
