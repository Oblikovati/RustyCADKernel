//! Curve removal, reconstruction and orientation repair from ShapeFix_Edge.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Direction2, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkshhealing::shape_build::ShapeBuildEdge;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

#[test]
fn projector_is_shared_and_initialized() {
    let fixer = ShapeFixEdge::new();
    assert!(!fixer.projector().is_null());
    fixer.projector().set_adjust_over_degen_mode(0);
    assert_eq!(fixer.projector().adjust_over_degen_mode(), 0);
}

#[test]
fn removing_inconsistent_curves_keeps_native_ok_status() {
    let (edge, face) = cylinder_fixture(false);
    let fixer = ShapeFixEdge::new();
    assert!(!fixer.fix_remove_pcurve(&edge, &face).unwrap());
    assert!(!fixer.fix_remove_curve_3d(&edge).unwrap());
    let vertex = ShapeAnalysisEdge::new().first_vertex(&edge);
    BRepBuilder::default()
        .update_vertex_point(&vertex, Point3::new(9., 0., 0.), 1e-7)
        .unwrap();
    assert!(fixer.fix_remove_pcurve(&edge, &face).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Ok));
    assert!(
        !ShapeAnalysisEdge::new()
            .has_pcurve_on_face(&edge, &face)
            .unwrap()
    );
    assert!(fixer.fix_remove_curve_3d(&edge).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Ok));
    assert!(BRepTool::curve(&edge).unwrap().0.is_null());
}

#[test]
fn rebuilding_missing_curve_sets_done_or_fail() {
    let (edge, face) = cylinder_fixture(false);
    let fixer = ShapeFixEdge::new();
    assert!(!fixer.fix_add_curve_3d(&edge).unwrap());
    ShapeBuildEdge.remove_curve_3d(&edge).unwrap();
    assert!(fixer.fix_add_curve_3d(&edge).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Done1));
    let curve = BRepTool::curve(&edge).unwrap().0;
    assert!(curve.value(0.).unwrap().distance(&Point3::new(1., 0., 0.)) < 1e-10);
    ShapeBuildEdge.remove_curve_3d(&edge).unwrap();
    ShapeBuildEdge.remove_pcurve_on_face(&edge, &face).unwrap();
    assert!(!fixer.fix_add_curve_3d(&edge).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Fail1));
}

#[test]
fn cloned_handle_observes_shared_status_after_curve_repair() {
    let (edge, _) = cylinder_fixture(false);
    ShapeBuildEdge.remove_curve_3d(&edge).unwrap();
    let fixer = Handle::new(ShapeFixEdge::new());
    let alias = fixer.clone();

    assert!(fixer.get().unwrap().fix_add_curve_3d(&edge).unwrap());
    assert!(alias.get().unwrap().status_flag(ShapeExtendStatus::Done1));
}

#[test]
fn reversed_cylinder_pcurve_is_mutated_in_place_and_clears_range_flags() {
    let (edge, face) = cylinder_fixture(true);
    let curve = BRepTool::curve_on_face(&edge, &face).unwrap().pcurve;
    let fixer = ShapeFixEdge::new();
    assert!(fixer.fix_reversed_2d(&edge, &face).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Done1));
    let repaired = BRepTool::curve_on_face(&edge, &face).unwrap();
    assert!(curve == repaired.pcurve);
    assert_eq!((repaired.first, repaired.last), (-1., 0.));
    assert_eq!(curve.value(-1.).unwrap(), Point2::new(0., 0.));
    assert!(!BRepTool::same_range(&edge).unwrap());
    assert!(!BRepTool::same_parameter(&edge).unwrap());
    assert!(!fixer.fix_reversed_2d(&edge, &face).unwrap());
}

#[test]
fn missing_pcurve_reports_native_reversal_failure() {
    let (edge, face) = cylinder_fixture(false);
    ShapeBuildEdge.remove_pcurve_on_face(&edge, &face).unwrap();
    let fixer = ShapeFixEdge::new();
    assert!(!fixer.fix_reversed_2d(&edge, &face).unwrap());
    assert!(fixer.status_flag(ShapeExtendStatus::Fail1));
}

fn cylinder_fixture(reversed: bool) -> (Edge, Face) {
    let edge = BRepLibMakeEdge::from_points(Point3::new(1., 0., 0.), Point3::new(1., 0., 1.))
        .unwrap()
        .edge()
        .unwrap();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.).unwrap()).cast();
    let (v, dy) = if reversed { (1., -1.) } else { (0., 1.) };
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0., v),
        Direction2::new(0., dy).unwrap(),
    ))
    .cast();
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, &face, 1e-7)
        .unwrap();
    builder
        .range_on_surface(&edge, &surface, &Location::default(), 0., 1.)
        .unwrap();
    (edge, face)
}
