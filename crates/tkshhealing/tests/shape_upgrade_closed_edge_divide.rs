//! Native sampling and state changes from tkshhealing_closed_edge_divide.cxx.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis2, Axis2d, Axis3, Point3};
use rustycad_tkshhealing::{
    shape_build::ShapeBuildEdge,
    shape_upgrade::{
        EdgeDivider, EdgeSplitKnots, ShapeUpgradeClosedEdgeDivide, ShapeUpgradeEdgeDivide,
        SplitCurve,
    },
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn circle_edge() -> Edge {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 1.).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
}
fn add_pcurve(edge: &Edge) -> Face {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::from_axis(Axis2d::default(), 1., true).unwrap()).cast();
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &plane, 1e-7)
        .unwrap();
    builder
        .update_edge_pcurve_on_face(edge, &curve, &face, 1e-7)
        .unwrap();
    builder
        .range_on_face(edge, &face, 0., std::f64::consts::TAU)
        .unwrap();
    face
}
fn knot_row(knots: EdgeSplitKnots) -> Vec<f64> {
    if knots.is_null() {
        return vec![0.];
    }
    knots.sequence(|s| {
        std::iter::once(s.len() as f64)
            .chain(s.iter().copied())
            .collect()
    })
}
fn observed_row(divider: &ShapeUpgradeClosedEdgeDivide, edge: &Edge) -> Vec<u64> {
    let changed = divider.compute(edge).unwrap();
    let mut actual = vec![
        f64::from(changed),
        f64::from(divider.has_curve_3d()),
        f64::from(divider.has_curve_2d()),
    ];
    actual.extend(knot_row(divider.knots_3d()));
    actual.extend(knot_row(divider.knots_2d()));
    actual.iter().map(|x| x.to_bits()).collect()
}
fn expected_row(name: &str) -> Vec<u64> {
    let expected = include_str!("../../../tools/oracle/tkshhealing_closed_edge_divide.expected");
    let row = expected
        .lines()
        .find(|s| s.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect();
    expected.iter().map(|x| x.to_bits()).collect()
}
fn assert_native_row(name: &str, divider: &ShapeUpgradeClosedEdgeDivide, edge: &Edge) {
    let actual = observed_row(divider, edge);
    let expected = expected_row(name);
    assert_eq!(actual, expected, "{name}");
}
#[test]
fn closed_edge_sampling_and_tolerance_match_native() {
    let edge = circle_edge();
    let divider = ShapeUpgradeClosedEdgeDivide::new();
    assert_native_row("free-circle", &divider, &edge);
    divider.set_face(&add_pcurve(&edge));
    assert_native_row("both-curves", &divider, &edge);
    let builder = BRepBuilder::default();
    builder
        .update_vertex_tolerance(&topexp::first_vertex(&edge, false), 3.)
        .unwrap();
    divider.set_max_tolerance(10.);
    assert_native_row("large-tolerance", &divider, &edge);
    divider.set_max_tolerance(1.);
    assert_native_row("clamped-tolerance", &divider, &edge);
}
#[test]
fn planar_only_and_unsplittable_edges_match_native() {
    let edge = circle_edge();
    let divider = ShapeUpgradeClosedEdgeDivide::new();
    let builder = BRepBuilder::default();
    divider.set_face(&add_pcurve(&edge));
    ShapeBuildEdge.remove_curve_3d(&edge).unwrap();
    assert_native_row("pcurve-only", &divider, &edge);
    builder.degenerated(&edge, true).unwrap();
    assert_native_row("degenerated", &divider, &edge);
    let line = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(1., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    assert_native_row("open-edge", &divider, &line);
}
#[test]
fn dynamic_criterion_shares_native_parent_and_tool_parameter_handles() {
    let divider = Handle::new(ShapeUpgradeClosedEdgeDivide::new());
    let base = divider.cast::<ShapeUpgradeEdgeDivide>();
    let criterion = divider.cast::<dyn EdgeDivider>();
    assert!(!base.is_null());
    assert!(criterion.compute(&circle_edge()).unwrap());
    let knots = base.knots_3d();
    assert_eq!(
        knots.object_address(),
        base.split_curve_3d_tool().split_values().object_address()
    );
    assert_eq!(knots.sequence(|s| s.len()), 3);
    divider.clear();
    assert!(base.knots_3d().is_null());
    assert!(!base.has_curve_3d());
    assert_eq!(knots.sequence(|s| s.len()), 3);
}
