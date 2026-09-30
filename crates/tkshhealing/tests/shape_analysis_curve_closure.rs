//! OCC966 preserves periodicity through trimmed wrappers.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkmath::gp::{Axis2, Axis22d};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCurve;

fn circle_basis() -> Handle<dyn GeomCurve> {
    Handle::new(GeomCircle::new(Axis2::default(), 2.0).unwrap()).cast()
}
#[test]
fn partial_circle_is_open_but_underlying_curve_is_periodic() {
    let basis = circle_basis();
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&basis, 0.0, 1.0, true, true).unwrap()).cast();
    assert!(!trimmed.is_periodic());
    assert!(ShapeAnalysisCurve::is_periodic(&trimmed).unwrap());
    assert!(!ShapeAnalysisCurve::is_closed(&trimmed, 1e-7).unwrap());
    assert!(ShapeAnalysisCurve::is_closed(&trimmed, 10.0).unwrap());
    assert!(ShapeAnalysisCurve::is_closed(&basis, -1.0).unwrap());
}
#[test]
fn pcurve_periodicity_unwraps_partial_circle() {
    let basis: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::new(Axis22d::default(), 2.0).unwrap()).cast();
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&basis, 0.0, 1.0, true, true).unwrap()).cast();
    assert!(!trimmed.is_periodic());
    assert!(ShapeAnalysisCurve::is_periodic_2d(&trimmed).unwrap());
}
#[test]
fn null_curves_return_explicit_errors() {
    assert!(ShapeAnalysisCurve::is_periodic(&Handle::default()).is_err());
    assert!(ShapeAnalysisCurve::is_periodic_2d(&Handle::default()).is_err());
    assert!(ShapeAnalysisCurve::is_closed(&Handle::default(), 1e-7).is_err());
}
