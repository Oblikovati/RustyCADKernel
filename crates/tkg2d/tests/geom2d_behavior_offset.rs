//! Geom2d_OffsetCurve and Geom2d_OffsetCurveUtils behavior. Bit patterns come from
//! the native oracle (target/oracle-tkg2d/numeric/native-probe, `g2_offset_utils`).
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::offset_curve_utils::{
    adjust_derivative, calculate_d0, calculate_d1, calculate_d2, calculate_d3, evaluate_d0,
    evaluate_d1, evaluate_d2, evaluate_d3, evaluate_d3_aliased, evaluate_dn,
};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dGeometry, Geom2dLine,
    Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2};
use std::f64::consts::PI;

fn circle(radius: f64) -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Handle::new(Geom2dCircle::from_axis(axis, radius, true).unwrap()).cast()
}

fn poles(values: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut poles = Array1::<Point2>::with_bounds(1, values.len() as i32).unwrap();
    for (i, (x, y)) in values.iter().enumerate() {
        poles[i as i32 + 1] = Point2::new(*x, *y);
    }
    poles
}

// A cubic whose first derivative vanishes at u = 0 (P0 == P1).
fn cusp() -> Handle<dyn Geom2dCurve> {
    let bezier = Geom2dBezierCurve::new(&poles(&[(0.0, 0.0), (0.0, 0.0), (1.0, 1.0), (2.0, 0.0)]));
    Handle::new(bezier.unwrap()).cast()
}

fn offset(basis: &Handle<dyn Geom2dCurve>, distance: f64) -> Handle<Geom2dOffsetCurve> {
    Handle::new(Geom2dOffsetCurve::new(basis, distance, false).unwrap())
}

#[test]
fn offset_circle_is_a_concentric_circle() {
    let curve = offset(&circle(2.0), 0.5);
    for u in [0.0, 0.7, 2.0, 4.5] {
        let d3 = curve.eval_d3(u).unwrap();
        EXPECT_NEAR(d3.point.distance(Point2::new(0.0, 0.0)), 2.5, 1e-14);
        EXPECT_NEAR(d3.d1.magnitude(), 2.5, 1e-14);
        EXPECT_NEAR(d3.d2.magnitude(), 2.5, 1e-14);
        EXPECT_NEAR(d3.d3.magnitude(), 2.5, 1e-14);
        let d2 = curve.eval_d2(u).unwrap();
        assert_eq!(d2.point, curve.eval_d0(u).unwrap());
        EXPECT_NEAR(curve.dn(u, 1).unwrap().magnitude(), 2.5, 1e-14);
        EXPECT_NEAR(curve.dn(u, 2).unwrap().magnitude(), 2.5, 1e-14);
        EXPECT_NEAR(curve.dn(u, 3).unwrap().magnitude(), 2.5, 1e-14);
        // Above order 3 the basis derivative is returned without the offset term.
        EXPECT_NEAR(curve.dn(u, 4).unwrap().magnitude(), 2.0, 1e-14);
    }
    let error = curve.eval_dn(0.0, 0).unwrap_err();
    assert_eq!(error.exception_type(), "Geom2d_UndefinedDerivative");
}

#[test]
fn offset_properties_follow_the_basis() {
    let curve = offset(&circle(2.0), 0.5);
    assert_eq!(
        (curve.first_parameter(), curve.last_parameter()),
        (0.0, 2.0 * PI)
    );
    assert!(curve.is_closed().unwrap() && curve.is_periodic());
    assert_eq!(curve.period().unwrap(), 2.0 * PI);
    assert_eq!(curve.continuity(), Shape::CN);
    assert_eq!(curve.basis_curve_continuity(), Shape::CN);
    assert!(curve.is_cn(3).unwrap());
    assert_eq!(
        curve.is_cn(-1).unwrap_err().exception_type(),
        "Standard_RangeError"
    );
    assert_eq!(curve.reversed_parameter(1.0).unwrap(), 2.0 * PI - 1.0);
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ))
    .cast();
    let shifted = offset(&line, 1.0);
    let scale = Transform2::scaling(Point2::new(0.0, 0.0), -3.0);
    assert_eq!(shifted.transformed_parameter(2.0, scale), 6.0);
    assert_eq!(shifted.parametric_transformation(scale), 3.0);
    assert!(!shifted.is_closed().unwrap());
    // The native Offset normal is (D1.Y, -D1.X): right of a line along +X.
    assert_eq!(shifted.value(2.0).unwrap(), Point2::new(2.0, -1.0));
}

#[test]
fn reverse_transform_and_copy() {
    let basis = circle(2.0);
    let curve = offset(&basis, 0.5);
    curve.reverse().unwrap();
    assert_eq!(curve.offset(), -0.5);
    // The basis is shared with the caller, as natively, so it is reversed too.
    EXPECT_NEAR(basis.value(PI / 2.0).unwrap().y(), -2.0, 1e-14);
    EXPECT_NEAR(curve.value(0.0).unwrap().x(), 2.5, 1e-14);
    curve
        .transform(Transform2::scaling(Point2::new(0.0, 0.0), -2.0))
        .unwrap();
    assert_eq!(curve.offset(), -1.0);
    EXPECT_NEAR(
        curve.value(1.0).unwrap().distance(Point2::new(0.0, 0.0)),
        5.0,
        1e-14,
    );
    let copy = Geom2dOffsetCurve::copy_of(&curve).unwrap();
    assert_eq!(copy.offset(), -1.0);
    assert!(copy.basis_curve() != curve.basis_curve());
    let generic: Handle<dyn Geom2dCurve> = curve.copy().unwrap().cast();
    assert_eq!(generic.value(0.3).unwrap(), curve.value(0.3).unwrap());
    // On the reversed basis the normal points inwards: a positive offset shrinks it.
    curve.set_offset_value(0.25);
    EXPECT_NEAR(
        curve.value(0.0).unwrap().distance(Point2::new(0.0, 0.0)),
        3.75,
        1e-14,
    );
}

#[test]
fn nested_trimmed_and_offset_bases_are_unwrapped() {
    let basis = circle(2.0);
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&basis, 0.5, 2.0, true, true).unwrap()).cast();
    let inner: Handle<dyn Geom2dCurve> = offset(&trimmed, 0.25).cast();
    let outer = offset(&inner, 0.5);
    assert_eq!(outer.offset(), 0.75);
    let unwrapped = outer.basis_curve();
    assert!(unwrapped.is_kind(<Geom2dTrimmedCurve as RuntimeTyped>::static_type()));
    assert_eq!(
        (unwrapped.first_parameter(), unwrapped.last_parameter()),
        (0.5, 2.0)
    );
    outer.set_basis_curve(&basis, false).unwrap();
    assert!(outer.basis_curve() == basis);
    let null: Handle<dyn Geom2dCurve> = Handle::default();
    let error = Geom2dOffsetCurve::new(&null, 1.0, false).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

fn c0_bspline(middle: (f64, f64)) -> Handle<dyn Geom2dCurve> {
    let mut knots = Array1::<f64>::with_bounds(1, 3).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 0.5, 1.0]);
    let mut mults = Array1::<i32>::with_bounds(1, 3).unwrap();
    mults.as_mut_slice().copy_from_slice(&[3, 2, 3]);
    let points = poles(&[(0.0, 0.0), (1.0, 0.0), middle, (3.0, 1.0), (4.0, 0.0)]);
    Handle::new(Geom2dBSplineCurve::new(&points, &knots, &mults, 2, false).unwrap()).cast()
}

#[test]
fn c0_bspline_bases_must_be_g1() {
    let kinked = c0_bspline((2.0, 0.0));
    let error = Geom2dOffsetCurve::new(&kinked, 0.1, false).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let unchecked = Geom2dOffsetCurve::new(&kinked, 0.1, true).unwrap();
    assert_eq!(unchecked.basis_curve_continuity(), Shape::C0);
    assert_eq!(unchecked.continuity(), Shape::C0);
    // Collinear poles around the double knot make the C0 curve tangent-continuous.
    let straight = c0_bspline((2.0, 0.5));
    let g1 = Geom2dOffsetCurve::new(&straight, 0.1, false).unwrap();
    assert_eq!(g1.basis_curve_continuity(), Shape::G1);
    assert_eq!(g1.continuity(), Shape::G1);
}

#[test]
fn cusp_uses_the_higher_derivative_direction() {
    let curve = offset(&cusp(), 0.25);
    assert_eq!(
        curve.eval_d0(0.0).unwrap_err().exception_type(),
        "Geom2d_UndefinedValue"
    );
    assert_eq!(
        curve.eval_d1(0.0).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
    // D2 at the cusp takes D1 := basis D2 = (6, 6); the offset normal is (1, -1)/sqrt(2).
    let d2 = curve.eval_d2(0.0).unwrap();
    let side: f64 = 0.25 / 2f64.sqrt();
    EXPECT_NEAR(d2.point.x(), side, 1e-15);
    EXPECT_NEAR(d2.point.y(), -side, 1e-15);
    assert!(curve.eval_d3(0.0).is_ok());
    let (mut d1, mut d2v, mut d3v, mut d4v) = (
        Vector2::default(),
        Vector2::default(),
        Vector2::default(),
        Vector2::default(),
    );
    let changed =
        adjust_derivative(&*cusp(), 4, 0.0, [&mut d1, &mut d2v, &mut d3v, &mut d4v]).unwrap();
    assert!(!changed);
    assert_eq!((d1.x(), d1.y()), (6.0, 6.0));
    // The next derivatives are the basis D3 = 6 (P3 - 3 P2 + 3 P1 - P0) and D4 = 0.
    assert_eq!((d2v.x(), d2v.y()), (-6.0, -18.0));
    assert_eq!((d3v.x(), d3v.y()), (0.0, 0.0));
    // Evaluated from the other end the chord runs backwards: the direction changes.
    let reversed = cusp().reversed().unwrap();
    let (mut a, mut b, mut c, mut d) = (
        Vector2::default(),
        Vector2::default(),
        Vector2::default(),
        Vector2::default(),
    );
    assert!(adjust_derivative(&*reversed, 3, 1.0, [&mut a, &mut b, &mut c, &mut d]).unwrap());
}

#[test]
fn evaluate_templates_match_the_curve() {
    let basis = circle(2.0);
    let curve = offset(&basis, 0.5);
    let u = 1.3;
    assert_eq!(
        evaluate_d0(u, &*basis, 0.5).unwrap(),
        Some(curve.eval_d0(u).unwrap())
    );
    assert_eq!(
        evaluate_d1(u, &*basis, 0.5).unwrap(),
        Some(curve.eval_d1(u).unwrap())
    );
    assert_eq!(
        evaluate_d2(u, &*basis, 0.5).unwrap(),
        Some(curve.eval_d2(u).unwrap())
    );
    assert_eq!(
        evaluate_d3(u, &*basis, 0.5).unwrap(),
        Some(curve.eval_d3(u).unwrap())
    );
    for n in [1, 2, 4] {
        assert_eq!(
            evaluate_dn(u, &*basis, 0.5, n).unwrap(),
            Some(curve.eval_dn(u, n).unwrap())
        );
    }
    // EvaluateDN(3) aliases D1 and D2 (native quirk), so it differs from EvalD3.
    let dn3 = evaluate_dn(u, &*basis, 0.5, 3).unwrap();
    assert_eq!(dn3, evaluate_d3_aliased(u, &*basis, 0.5).unwrap());
    assert_ne!(dn3, Some(curve.eval_dn(u, 3).unwrap()));
    assert_eq!(evaluate_d0(0.0, &*cusp(), 0.5).unwrap(), None);
    assert_eq!(evaluate_d1(0.0, &*cusp(), 0.5).unwrap(), None);
    assert!(evaluate_d2(0.0, &*cusp(), 0.5).unwrap().is_some());
    assert!(evaluate_d3(0.0, &*cusp(), 0.5).unwrap().is_some());
}

// One g2_offset_utils request: P, D1..D4, offset, direction change, and the native
// output bits of CalculateD0..D3 (flag, then the point and derivatives).
struct UtilsCase {
    input: [f64; 12],
    d0: (bool, [u64; 2]),
    d1: (bool, [u64; 4]),
    d2: (bool, [u64; 6]),
    d3: (bool, [u64; 8]),
}

fn v(x: f64, y: f64) -> Vector2 {
    Vector2::new(x, y)
}

fn bits(point: Point2, vectors: &[Vector2]) -> Vec<u64> {
    let mut all = vec![point.x().to_bits(), point.y().to_bits()];
    for vector in vectors {
        all.extend([vector.x().to_bits(), vector.y().to_bits()]);
    }
    all
}

fn check(case: &UtilsCase) {
    let i = case.input;
    let p = Point2::new(i[0], i[1]);
    let (d1, d2, d3, d4, offset, change) = (
        v(i[2], i[3]),
        v(i[4], i[5]),
        v(i[6], i[7]),
        v(i[8], i[9]),
        i[10],
        i[11] != 0.0,
    );
    let mut value = p;
    assert_eq!(calculate_d0(&mut value, d1, offset).unwrap(), case.d0.0);
    assert_eq!(bits(value, &[]), case.d0.1);
    let (mut value, mut a1) = (p, d1);
    assert_eq!(calculate_d1(&mut value, &mut a1, d2, offset), case.d1.0);
    assert_eq!(bits(value, &[a1]), case.d1.1);
    let (mut value, mut a1, mut a2) = (p, d1, d2);
    assert_eq!(
        calculate_d2(&mut value, &mut a1, &mut a2, d3, change, offset),
        case.d2.0
    );
    assert_eq!(bits(value, &[a1, a2]), case.d2.1);
    let (mut value, mut a1, mut a2, mut a3) = (p, d1, d2, d3);
    assert_eq!(
        calculate_d3(&mut value, [&mut a1, &mut a2, &mut a3], d4, change, offset),
        case.d3.0
    );
    assert_eq!(bits(value, &[a1, a2, a3]), case.d3.1);
}

const P: [u64; 2] = [4607182418800017408, 4611686018427387904];

#[test]
fn calculate_functions_take_the_native_branches() {
    // D1 through the low-magnitude branch; D2 and D3 refuse (R4, R6 below Resolution).
    check(&UtilsCase {
        input: [
            1.0, 2.0, 1e-110, -2e-110, 3.0, 1.0, -1.0, 2.0, 0.5, 0.25, 1.5, 0.0,
        ],
        d0: (true, [13823198110047783308, 4608664911954314958]),
        d1: (
            true,
            [
                13823198110047783304,
                4608664911954314959,
                6252120573803073757,
                15479996210285220061,
            ],
        ),
        d2: (
            false,
            [
                P[0],
                P[1],
                2961130568675457522,
                12189006205157603826,
                4613937818241073152,
                4607182418800017408,
            ],
        ),
        d3: (
            false,
            [
                P[0],
                P[1],
                2961130568675457522,
                12189006205157603826,
                4613937818241073152,
                4607182418800017408,
                13830554455654793216,
                4611686018427387904,
            ],
        ),
    });
    // D2 through its low-magnitude branch (R5 below, R4 above Resolution).
    check(&UtilsCase {
        input: [
            1.0, 2.0, 1e-65, 0.0, 3.0, 1.0, -1.0, 2.0, 0.5, 0.25, 1.5, 1.0,
        ],
        d0: (true, [4607182418800017408, 4602678819172646912]),
        d1: (
            true,
            [
                4607182418800017408,
                4602678819172646912,
                5581870973988895318,
                0,
            ],
        ),
        d2: (
            true,
            [
                4607182418800017408,
                4602678819172646912,
                5581870973988895318,
                0,
                15789684989240756649,
                6554324918606258752,
            ],
        ),
        d3: (
            false,
            [
                P[0],
                P[1],
                3634644143635780247,
                0,
                4613937818241073152,
                4607182418800017408,
                13830554455654793216,
                4611686018427387904,
            ],
        ),
    });
    // D3 through its low-magnitude branch (R7 below, R6 above Resolution).
    check(&UtilsCase {
        input: [
            1.0, 2.0, 0.0, 1e-47, 3.0, 1.0, -1.0, 2.0, 0.5, 0.25, 1.5, 1.0,
        ],
        d0: (true, [4612811918334230528, 4611686018427387904]),
        d1: (
            true,
            [
                4612811918334230528,
                4611686018427387904,
                0,
                14543166275391796328,
            ],
        ),
        d2: (
            true,
            [
                4612811918334230528,
                4611686018427387904,
                0,
                14543166275391796328,
                15253742567787975042,
                6027385007809653934,
            ],
        ),
        d3: (
            true,
            [
                4612811918334230528,
                4611686018427387904,
                0,
                14543166275391796327,
                6026372956807103804,
                6027385007809653934,
                6735025587896326749,
                6748744128797338421,
            ],
        ),
    });
    // A regular tangent with a direction change: every function uses its stable branch.
    check(&UtilsCase {
        input: [1.0, 2.0, 2.0, 1.0, 3.0, 1.0, -1.0, 2.0, 0.5, 0.25, 1.5, 1.0],
        d0: (true, [4610203525273090354, 4604105192162466618]),
        d1: (
            true,
            [
                4610203525273090354,
                4604105192162466618,
                4610477575838158726,
                4605973976210788230,
            ],
        ),
        d2: (
            true,
            [
                4610203525273090354,
                4604105192162466618,
                4610477575838158726,
                4605973976210788230,
                13829958838290145856,
                4591880660478671040,
            ],
        ),
        d3: (
            true,
            [
                4610203525273090354,
                4604105192162466618,
                4610477575838158726,
                4605973976210788230,
                13817563892440270400,
                4623232841392921180,
                13849206076759851714,
                4629252831438371336,
            ],
        ),
    });
}

#[test]
fn zero_tangents_are_refused() {
    let mut p = Point2::new(1.0, 2.0);
    assert!(!calculate_d0(&mut p, v(0.0, 0.0), 1.0).unwrap());
    let mut d1 = v(0.0, 0.0);
    assert!(!calculate_d1(&mut p, &mut d1, v(1.0, 1.0), 1.0));
    assert_eq!(p, Point2::new(1.0, 2.0));
}
