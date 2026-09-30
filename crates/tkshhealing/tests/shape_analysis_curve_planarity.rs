//! Native normal selection and supplied-normal tolerance rules.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkmath::gp::{Axis2, Point3, Xyz};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCurve;
#[test]
fn planar_polygon_produces_unit_normal() {
    let points = [
        Point3::default(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let mut normal = Xyz::default();
    assert!(
        ShapeAnalysisCurve::is_planar_points(&Array1::from_shared(&points, 1), &mut normal, 1e-7)
            .unwrap()
    );
    assert_eq!(normal, Xyz::new(0.0, 0.0, 1.0));
}
#[test]
fn two_points_check_original_unnormalized_normal_with_confusion() {
    let points = [Point3::default(), Point3::new(1.0, 0.0, 0.0)];
    let mut normal = Xyz::new(1.0, 0.0, 0.0);
    assert!(
        !ShapeAnalysisCurve::is_planar_points(&Array1::from_shared(&points, 1), &mut normal, 100.0)
            .unwrap()
    );
    assert_eq!(normal, Xyz::new(1.0, 0.0, 0.0));
}
#[test]
fn tiny_supplied_normal_falls_back_to_z_without_rechecking_points() {
    let points = [
        Point3::default(),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 2.0),
    ];
    let mut normal = Xyz::new(1e-9, 0.0, 0.0);
    assert!(
        ShapeAnalysisCurve::is_planar_points(&Array1::from_shared(&points, 1), &mut normal, 1e-7)
            .unwrap()
    );
    assert_eq!(normal, Xyz::new(0.0, 0.0, 1.0));
}
#[test]
fn partial_circle_uses_basis_plane_and_preserves_supplied_normal_scale() {
    let circle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 2.0).unwrap()).cast();
    let trimmed = Handle::new(GeomTrimmedCurve::new(&circle, 0.0, 1.0, true, true).unwrap()).cast();
    let mut normal = Xyz::new(0.0, 0.0, 3.0);
    assert!(ShapeAnalysisCurve::is_planar(&trimmed, &mut normal, 1e-7).unwrap());
    assert_eq!(normal, Xyz::new(0.0, 0.0, 3.0));
}
