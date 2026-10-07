use super::*;
use crate::chfi_ds::{SurfData, shared};
use rustycad_tkg3d::geom::GeomLine;
use rustycad_tkmath::gp::Direction3;

fn line_curve() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::from_point(
        Point3::default(),
        Direction3::new(0., 0., 1.).unwrap(),
    ))
    .cast()
}

/// A periodic quadratic B-spline with four poles on the unit square, knots 0..4.
fn periodic_bspline() -> Handle<dyn GeomCurve> {
    let mut pole_buffer = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 0.),
        Point3::new(0., 1., 0.),
    ];
    let mut knot_buffer = [0., 1., 2., 3., 4.];
    let mut mult_buffer = [1; 5];
    let poles = Array1::from_slice(&mut pole_buffer);
    let knots = Array1::from_slice(&mut knot_buffer);
    let mults = Array1::from_slice(&mut mult_buffer);
    Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 2, true).unwrap()).cast()
}

#[test]
fn default_spine_has_infinite_saved_bounds() {
    let spine = ElSpine::new();
    assert_eq!(spine.saved_first_parameter(), precision::infinite());
    spine.set_first_parameter(0.25);
    spine.save_first_parameter();
    spine.set_last_parameter(2.0);
    spine.save_last_parameter();
    assert_eq!(spine.saved_first_parameter(), 0.25);
    assert_eq!(spine.saved_last_parameter(), 2.0);
}

#[test]
fn period_raises_standard_failure_when_not_periodic() {
    let spine = ElSpine::new();
    let failure = spine.period().unwrap_err();
    assert_eq!(failure.message(), "ElSpine non periodique");
    assert_eq!(
        spine.set_origin(0.0).unwrap_err().message(),
        "Elspine non periodique"
    );
}

#[test]
fn the_period_is_taken_when_periodicity_is_set() {
    let spine = ElSpine::new();
    spine.set_first_parameter(1.0);
    spine.set_last_parameter(4.0);
    spine.set_periodic(true);
    spine.set_last_parameter(9.0);
    assert_eq!(spine.period().unwrap(), 3.0);
    assert!(spine.is_periodic().unwrap());
}

#[test]
fn geometry_comes_from_the_loaded_curve_and_bounds_are_own() {
    let spine = ElSpine::new();
    spine.set_curve(&line_curve()).unwrap();
    spine.set_first_parameter(-1.0);
    spine.set_last_parameter(1.0);
    assert_eq!(spine.get_type().unwrap(), CurveType::Line);
    assert_eq!(spine.value(2.0).unwrap(), Point3::new(0., 0., 2.));
    assert_eq!(spine.first_parameter().unwrap(), -1.0);
    assert!(spine.eval_dn(0.0, 1).is_err());
    assert!(spine.is_closed().is_err());
}

#[test]
fn set_origin_moves_the_origin_of_a_periodic_bspline() {
    let spine = ElSpine::new();
    spine.set_curve(&periodic_bspline()).unwrap();
    let before = spine.value(1.5).unwrap();
    spine.set_periodic(true);
    spine.set_origin(1.5).unwrap();
    let bspline = spine.bspline().unwrap();
    assert_eq!(bspline.get().unwrap().first_parameter(), 1.5);
    assert!(spine.value(1.5).unwrap().distance(&before) < 1e-12);
}

#[test]
fn shallow_copy_keeps_fields_and_shares_the_curve() {
    let spine = ElSpine::new();
    spine.set_curve(&line_curve()).unwrap();
    spine.add_vertex_with_tangent(Axis1::default());
    spine.change_previous(Some(shared(SurfData::new())));
    spine.set_first_point_and_tgt(Point3::new(1., 2., 3.), Vector3::new(0., 0., 1.));
    let copy = spine.shallow_spine().unwrap();
    assert_eq!(copy.nb_vertices(), 1);
    assert!(copy.previous().is_some() && copy.next().is_none());
    assert_eq!(copy.first_point_and_tgt().0, Point3::new(1., 2., 3.));
    assert_eq!(copy.vertex_with_tangent(1).unwrap(), Axis1::default());
    assert_eq!(
        copy.vertex_with_tangent(2).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
