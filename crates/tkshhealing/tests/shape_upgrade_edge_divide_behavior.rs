//! Curve discovery and split-state behavior over real builder-created geometry.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{self, Edge, Face},
};
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::{
    gp::{Axis1, Axis3, Point2, Point3, dx},
    toploc::Location,
};
use rustycad_tkshhealing::{
    shape_build::ShapeBuildEdge,
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{
        EdgeDivider, EdgeSplitKnots, FixSmallCurves, ShapeUpgradeEdgeDivide,
        ShapeUpgradeFixSmallCurves, SplitCurve,
    },
};
#[path = "divide_support/criteria.rs"]
mod criteria;
use criteria::{Midpoint2d, Midpoint3d};

fn spatial_line() -> Handle<dyn GeomCurve> {
    let axis = Axis1::new(Point3::default(), dx());
    let line = GeomLine::new(axis);
    Handle::new(line).cast()
}
fn reparameterized_pcurve() -> Handle<dyn Geom2dCurve> {
    let points = [Point2::new(2., 0.), Point2::new(5., 0.)];
    let poles = Array1::from_shared(&points, 1);
    let knots = Array1::from_shared(&[10., 16.], 1);
    let mults = Array1::from_shared(&[2, 2], 1);
    Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false).unwrap()).cast()
}
fn edge_with_parameterizations() -> (Edge, Face) {
    let mut edge = Edge::default();
    ShapeBuildEdge
        .make_edge_on_curve_range(&mut edge, &spatial_line(), &Location::default(), 2., 5.)
        .unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    let builder = BRepBuilder::default();
    builder
        .make_face_on_surface(&mut face, &plane, 1e-7)
        .unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &reparameterized_pcurve(), &face, 0.)
        .unwrap();
    builder.range_on_face(&edge, &face, 10., 16.).unwrap();
    (edge, face)
}
fn knot_values(knots: &EdgeSplitKnots) -> Vec<f64> {
    assert!(!knots.is_null());
    knots.sequence(|sequence| sequence.iter().copied().collect())
}
fn midpoint_divider(face: &Face) -> ShapeUpgradeEdgeDivide {
    let divider = ShapeUpgradeEdgeDivide::new();
    divider.set_face(face);
    divider.set_split_curve_3d_tool(&Handle::new(Midpoint3d::default()).cast());
    divider.set_split_curve_2d_tool(&Handle::new(Midpoint2d::default()).cast());
    divider
}
#[test]
fn discovers_independent_parameterizations_without_modifying_geometry() {
    let (edge, face) = edge_with_parameterizations();
    let divider = ShapeUpgradeEdgeDivide::new();
    let original = BRepTool::curve(&edge).unwrap().0;
    divider.set_face(&face);
    assert!(!divider.compute(&edge).unwrap());
    assert!(divider.has_curve_3d());
    assert!(divider.has_curve_2d());
    assert_eq!(knot_values(&divider.knots_3d()), vec![2., 5.]);
    assert_eq!(knot_values(&divider.knots_2d()), vec![10., 16.]);
    assert!(BRepTool::curve(&edge).unwrap().0 == original);
    assert_eq!(original.value(2.).unwrap(), Point3::new(2., 0., 0.));
    assert_eq!(
        reparameterized_pcurve().value(16.).unwrap(),
        Point2::new(5., 0.)
    );
}
#[test]
fn derived_compute_splits_both_parameterizations_at_matching_geometry() {
    let (edge, face) = edge_with_parameterizations();
    let divider = midpoint_divider(&face);
    assert!(divider.compute(&edge).unwrap());
    assert_eq!(knot_values(&divider.knots_3d()), vec![2., 3.5, 5.]);
    assert_eq!(knot_values(&divider.knots_2d()), vec![10., 13., 16.]);
    let spatial = divider.split_curve_3d_tool();
    let planar = divider.split_curve_2d_tool();
    spatial.cast::<dyn SplitCurve>().build(true).unwrap();
    planar.cast::<dyn SplitCurve>().build(true).unwrap();
    assert_eq!(spatial.curves().array1().len(), 2);
    assert_eq!(planar.curves().array1().len(), 2);
    let first_spatial = spatial.curves().array1().value(1).unwrap().clone();
    let first_planar = planar.curves().array1().value(1).unwrap().clone();
    assert_eq!(first_spatial.value(3.5).unwrap(), Point3::new(3.5, 0., 0.));
    assert_eq!(first_planar.value(13.).unwrap(), Point2::new(3.5, 0.));
}
#[test]
fn exposed_knots_share_tool_state_and_clear_retains_previous_handles() {
    let (edge, face) = edge_with_parameterizations();
    let divider = midpoint_divider(&face);
    divider.compute(&edge).unwrap();
    let spatial = divider.knots_3d();
    let planar = divider.knots_2d();
    assert!(spatial == divider.split_curve_3d_tool().split_values());
    assert!(planar == divider.split_curve_2d_tool().split_values());
    spatial.change_sequence(|s| s.set_value(2, 3.25)).unwrap();
    assert_eq!(
        knot_values(&divider.split_curve_3d_tool().split_values()),
        vec![2., 3.25, 5.]
    );
    divider.clear();
    assert!(!divider.has_curve_3d());
    assert!(!divider.has_curve_2d());
    assert!(divider.knots_3d().is_null());
    assert!(divider.knots_2d().is_null());
    assert_eq!(knot_values(&spatial), vec![2., 3.25, 5.]);
    assert_eq!(knot_values(&planar), vec![10., 13., 16.]);
    assert!(divider.edge_divide_data().with(|f| f.face.is_same(&face)));
}
#[test]
fn pcurve_only_edge_still_dispatches_the_planar_criterion() {
    let (edge, face) = edge_with_parameterizations();
    ShapeBuildEdge.remove_curve_3d(&edge).unwrap();
    let divider = midpoint_divider(&face);
    assert!(divider.compute(&edge).unwrap());
    assert!(!divider.has_curve_3d());
    assert!(divider.knots_3d().is_null());
    assert!(divider.has_curve_2d());
    assert_eq!(knot_values(&divider.knots_2d()), vec![10., 13., 16.]);
}
#[test]
fn free_edge_preserves_unused_pcurve_tool_status() {
    let (edge, _) = edge_with_parameterizations();
    let divider = ShapeUpgradeEdgeDivide::new();
    let planar = divider.split_curve_2d_tool();
    planar.split_curve_data().with_mut(|f| f.status = 2);
    assert!(divider.compute(&edge).unwrap());
    assert!(!divider.has_curve_2d());
    assert!(divider.knots_2d().is_null());
    assert!(planar.status_flag(ShapeExtendStatus::Done2));
    divider.clear();
    assert!(planar.status_flag(ShapeExtendStatus::Done2));
}
#[test]
fn reversed_edges_keep_native_unoriented_split_ranges() {
    let (edge, face) = edge_with_parameterizations();
    let reversed = topods::edge(&edge.reversed()).unwrap();
    let divider = midpoint_divider(&face);
    assert!(divider.compute(&reversed).unwrap());
    assert_eq!(knot_values(&divider.knots_3d()), vec![2., 3.5, 5.]);
    assert_eq!(knot_values(&divider.knots_2d()), vec![10., 13., 16.]);
}
#[test]
fn repeated_compute_replaces_knots_and_keeps_inherited_tolerances() {
    let (edge, face) = edge_with_parameterizations();
    let divider = midpoint_divider(&face);
    divider.set_precision(2e-6);
    divider.compute(&edge).unwrap();
    let previous = divider.knots_2d();
    divider.compute(&edge).unwrap();
    assert!(previous != divider.knots_2d());
    assert_eq!(knot_values(&previous), vec![10., 13., 16.]);
    assert_eq!(divider.precision(), 2e-6);
}
#[test]
fn empty_builder_edge_has_no_geometry_or_split_parameters() {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    let divider = ShapeUpgradeEdgeDivide::new();
    assert!(!divider.compute(&edge).unwrap());
    assert!(!divider.has_curve_3d());
    assert!(!divider.has_curve_2d());
    assert!(divider.knots_3d().is_null());
    assert!(divider.knots_2d().is_null());
}
#[test]
fn small_curve_base_preserves_geometry_handles_and_parameter_bits() {
    let tool = small_tool_on_plane();
    let mut spatial = spatial_line();
    let mut planar = reparameterized_pcurve();
    let mut reverse = planar.clone();
    let original = (spatial.clone(), planar.clone(), reverse.clone());
    let (mut first, mut last) = (-0.0, 16.);
    let changed = tool
        .approx(
            &mut spatial,
            &mut planar,
            &mut reverse,
            &mut first,
            &mut last,
        )
        .unwrap();
    assert!(!changed);
    assert!((spatial, planar, reverse) == original);
    assert_eq!(first.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(last, 16.);
    assert!(tool.status_flag(ShapeExtendStatus::Ok));
}
#[test]
fn small_curve_init_retains_status_and_assigned_splitting_tools() {
    let (edge, face) = edge_with_parameterizations();
    let tool = ShapeUpgradeFixSmallCurves::new();
    assert!(tool.split_curve_3d_tool().is_null());
    assert!(tool.split_curve_2d_tool().is_null());
    let divider = midpoint_divider(&face);
    let spatial = divider.split_curve_3d_tool();
    let planar = divider.split_curve_2d_tool();
    tool.set_split_curve_3d_tool(&spatial);
    tool.set_split_curve_2d_tool(&planar);
    tool.fix_small_curves_data().with_mut(|f| f.status = 4);
    tool.init(&edge, &face);
    tool.set_precision(3e-6);
    assert!(tool.status_flag(ShapeExtendStatus::Done3));
    assert_eq!(tool.precision(), 3e-6);
    assert!(tool.split_curve_3d_tool() == spatial);
    assert!(tool.split_curve_2d_tool() == planar);
    assert!(
        tool.fix_small_curves_data()
            .with(|f| f.edge.is_same(&edge) && f.face.is_same(&face))
    );
}

fn small_tool_on_plane() -> ShapeUpgradeFixSmallCurves {
    let (edge, face) = edge_with_parameterizations();
    let tool = ShapeUpgradeFixSmallCurves::new();
    tool.init(&edge, &face);
    tool
}
