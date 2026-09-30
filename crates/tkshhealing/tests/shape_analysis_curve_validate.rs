//! Range validation can mutate the caller's shared curve as well as parameters.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCurve;
fn validation_line() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).cast()
}
#[test]
fn descending_line_range_reverses_shared_geometry_and_returns_false() {
    let curve = validation_line();
    let alias = curve.clone();
    let (mut first, mut last) = (3.0, 1.0);
    assert!(
        !ShapeAnalysisCurve
            .validate_range(&curve, &mut first, &mut last, 1e-7)
            .unwrap()
    );
    assert_eq!((first, last), (-3.0, -1.0));
    assert_eq!(alias.value(1.0).unwrap(), Point3::new(-1.0, 0.0, 0.0));
}
#[test]
fn equal_line_range_expands_by_parametric_confusion() {
    let (mut first, mut last) = (0.0, 0.0);
    assert!(
        !ShapeAnalysisCurve
            .validate_range(&validation_line(), &mut first, &mut last, 1e-7)
            .unwrap()
    );
    assert_eq!((first, last), (-1e-9, 1e-9));
}
#[test]
fn bounded_curve_clamps_before_accepting_ordered_range() {
    let coordinates = [Point3::default(), Point3::new(1.0, 2.0, 0.0)];
    let poles = Array1::from_shared(&coordinates, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    let (mut first, mut last) = (-3.0, 7.0);
    assert!(
        ShapeAnalysisCurve
            .validate_range(&curve, &mut first, &mut last, 1e-7)
            .unwrap()
    );
    assert_eq!((first, last), (0.0, 1.0));
}
