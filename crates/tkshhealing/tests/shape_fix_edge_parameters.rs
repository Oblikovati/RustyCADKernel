//! Native status and tolerance fixture, including the status reset during vertex repair.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis2d, Direction3, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

#[test]
fn same_parameter_repairs_match_native_status_and_tolerances() {
    for (name, same, z) in [
        ("same-flat", true, 0.),
        ("same-gap", true, 0.2),
        ("repair-flat", false, 0.),
        ("repair-gap", false, 0.2),
    ] {
        let (edge, _) = edge_with_pcurve(same, z);
        let mut fixer = ShapeFixEdge::new();
        let changed = fixer.fix_same_parameter(&edge, 0.).unwrap();
        assert_fixture(name, &fixer, &edge, changed);
    }
}

#[test]
fn degenerated_edge_only_restores_parameter_flag() {
    let (edge, _) = edge_with_pcurve(false, 0.2);
    BRepBuilder::default().degenerated(&edge, true).unwrap();
    let mut fixer = ShapeFixEdge::new();
    let changed = fixer.fix_same_parameter(&edge, 0.).unwrap();
    assert_fixture("degenerated", &fixer, &edge, changed);
}

#[test]
fn vertex_repair_context_preserves_source_and_replaces_result_vertices() {
    for with_context in [false, true] {
        let (edge, _) = edge_with_pcurve(true, 0.2);
        let mut fixer = ShapeFixEdge::new();
        assert!(fixer.context().is_null());
        let context = Handle::new(ShapeBuildReShape::new());
        if with_context {
            fixer.set_context(&context);
        }
        let changed = fixer.fix_vertex_tolerance(&edge).unwrap();
        let name = if with_context {
            "vertex-context-source"
        } else {
            "vertex-direct"
        };
        assert_fixture(name, &fixer, &edge, changed);
        if with_context {
            let result = Edge::unchecked(context.apply(&edge, ShapeEnum::Shape).unwrap());
            assert_fixture("vertex-context-result", &fixer, &result, changed);
        }
    }
}

fn edge_with_pcurve(same: bool, z: f64) -> (Edge, Handle<dyn GeomSurface>) {
    let edge = BRepLibMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(
        GeomPlane::from_point(Point3::new(0., 0., z), Direction3::new(0., 0., 1.).unwrap())
            .unwrap(),
    )
    .cast();
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    let builder = BRepBuilder::default();
    builder
        .update_edge_pcurve(&edge, &pcurve, &surface, &Location::default(), 1e-7, None)
        .unwrap();
    builder
        .range_on_surface(&edge, &surface, &Location::default(), 0., 1.)
        .unwrap();
    builder.same_parameter(&edge, same).unwrap();
    (edge, surface)
}

fn assert_fixture(name: &str, fixer: &ShapeFixEdge, edge: &Edge, changed: bool) {
    let fixture = include_str!("../../../tools/oracle/tkshhealing_fix_edge_parameters.expected");
    let row = fixture
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    let mut actual = vec![f64::from(changed)];
    for index in 1..=17 {
        actual.push(f64::from(
            fixer.status_flag(ShapeExtendStatus::from_index(index).unwrap()),
        ));
    }
    actual.extend([
        f64::from(BRepTool::same_parameter(edge).unwrap()),
        f64::from(BRepTool::same_range(edge).unwrap()),
    ]);
    let analyzer = ShapeAnalysisEdge::new();
    actual.extend([
        BRepTool::edge_tolerance(edge).unwrap(),
        BRepTool::vertex_tolerance(&analyzer.first_vertex(edge)).unwrap(),
        BRepTool::vertex_tolerance(&analyzer.last_vertex(edge)).unwrap(),
    ]);
    assert_eq!(actual, expected, "native fixture {name}");
}

#[test]
fn same_range_repair_uses_parameter_confusion() {
    let (edge, surface) = edge_with_pcurve(true, 0.);
    let builder = BRepBuilder::default();
    builder.same_range(&edge, false).unwrap();
    builder
        .range_on_surface(&edge, &surface, &Location::default(), 2e-8, 1. + 2e-8)
        .unwrap();
    let mut fixer = ShapeFixEdge::new();
    assert!(!fixer.fix_same_parameter(&edge, 0.).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Ok));
    assert!(BRepTool::same_range(&edge).unwrap());
    let (_, first, last) = ShapeAnalysisEdge::new()
        .pcurve_on_surface(&edge, &surface, &Location::default(), false)
        .unwrap();
    assert_eq!((first, last), (0., 1.));
    let (curve, _, _) = ShapeAnalysisEdge::new()
        .pcurve_on_surface(&edge, &surface, &Location::default(), false)
        .unwrap();
    assert_eq!(curve.value(0.).unwrap().x(), 2e-8);
}

#[test]
fn vertex_repair_on_face_and_removed_context_match_native_paths() {
    let (edge, surface) = edge_with_pcurve(true, 0.2);
    let mut face = rustycad_tkbrep::topods::Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    let mut fixer = ShapeFixEdge::new();
    assert!(fixer.fix_vertex_tolerance_on_face(&edge, &face).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Done2));
    assert!(!fixer.fix_vertex_tolerance_on_face(&edge, &face).unwrap());
    let context = Handle::new(ShapeBuildReShape::new());
    context.remove(&edge).unwrap();
    fixer.set_context(&context);
    assert!(!fixer.fix_vertex_tolerance(&edge).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Ok));
}
