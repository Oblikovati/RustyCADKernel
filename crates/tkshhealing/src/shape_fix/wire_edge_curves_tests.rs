use super::super::ShapeFixWire;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_extend::ShapeExtendStatus;
use crate::shape_fix::FixRoot;
use crate::shape_fix::ShapeFixEdge;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Cylinder3, Point3, Sphere3, StandardDirection3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire};
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

#[test]
fn unloaded_and_disabled_modes_match_native_status() {
    let unloaded = ShapeFixWire::new();
    assert!(!unloaded.fix_edge_curves().unwrap());
    assert!(unloaded.status_edge_curves(ShapeExtendStatus::Ok));
    let (wire, face, edge) = cylinder_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    disable_other_curve_passes(&fixer);
    fixer.set_fix_add_pcurve_mode(0);
    assert!(!fixer.fix_edge_curves().unwrap());
    assert!(
        !ShapeAnalysisEdge::new()
            .has_pcurve_on_face(&edge, &face)
            .unwrap()
    );
    assert!(fixer.status_edge_curves(ShapeExtendStatus::Ok));
}

#[test]
fn add_pcurve_only_matches_native_fixture() {
    let (wire, face, edge) = cylinder_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    disable_other_curve_passes(&fixer);
    fixer.set_fix_add_pcurve_mode(1);
    let fixed = fixer.fix_edge_curves().unwrap();
    assert!(fixed);
    assert!(fixer.status_edge_curves(ShapeExtendStatus::Done3));
    assert!(!fixer.status_edge_curves(ShapeExtendStatus::Fail3));
    assert!(
        ShapeAnalysisEdge::new()
            .has_pcurve_on_face(&edge, &face)
            .unwrap()
    );
}

#[test]
fn default_modes_keep_native_failure_and_success_flags() {
    let (wire, face, edge) = cylinder_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    assert!(fixer.fix_edge_curves().unwrap());
    assert_default_status_flags(&fixer);
    assert!(
        ShapeAnalysisEdge::new()
            .has_pcurve_on_face(&edge, &face)
            .unwrap()
    );
}

#[test]
fn failed_curve_rebuild_removes_incomplete_edge_and_keeps_fail_flag() {
    let (wire, _, edge) = cylinder_wire();
    ShapeBuildEdge.remove_curve_3d(&edge).unwrap();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();
    disable_other_curve_passes(&fixer);
    fixer.set_fix_add_curve_3d_mode(1);
    assert!(fixer.fix_edge_curves().unwrap());
    assert_eq!(fixer.nb_edges(), 0);
    assert!(fixer.status_edge_curves(ShapeExtendStatus::Done5));
    assert!(fixer.status_edge_curves(ShapeExtendStatus::Fail5));
}

#[test]
fn add_pcurve_splits_edge_at_spherical_pole_like_native() {
    let (wire, face) = sphere_pole_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    disable_other_curve_passes(&fixer);
    fixer.set_fix_add_pcurve_mode(1);
    assert!(fixer.fix_edge_curves().unwrap());
    assert_eq!(fixer.nb_edges(), 2);
    assert_spherical_split_ranges(&fixer, &face);
    assert!(fixer.status_edge_curves(ShapeExtendStatus::Done3));
    assert!(!fixer.status_edge_curves(ShapeExtendStatus::Fail3));
}

fn assert_spherical_split_ranges(fixer: &ShapeFixWire, face: &Face) {
    let first_edge = fixer.wire_data().unwrap().edge(1).unwrap();
    let second_edge = fixer.wire_data().unwrap().edge(2).unwrap();
    assert_edge_range(&first_edge, 5.783_185_307_179_586, std::f64::consts::TAU);
    assert_edge_range(&second_edge, std::f64::consts::TAU, 6.783_185_307_179_586);
    let analysis = ShapeAnalysisEdge::new();
    assert_pcurve_range(
        &analysis,
        &first_edge,
        face,
        5.783_185_307_179_586,
        std::f64::consts::TAU,
    );
    assert_pcurve_range(
        &analysis,
        &second_edge,
        face,
        std::f64::consts::TAU,
        6.783_185_307_179_586,
    );
}

fn assert_pcurve_range(
    analysis: &ShapeAnalysisEdge,
    edge: &Edge,
    face: &Face,
    expected_first: f64,
    expected_last: f64,
) {
    let (curve, first, last) = analysis.pcurve_on_face(edge, face, true).unwrap();
    assert!(!curve.is_null());
    assert!((first - expected_first).abs() < 1e-12);
    assert!((last - expected_last).abs() < 1e-12);
}

fn assert_edge_range(edge: &Edge, expected_first: f64, expected_last: f64) {
    let (first, last) = rustycad_tkbrep::brep::BRepTool::range(edge).unwrap();
    assert!((first - expected_first).abs() < 1e-12);
    assert!((last - expected_last).abs() < 1e-12);
}

fn sphere_pole_wire() -> (Wire, Face) {
    let sphere = Sphere3::new(Axis3::default(), 1.0).unwrap();
    let face = BRepBuilderAPIMakeFace::from_sphere_bounds(
        sphere,
        (
            0.0,
            std::f64::consts::TAU,
            -std::f64::consts::FRAC_PI_2,
            std::f64::consts::FRAC_PI_2,
        ),
    )
    .unwrap()
    .face()
    .unwrap();
    let circle_axis = Axis2::with_standard_frame(
        Point3::new(0.0, 0.0, 0.0),
        StandardDirection3::X,
        StandardDirection3::Z,
    );
    let circle = Circle3::new(circle_axis, 1.0).unwrap();
    let edge = BRepLibMakeEdge::from_circ_params(circle, -0.5, 0.5)
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    (wire, face)
}

fn assert_default_status_flags(fixer: &ShapeFixWire) {
    assert!(fixer.status_edge_curves(ShapeExtendStatus::Done3));
    assert!(fixer.status_edge_curves(ShapeExtendStatus::Fail1));
    assert_unused_status_flags(fixer);
}

fn assert_unused_status_flags(fixer: &ShapeFixWire) {
    for status in unused_status_flags() {
        assert!(!fixer.status_edge_curves(status));
    }
}

fn unused_status_flags() -> [ShapeExtendStatus; 14] {
    [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done4,
        ShapeExtendStatus::Done5,
        ShapeExtendStatus::Done6,
        ShapeExtendStatus::Done7,
        ShapeExtendStatus::Done8,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
        ShapeExtendStatus::Fail4,
        ShapeExtendStatus::Fail5,
        ShapeExtendStatus::Fail6,
        ShapeExtendStatus::Fail7,
        ShapeExtendStatus::Fail8,
    ]
}

#[test]
fn all_curve_modes_use_native_minus_one_defaults_and_setters() {
    let fixer = ShapeFixWire::new();
    assert_all_defaults_are_minus_one(&fixer);
    set_all_modes(&fixer, 0);
    assert_all_modes_are_zero(&fixer);
}

#[test]
fn plane_projection_and_valid_geometry_removals_match_native_noops() {
    assert_plane_add_pcurve_is_noop();
    assert_valid_pcurve_removal_is_noop();
    assert_valid_curve_removal_is_noop();
}

fn assert_plane_add_pcurve_is_noop() {
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(
        rustycad_tkmath::gp::Plane3::default(),
        (-1.0, 1.0, -1.0, 1.0),
    )
    .unwrap()
    .face()
    .unwrap();
    let edge = BRepLibMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    disable_other_curve_passes(&fixer);
    fixer.set_fix_add_pcurve_mode(1);
    assert!(!fixer.fix_edge_curves().unwrap());
    assert_status_pair(
        &fixer,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail3,
        false,
    );
}

fn assert_valid_pcurve_removal_is_noop() {
    let (wire, face, edge) = cylinder_wire();
    ShapeFixEdge::new()
        .fix_add_pcurve(&edge, &face, false, 1e-7)
        .unwrap();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    disable_other_curve_passes(&fixer);
    fixer.set_fix_add_pcurve_mode(0);
    fixer.set_fix_remove_pcurve_mode(1);
    assert!(!fixer.fix_edge_curves().unwrap());
    assert_status_pair(
        &fixer,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Fail2,
        false,
    );
    assert!(
        ShapeAnalysisEdge::new()
            .has_pcurve_on_face(&edge, &face)
            .unwrap()
    );
}

fn assert_valid_curve_removal_is_noop() {
    let (wire, face, edge) = cylinder_wire();
    let fixer = ShapeFixWire::from_wire_face(&wire, &face, 1e-7).unwrap();
    disable_other_curve_passes(&fixer);
    fixer.set_fix_add_pcurve_mode(0);
    fixer.set_fix_remove_curve_3d_mode(1);
    assert!(!fixer.fix_edge_curves().unwrap());
    assert_status_pair(
        &fixer,
        ShapeExtendStatus::Done4,
        ShapeExtendStatus::Fail4,
        false,
    );
    assert!(!BRepTool::curve(&edge).unwrap().0.is_null());
}

fn assert_status_pair(
    fixer: &ShapeFixWire,
    done: ShapeExtendStatus,
    failed: ShapeExtendStatus,
    expected: bool,
) {
    assert_eq!(fixer.status_edge_curves(done), expected);
    assert_eq!(fixer.status_edge_curves(failed), expected);
}

fn assert_all_defaults_are_minus_one(fixer: &ShapeFixWire) {
    assert_eq!(fixer.fix_edge_curves_mode(), -1);
    assert_eq!(fixer.fix_reversed_2d_mode(), -1);
    assert_eq!(fixer.fix_remove_pcurve_mode(), -1);
    assert_eq!(fixer.fix_add_pcurve_mode(), -1);
    assert_eq!(fixer.fix_remove_curve_3d_mode(), -1);
    assert_eq!(fixer.fix_add_curve_3d_mode(), -1);
    assert_eq!(fixer.fix_seam_mode(), -1);
    assert_eq!(fixer.fix_shifted_mode(), -1);
    assert_eq!(fixer.fix_same_parameter_mode(), -1);
    assert_eq!(fixer.fix_vertex_tolerance_mode(), -1);
}

fn set_all_modes(fixer: &ShapeFixWire, mode: i32) {
    fixer.set_fix_edge_curves_mode(mode);
    fixer.set_fix_reversed_2d_mode(mode);
    fixer.set_fix_remove_pcurve_mode(mode);
    fixer.set_fix_add_pcurve_mode(mode);
    fixer.set_fix_remove_curve_3d_mode(mode);
    fixer.set_fix_add_curve_3d_mode(mode);
    fixer.set_fix_seam_mode(mode);
    fixer.set_fix_shifted_mode(mode);
    fixer.set_fix_same_parameter_mode(mode);
    fixer.set_fix_vertex_tolerance_mode(mode);
}

fn assert_all_modes_are_zero(fixer: &ShapeFixWire) {
    assert_eq!(fixer.fix_edge_curves_mode(), 0);
    assert_eq!(fixer.fix_reversed_2d_mode(), 0);
    assert_eq!(fixer.fix_remove_pcurve_mode(), 0);
    assert_eq!(fixer.fix_add_pcurve_mode(), 0);
    assert_eq!(fixer.fix_remove_curve_3d_mode(), 0);
    assert_eq!(fixer.fix_add_curve_3d_mode(), 0);
    assert_eq!(fixer.fix_seam_mode(), 0);
    assert_eq!(fixer.fix_shifted_mode(), 0);
    assert_eq!(fixer.fix_same_parameter_mode(), 0);
    assert_eq!(fixer.fix_vertex_tolerance_mode(), 0);
}

fn disable_other_curve_passes(fixer: &ShapeFixWire) {
    fixer.set_fix_reversed_2d_mode(0);
    fixer.set_fix_remove_pcurve_mode(0);
    fixer.set_fix_remove_curve_3d_mode(0);
    fixer.set_fix_add_curve_3d_mode(0);
    fixer.set_fix_seam_mode(0);
    fixer.set_fix_shifted_mode(0);
    fixer.set_fix_same_parameter_mode(0);
    fixer.set_fix_vertex_tolerance_mode(0);
}

fn cylinder_wire() -> (Wire, Face, Edge) {
    let cylinder = Cylinder3::new(Axis3::default(), 1.0).unwrap();
    let face = BRepBuilderAPIMakeFace::from_cylinder_bounds(
        cylinder,
        (0.0, std::f64::consts::TAU, 0.0, 2.0),
    )
    .unwrap()
    .face()
    .unwrap();
    let edge = BRepLibMakeEdge::from_points(Point3::new(1., 0., 0.), Point3::new(1., 0., 1.))
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    (wire, face, edge)
}
