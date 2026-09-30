//! Behaviour and preserved quirks of the Adaptor2d and Geom2dAdaptor ports that the
//! legacy suites do not reach; the numeric corpus compares the same paths with OCCT.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{
    Adaptor2dCurve2d, Adaptor2dCurve2dBase, Adaptor2dLine2d, Adaptor2dOffsetCurve,
};
use rustycad_tkg2d::geom2d::offset_curve_utils::{evaluate_d3, evaluate_d3_aliased};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::{Geom2dAdaptorCurve, make_curve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};

fn kind<T>(result: Result<T, Failure>) -> &'static str {
    match result {
        Ok(_) => "ok",
        Err(error) => error.exception_type(),
    }
}

fn circle(radius: f64, sense: bool) -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::new(Point2::new(1.0, -1.0), Direction2::new(0.0, 1.0).unwrap());
    Handle::new(Geom2dCircle::from_axis(axis, radius, sense).unwrap()).cast()
}

// Degree 3 with interior knots of multiplicity 1, 2 and 3.
fn knotted() -> Handle<dyn Geom2dCurve> {
    let mut knots = Array1::<f64>::with_bounds(1, 5).unwrap();
    let mut mults = Array1::<i32>::with_bounds(1, 5).unwrap();
    for (i, (k, m)) in [(0.0, 4), (0.25, 1), (0.5, 2), (0.75, 3), (1.0, 4)]
        .into_iter()
        .enumerate()
    {
        knots[i as i32 + 1] = k;
        mults[i as i32 + 1] = m;
    }
    let mut poles = Array1::<Point2>::with_bounds(1, 10).unwrap();
    for i in 1..=10 {
        poles[i] = Point2::new(f64::from(i), f64::from(i * i % 7));
    }
    Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 3, false).unwrap()).cast()
}

fn geom(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Adaptor2dCurve2d> {
    Handle::new(Geom2dAdaptorCurve::with_curve(curve).unwrap()).cast()
}

#[test]
fn base_class_raises_not_implemented_except_nb_samples() {
    let base = Adaptor2dCurve2dBase::default();
    assert_eq!(kind(base.first_parameter()), "Standard_NotImplemented");
    assert_eq!(kind(base.value(0.0)), "Standard_NotImplemented");
    assert_eq!(kind(base.nb_poles()), "Standard_NotImplemented");
    assert_eq!(base.nb_samples().unwrap(), 20);
    let handle: Handle<dyn Adaptor2dCurve2d> = Handle::new(base).cast();
    assert_eq!(handle.dynamic_type().unwrap().name(), "Adaptor2d_Curve2d");
}

#[test]
fn line_rejects_other_kinds_and_derivative_order_zero() {
    let line = Adaptor2dLine2d::new(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
        0.0,
        1.0,
    );
    assert_eq!(kind(line.period()), "Standard_NoSuchObject");
    assert_eq!(kind(line.circle()), "Standard_NoSuchObject");
    assert_eq!(kind(line.eval_dn(0.0, 0)), "Standard_OutOfRange");
    line.load(line.line().unwrap());
    assert!(line.first_parameter().unwrap() < -1e100 && line.last_parameter().unwrap() > 1e100);
}

#[test]
fn offset_circle_radius_follows_frame_sense_and_flips_when_negative() {
    let direct = Adaptor2dOffsetCurve::with_offset(&geom(&circle(2.0, true)), 0.5).unwrap();
    assert_eq!(direct.circle().unwrap().radius(), 2.5);
    let indirect = Adaptor2dOffsetCurve::with_offset(&geom(&circle(1.5, false)), 0.5).unwrap();
    assert_eq!(indirect.circle().unwrap().radius(), 1.0);
    let flipped = Adaptor2dOffsetCurve::with_offset(&geom(&circle(2.0, true)), -3.0).unwrap();
    let result = flipped.circle().unwrap();
    assert_eq!(result.radius(), 1.0);
    assert_eq!(result.position().x_direction().y(), -1.0);
    let null = Adaptor2dOffsetCurve::with_offset(&geom(&circle(2.0, true)), -2.0).unwrap();
    assert_eq!(kind(null.circle()), "Standard_NoSuchObject");
}

#[test]
fn offset_of_c0_basis_has_no_continuity_and_offset_dn_is_not_implemented() {
    let offset = Adaptor2dOffsetCurve::with_offset(&geom(&knotted()), 0.1).unwrap();
    assert_eq!(kind(offset.continuity()), "Standard_TypeMismatch");
    assert_eq!(kind(offset.eval_dn(0.5, 1)), "Standard_NotImplemented");
    // Offset of an offset: EvalD3 needs the basis EvalDN, which is not implemented.
    let nested = Adaptor2dOffsetCurve::with_offset(&Handle::new(offset).cast(), 0.1).unwrap();
    assert_eq!(kind(nested.eval_d3(0.5)), "Standard_NotImplemented");
}

#[test]
fn offset_intervals_leave_the_basis_array_outside_the_bounds() {
    // Bounds below the first basis knot: the Intervals copy reads T(0).
    let offset = Adaptor2dOffsetCurve::with_bounds(&geom(&knotted()), 0.1, -1.0, 0.2);
    assert_eq!(offset.nb_intervals(Shape::C0).unwrap(), 2);
    let mut bounds = Array1::<f64>::with_bounds(1, 3).unwrap();
    assert_eq!(
        kind(offset.intervals(&mut bounds, Shape::C0)),
        "Standard_OutOfRange"
    );
    // Bounds [0, 0] of the one-argument constructor: the backward scan reaches T(0).
    let zero = Adaptor2dOffsetCurve::with_curve(&geom(&knotted()));
    assert_eq!(kind(zero.nb_intervals(Shape::C0)), "Standard_OutOfRange");
}

#[test]
fn offset_load_stores_the_offset_before_reading_the_basis_bounds() {
    let offset = Adaptor2dOffsetCurve::with_bounds(&geom(&circle(1.0, true)), 1.0, 0.2, 0.4);
    offset.load_curve(&Handle::new(Adaptor2dCurve2dBase::default()).cast());
    assert_eq!(offset.offset(), 0.0);
    assert_eq!(kind(offset.load_offset(3.0)), "Standard_NotImplemented");
    assert_eq!(offset.offset(), 3.0);
    assert_eq!(offset.first_parameter().unwrap(), 0.2);
}

#[test]
fn geom_adaptor_loads_the_basis_of_a_trimmed_curve() {
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&circle(2.0, true), 0.5, 2.5, true, true).unwrap())
            .cast();
    let adaptor = Geom2dAdaptorCurve::with_curve(&trimmed).unwrap();
    assert_eq!(adaptor.get_type().unwrap(), CurveType::Circle);
    assert!(adaptor.curve().is_instance_name("Geom2d_Circle"));
    assert_eq!(
        (
            adaptor.first_parameter().unwrap(),
            adaptor.last_parameter().unwrap()
        ),
        (0.5, 2.5)
    );
}

#[test]
fn geom_adaptor_period_skips_the_periodicity_check() {
    let adaptor = Geom2dAdaptorCurve::with_range(&knotted(), 0.2, 0.4).unwrap();
    assert!(!adaptor.is_periodic().unwrap());
    assert_eq!(adaptor.period().unwrap(), 1.0);
}

#[test]
fn geom_adaptor_continuity_and_intervals_follow_the_knots_in_range() {
    let whole = Geom2dAdaptorCurve::with_curve(&knotted()).unwrap();
    assert_eq!(whole.continuity().unwrap(), Shape::C0);
    // C2 breaks at the knots of multiplicity 2 and 3 only.
    assert_eq!(whole.nb_intervals(Shape::C2).unwrap(), 3);
    assert_eq!(kind(whole.nb_intervals(Shape::G1)), "Standard_DomainError");
    let inner = Geom2dAdaptorCurve::with_range(&knotted(), 0.3, 0.45).unwrap();
    assert_eq!(inner.continuity().unwrap(), Shape::CN);
    let span = Geom2dAdaptorCurve::with_range(&knotted(), 0.25, 0.75).unwrap();
    assert_eq!(span.continuity().unwrap(), Shape::C1);
}

#[test]
fn geom_adaptor_boundary_spans_evaluate_like_the_curve() {
    let curve = knotted();
    let adaptor = Geom2dAdaptorCurve::with_range(&curve, 0.25, 0.75).unwrap();
    let bspline: Handle<Geom2dBSplineCurve> = curve.cast();
    assert_eq!(
        adaptor.eval_d0(0.75).unwrap(),
        bspline.local_d0(0.75, 3, 4).unwrap()
    );
    assert_eq!(
        adaptor.eval_d0(0.25).unwrap(),
        bspline.local_d0(0.25, 2, 3).unwrap()
    );
    let inside = adaptor.eval_d0(0.4).unwrap();
    assert!(inside.distance(curve.value(0.4).unwrap()) < 1e-12);
}

#[test]
fn geom_adaptor_offset_dn3_reads_the_second_derivative_as_the_first() {
    // Regression for the EvaluateDN aliasing quirk (theD1 and theD2 are one vector).
    let offset_curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&circle(2.0, true), 0.5, false).unwrap()).cast();
    let adaptor = Geom2dAdaptorCurve::with_curve(&offset_curve).unwrap();
    let basis = Geom2dAdaptorCurve::with_curve(&circle(2.0, true)).unwrap();
    let dn3 = adaptor.eval_dn(1.0, 3).unwrap();
    assert_eq!(Some(dn3), evaluate_d3_aliased(1.0, &basis, 0.5).unwrap());
    let d3 = evaluate_d3(1.0, &basis, 0.5).unwrap().unwrap().d3;
    assert_eq!(adaptor.eval_d3(1.0).unwrap().d3, d3);
    assert!((dn3 - d3).magnitude() > 1e-3);
}

#[test]
fn make_curve_trims_and_clips_to_the_curve_range() {
    let clipped =
        make_curve(&Geom2dAdaptorCurve::with_range(&knotted(), -0.5, 1.5).unwrap()).unwrap();
    assert!(clipped.is_instance_name("Geom2d_TrimmedCurve"));
    assert_eq!(
        (clipped.first_parameter(), clipped.last_parameter()),
        (0.0, 1.0)
    );
    let same = make_curve(&Geom2dAdaptorCurve::with_curve(&knotted()).unwrap()).unwrap();
    assert!(same.is_instance_name("Geom2d_BSplineCurve"));
    let offset = Adaptor2dOffsetCurve::with_offset(&geom(&knotted()), 0.1).unwrap();
    assert_eq!(kind(make_curve(&offset)), "Standard_DomainError");
    assert_eq!(
        kind(make_curve(&Geom2dAdaptorCurve::default())),
        "Standard_DomainError"
    );
}
