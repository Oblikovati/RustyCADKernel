//! Geom2d_BezierCurve construction, edition and access members.
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2};

fn points(values: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut array = Array1::<Point2>::with_bounds(1, values.len() as i32).unwrap();
    for (i, (x, y)) in values.iter().enumerate() {
        array[i as i32 + 1] = Point2::new(*x, *y);
    }
    array
}

fn reals(lower: i32, values: &[f64]) -> Array1<'static, f64> {
    let mut array = Array1::<f64>::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

const POLES: [(f64, f64); 4] = [(0.0, 0.0), (1.0, 2.0), (3.0, 2.0), (4.0, 0.0)];

fn cubic() -> Handle<Geom2dBezierCurve> {
    Handle::new(Geom2dBezierCurve::new(&points(&POLES)).unwrap())
}

fn rational() -> Handle<Geom2dBezierCurve> {
    let curve = Geom2dBezierCurve::with_weights(&points(&POLES), &reals(1, &[1.0, 2.0, 0.5, 1.0]));
    Handle::new(curve.unwrap())
}

fn near(a: Point2, b: Point2) {
    assert!(a.distance(b) <= 1e-13, "{a:?} != {b:?}");
}

#[test]
fn construction_checks_poles_and_weights() {
    let one = Geom2dBezierCurve::new(&points(&[(0.0, 0.0)]))
        .err()
        .unwrap();
    assert_eq!(one.exception_type(), "Standard_ConstructionError");
    let many: Vec<(f64, f64)> = (0..27).map(|i| (i as f64, 0.0)).collect();
    assert!(Geom2dBezierCurve::new(&points(&many)).is_err());
    assert!(Geom2dBezierCurve::new(&points(&many[..26])).is_ok());
    let p = points(&POLES);
    let short = Geom2dBezierCurve::with_weights(&p, &reals(1, &[1.0, 1.0]))
        .err()
        .unwrap();
    assert_eq!(short.exception_type(), "Standard_ConstructionError");
    let zero = Geom2dBezierCurve::with_weights(&p, &reals(1, &[1.0, 0.0, 1.0, 1.0]))
        .err()
        .unwrap();
    assert_eq!(zero.exception_type(), "Standard_ConstructionError");
    // The native checks read Weights(1..n) whatever the lower bound.
    let shifted = Geom2dBezierCurve::with_weights(&p, &reals(0, &[1.0; 4]))
        .err()
        .unwrap();
    assert_eq!(shifted.exception_type(), "Standard_OutOfRange");
    let equal = Geom2dBezierCurve::with_weights(&p, &reals(1, &[2.0; 4])).unwrap();
    assert!(!equal.is_rational() && equal.weights().is_none());
    assert_eq!(equal.weight(2).unwrap(), 1.0);
}

#[test]
fn queries_and_evaluation() {
    let c = cubic();
    assert_eq!(
        (c.degree(), c.nb_poles(), Geom2dBezierCurve::max_degree()),
        (3, 4, 25)
    );
    assert_eq!((c.first_parameter(), c.last_parameter()), (0.0, 1.0));
    assert!(!c.is_periodic() && !c.is_closed().unwrap() && c.is_cn(7).unwrap());
    assert_eq!(c.continuity(), Shape::CN);
    assert_eq!(
        (c.start_point().unwrap(), c.end_point().unwrap()),
        (Point2::new(0.0, 0.0), Point2::new(4.0, 0.0))
    );
    // B(1/2) = (P0 + 3 P1 + 3 P2 + P3) / 8 = (2, 1.5); B'(1/2) = 3/4 (P3 + P2 - P1 - P0) = (4.5, 0).
    let d3 = c.eval_d3(0.5).unwrap();
    near(d3.point, Point2::new(2.0, 1.5));
    EXPECT_NEAR(d3.d1.x(), 4.5, 1e-14);
    EXPECT_NEAR(d3.d1.y(), 0.0, 1e-14);
    assert_eq!(c.eval_d2(0.5).unwrap().d2, d3.d2);
    assert_eq!(c.eval_dn(0.5, 3).unwrap(), d3.d3);
    assert_eq!(c.eval_dn(0.5, 4).unwrap().magnitude(), 0.0);
    assert_eq!(
        c.eval_dn(0.5, 0).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
    assert_eq!(c.reversed_parameter(0.25).unwrap(), 0.75);
    assert_eq!(c.knots().as_slice(), &[0.0, 1.0]);
    assert_eq!(c.multiplicities().unwrap().as_slice(), &[4, 4]);
    assert_eq!(
        c.knot_sequence().unwrap().as_slice(),
        &[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
    );
    assert_eq!(
        c.pole(5).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        c.weight(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn degree_elevation_and_segment_keep_the_shape() {
    for c in [cubic(), rational()] {
        let reference = Geom2dBezierCurve::copy_of(&c).unwrap();
        c.increase(3).unwrap();
        c.increase(6).unwrap();
        assert_eq!(c.degree(), 6);
        near(c.value(0.3).unwrap(), reference.value(0.3).unwrap());
        assert_eq!(
            c.increase(4).unwrap_err().exception_type(),
            "Standard_ConstructionError"
        );
        c.segment(0.25, 0.75).unwrap();
        near(c.value(0.0).unwrap(), reference.value(0.25).unwrap());
        near(c.value(1.0).unwrap(), reference.value(0.75).unwrap());
        near(c.value(0.5).unwrap(), reference.value(0.5).unwrap());
    }
}

#[test]
fn pole_insertion_and_removal() {
    let c = cubic();
    c.insert_pole_after(2, Point2::new(2.0, 5.0), 1.0).unwrap();
    assert_eq!(
        (c.nb_poles(), c.pole(3).unwrap()),
        (5, Point2::new(2.0, 5.0))
    );
    assert!(!c.is_rational());
    c.insert_pole_before(1, Point2::new(-1.0, 0.0), 3.0)
        .unwrap();
    assert!(c.is_rational());
    assert_eq!((c.weight(1).unwrap(), c.weight(2).unwrap()), (3.0, 1.0));
    c.remove_pole(1).unwrap();
    assert!(!c.is_rational());
    let r = rational();
    r.remove_pole(2).unwrap();
    assert!(r.is_rational());
    assert_eq!((r.nb_poles(), r.weight(2).unwrap()), (3, 0.5));
    assert_eq!(
        r.insert_pole_after(9, Point2::new(0.0, 0.0), 1.0)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        r.insert_pole_after(1, Point2::new(0.0, 0.0), 0.0)
            .unwrap_err()
            .exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        r.remove_pole(4).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let line = Geom2dBezierCurve::new(&points(&[(0.0, 0.0), (1.0, 0.0)])).unwrap();
    assert_eq!(
        line.remove_pole(1).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    // InsertPoleAfter refuses a curve of MaxDegree() poles, one fewer than accepted.
    let full: Vec<(f64, f64)> = (0..25).map(|i| (i as f64, 0.0)).collect();
    let full = Geom2dBezierCurve::new(&points(&full)).unwrap();
    assert!(
        full.insert_pole_after(1, Point2::new(0.0, 1.0), 1.0)
            .is_err()
    );
}

#[test]
fn poles_weights_reverse_and_closure() {
    let c = rational();
    let reference = rational();
    c.reverse().unwrap();
    near(c.value(0.2).unwrap(), reference.value(0.8).unwrap());
    assert_eq!(c.weight(2).unwrap(), 0.5);
    c.set_pole(4, Point2::new(4.0, 0.0)).unwrap();
    assert!(c.is_closed().unwrap());
    c.set_pole_weighted(1, Point2::new(1.0, 1.0), 2.0).unwrap();
    assert!(!c.is_closed().unwrap());
    assert_eq!(
        c.set_pole(0, Point2::new(0.0, 0.0))
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    let n = cubic();
    n.set_weight(2, 1.0).unwrap();
    assert!(!n.is_rational());
    n.set_weight(2, 2.0).unwrap();
    assert!(n.is_rational());
    n.set_weight(2, 1.0).unwrap();
    assert!(!n.is_rational());
    assert_eq!(
        n.set_weight(2, 0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    n.transform(Transform2::translation([1.0, 1.0])).unwrap();
    assert_eq!(n.start_point().unwrap(), Point2::new(1.0, 1.0));
}

#[test]
fn copying_getters_and_resolution() {
    let c = cubic();
    let mut poles = Array1::<Point2>::with_bounds(0, 3).unwrap();
    c.poles_into(&mut poles).unwrap();
    assert_eq!(poles.as_slice(), c.poles().as_slice());
    let mut wrong = Array1::<Point2>::with_bounds(1, 3).unwrap();
    assert_eq!(
        c.poles_into(&mut wrong).unwrap_err().exception_type(),
        "Standard_DimensionError"
    );
    let mut weights = Array1::<f64>::with_bounds(1, 4).unwrap();
    c.weights_into(&mut weights).unwrap();
    assert_eq!(weights.as_slice(), &[1.0; 4]);
    // Non-rational weights are written at 1..n whatever the lower bound.
    let mut shifted = Array1::<f64>::with_bounds(0, 3).unwrap();
    assert_eq!(
        c.weights_into(&mut shifted).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let r = rational();
    r.weights_into(&mut weights).unwrap();
    assert_eq!(weights.as_slice(), &[1.0, 2.0, 0.5, 1.0]);
    r.weights_array(|stored| assert_eq!(stored.as_slice(), &[1.0, 2.0, 0.5, 1.0]));
    let line = Geom2dBezierCurve::new(&points(&[(0.0, 0.0), (1.0, 1.25)])).unwrap();
    // BSplCLib::Resolution: the L1 norm of the pole difference, 2.25 on the unit span.
    EXPECT_NEAR(line.resolution(1e-3).unwrap(), 1e-3 / 2.25, 1e-18);
    EXPECT_NEAR(line.resolution(2e-3).unwrap(), 2e-3 / 2.25, 1e-18);
    let generic: Handle<dyn Geom2dCurve> = r.copy().unwrap().cast();
    assert_eq!(generic.value(0.4).unwrap(), r.value(0.4).unwrap());
}
