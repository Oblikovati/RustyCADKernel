//! Regression and behaviour tests for the BSplSLib port: preserved native quirks,
//! documented rejections of undefined native accesses, and algebraic round trips.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::bsplslib::{
    self, EvaluatorFunction, InterpolationKnots, MultiplySource, ResolutionDirection, SurfaceSpan,
};
use rustycad_tkmath::gp::Point3;

fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut array: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

fn ints(values: &[i32]) -> Array1<'static, i32> {
    let mut array: Array1<'static, i32> = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

// Poles(i, j) = pole(i, j) on a 1-based rows x cols grid.
fn grid(rows: i32, cols: i32, pole: impl Fn(i32, i32) -> Point3) -> Array2<'static, Point3> {
    let mut result: Array2<'static, Point3> = Array2::with_bounds(1, rows, 1, cols).unwrap();
    for i in 1..=rows {
        for j in 1..=cols {
            result[(i, j)] = pole(i, j);
        }
    }
    result
}

/// A biquadratic-by-cubic surface on knots {0, 1, 2} x {0, 1}, clamped.
struct Fixture {
    uk: Array1<'static, f64>,
    um: Array1<'static, i32>,
    vk: Array1<'static, f64>,
    vm: Array1<'static, i32>,
    poles: Array2<'static, Point3>,
}

fn fixture() -> Fixture {
    let pole = |i: i32, j: i32| {
        let (x, y) = (f64::from(i), f64::from(j));
        Point3::new(x, y, (x * 0.7).sin() + (y * 0.4).cos() * x)
    };
    Fixture {
        uk: reals(&[0.0, 1.0, 2.0]),
        um: ints(&[3, 1, 3]),
        vk: reals(&[0.0, 1.0]),
        vm: ints(&[4, 4]),
        poles: grid(4, 4, pole),
    }
}

impl Fixture {
    fn span(&self, u: f64, v: f64) -> SurfaceSpan<'_, 'static> {
        SurfaceSpan {
            u,
            v,
            u_index: 0,
            v_index: 0,
            u_knots: &self.uk,
            v_knots: &self.vk,
            u_mults: Some(&self.um),
            v_mults: Some(&self.vm),
            u_degree: 2,
            v_degree: 3,
            u_rat: false,
            v_rat: false,
            u_per: false,
            v_per: false,
        }
    }
    fn point(&self, u: f64, v: f64) -> Point3 {
        bsplslib::d0(&self.span(u, v), &self.poles, None).unwrap()
    }
}

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    a.distance(&b) <= tolerance
}

const SAMPLES: [(f64, f64); 4] = [(0.0, 0.0), (0.3, 0.9), (1.25, 0.5), (2.0, 1.0)];

#[test]
fn is_rational_reads_its_default_tolerance_from_weights_i1_i2() {
    let mut weights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 2).unwrap();
    weights.init(1.0);
    // Epsilon(Weights(I1, I2)) reads row 1, column 3 of a two-column grid.
    assert!(bsplslib::is_rational(&weights, 1, 3, 1, 2, 0.0).is_err());
    assert!(!bsplslib::is_rational(&weights, 1, 3, 1, 2, 1e-9).unwrap());
    weights[(2, 1)] = 2.0;
    assert!(bsplslib::is_rational(&weights, 1, 2, 1, 2, 0.0).unwrap());
}

#[test]
fn dn_above_the_degree_is_zero_for_polynomial_patches() {
    let f = fixture();
    let vn = bsplslib::dn(&f.span(0.4, 0.6), 3, 0, &f.poles, None).unwrap();
    assert_eq!([vn.x(), vn.y(), vn.z()], [0.0, 0.0, 0.0]);
    let d2 = bsplslib::d2(&f.span(0.4, 0.6), &f.poles, None).unwrap();
    let dn = bsplslib::dn(&f.span(0.4, 0.6), 1, 1, &f.poles, None).unwrap();
    assert_eq!(d2.vuv, dn);
}

#[test]
fn insert_then_remove_restores_the_surface() {
    let f = fixture();
    let (add, add_mults) = (reals(&[0.5]), ints(&[1]));
    let mut poles = grid(5, 4, |_, _| Point3::new(0.0, 0.0, 0.0));
    let (mut knots, mut mults) = (reals(&[0.0; 4]), ints(&[0; 4]));
    let inserted = (&mut poles, &mut knots, &mut mults);
    bsplslib::insert_knots(
        true,
        2,
        false,
        &f.poles,
        None,
        &f.uk,
        &f.um,
        &add,
        Some(&add_mults),
        inserted.0,
        None,
        inserted.1,
        inserted.2,
        0.0,
        true,
    )
    .unwrap();
    assert_eq!(knots.as_slice(), &[0.0, 0.5, 1.0, 2.0]);
    let mut restored = grid(4, 4, |_, _| Point3::new(0.0, 0.0, 0.0));
    let (mut rk, mut rm) = (reals(&[0.0; 3]), ints(&[0; 3]));
    let removed = bsplslib::remove_knot(
        true,
        2,
        0,
        2,
        false,
        &poles,
        None,
        &knots,
        &mults,
        &mut restored,
        None,
        &mut rk,
        &mut rm,
        1e-9,
    )
    .unwrap();
    assert!(removed);
    assert!(
        restored
            .as_slice()
            .iter()
            .zip(f.poles.as_slice())
            .all(|(a, b)| close(*a, *b, 1e-12))
    );
}

#[test]
fn increase_degree_preserves_the_surface() {
    let f = fixture();
    // V is a Bezier cubic: elevating to 5 adds two poles and keeps both knots.
    let mut poles = grid(4, 6, |_, _| Point3::new(0.0, 0.0, 0.0));
    let (mut knots, mut mults) = (reals(&[0.0; 2]), ints(&[0; 2]));
    bsplslib::increase_degree(
        false, 3, 5, false, &f.poles, None, &f.vk, &f.vm, &mut poles, None, &mut knots, &mut mults,
    )
    .unwrap();
    assert_eq!(mults.as_slice(), &[6, 6]);
    for (u, v) in SAMPLES {
        let span = SurfaceSpan {
            v_mults: Some(&mults),
            v_knots: &knots,
            v_degree: 5,
            ..f.span(u, v)
        };
        assert!(close(
            bsplslib::d0(&span, &poles, None).unwrap(),
            f.point(u, v),
            1e-12
        ));
    }
}

#[test]
fn poles_coefficients_need_the_poles_shape() {
    let poles = grid(2, 3, |i, j| {
        Point3::new(f64::from(i), f64::from(j), f64::from(i * j))
    });
    // Natively a [max][min] target with fewer U than V poles is filled through row-major
    // offsets without the bound check; the port raises the checked Standard_OutOfRange.
    let mut transposed = grid(3, 2, |_, _| Point3::new(0.0, 0.0, 0.0));
    assert!(bsplslib::poles_coefficients(&poles, None, &mut transposed, None).is_err());
    let mut coefficients = grid(2, 3, |_, _| Point3::new(0.0, 0.0, 0.0));
    bsplslib::poles_coefficients(&poles, None, &mut coefficients, None).unwrap();
    assert_eq!(coefficients[(1, 1)], poles[(1, 1)]);
}

struct Constant(f64);

impl EvaluatorFunction for Constant {
    fn evaluate(&self, _request: i32, _u: f64, _v: f64, result: &mut f64, error_code: &mut i32) {
        *result = self.0;
        *error_code = 0;
    }
}

#[test]
fn function_multiply_requires_weights_and_scales_numerator_and_denominator() {
    let f = fixture();
    let (uf, vf) = (
        reals(&[0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]),
        reals(&[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]),
    );
    let knots = InterpolationKnots {
        u_flat_knots: &uf,
        v_flat_knots: &vf,
        u_new_degree: 2,
        v_new_degree: 3,
    };
    let (mut numerator, mut denominator) = (
        grid(4, 4, |_, _| Point3::new(0.0, 0.0, 0.0)),
        Array2::with_bounds(1, 4, 1, 4).unwrap(),
    );
    let mut status: i32 = -7;
    let unweighted = MultiplySource {
        surface: f.span(0.0, 0.0),
        poles: &f.poles,
        weights: None,
    };
    // Natively the evaluation dereferences the absent weights.
    assert!(
        bsplslib::function_multiply(
            &Constant(2.0),
            &unweighted,
            &knots,
            &mut numerator,
            &mut denominator,
            &mut status
        )
        .is_err()
    );
    let weights = bsplslib::unit_weights(4, 4).unwrap();
    let source = MultiplySource {
        surface: f.span(0.0, 0.0),
        poles: &f.poles,
        weights: Some(&weights),
    };
    bsplslib::function_multiply(
        &Constant(2.0),
        &source,
        &knots,
        &mut numerator,
        &mut denominator,
        &mut status,
    )
    .unwrap();
    assert_eq!(status, 0);
    assert!(
        denominator
            .as_slice()
            .iter()
            .all(|w| (w - 2.0).abs() < 1e-12)
    );
    let pole = f.poles[(2, 3)];
    assert!(close(
        numerator[(2, 3)],
        Point3::new(2.0 * pole.x(), 2.0 * pole.y(), 2.0 * pole.z()),
        1e-10
    ));
}

#[test]
fn resolution_of_a_bilinear_patch_follows_the_pole_difference_bound() {
    let poles = grid(2, 2, |i, j| {
        Point3::new(2.0 * f64::from(i), 3.0 * f64::from(j), 0.0)
    });
    let (knots, mults) = (reals(&[0.0, 1.0]), ints(&[2, 2]));
    let direction = ResolutionDirection {
        knots: &knots,
        mults: &mults,
        degree: 1,
        rational: false,
        periodic: false,
    };
    let (ut, vt) = bsplslib::resolution(&poles, None, direction, direction, 1.0).unwrap();
    assert!((ut - 1.0 / (2.0 * std::f64::consts::SQRT_2)).abs() < 1e-15);
    assert!((vt - 1.0 / (3.0 * std::f64::consts::SQRT_2)).abs() < 1e-15);
}

#[test]
fn flat_pole_arrays_round_trip_in_both_directions() {
    let f = fixture();
    let weights = bsplslib::unit_weights(4, 4).unwrap();
    for u_direction in [true, false] {
        let mut flat: Array1<'static, f64> = Array1::with_bounds(1, 64).unwrap();
        bsplslib::set_rational_poles(&f.poles, &weights, &mut flat, u_direction).unwrap();
        let mut poles = grid(4, 4, |_, _| Point3::new(0.0, 0.0, 0.0));
        let mut back: Array2<'static, f64> = Array2::with_bounds(1, 4, 1, 4).unwrap();
        bsplslib::get_rational_poles(&flat, &mut poles, &mut back, u_direction).unwrap();
        assert_eq!(poles.as_slice(), f.poles.as_slice());
        assert!(back.as_slice().iter().all(|w| *w == 1.0));
    }
}

#[test]
fn iso_curves_evaluate_like_the_surface() {
    let f = fixture();
    let mut curve: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    bsplslib::iso(
        1.25,
        true,
        &f.poles,
        None,
        &f.uk,
        Some(&f.um),
        2,
        false,
        &mut curve,
        None,
    )
    .unwrap();
    // The U iso at 1.25 is the V Bezier cubic of the surface.
    let (bk, bm) = (reals(&[0.0, 1.0]), ints(&[4, 4]));
    let point = rustycad_tkmath::bsplclib::d0(
        &rustycad_tkmath::bsplclib::CurveSpan {
            u: 0.5,
            index: 0,
            degree: 3,
            periodic: false,
            knots: &bk,
            mults: Some(&bm),
        },
        &curve,
        None,
    )
    .unwrap();
    assert!(close(point, f.point(1.25, 0.5), 1e-12));
}

#[test]
fn undefined_native_accesses_are_rejected() {
    // Negative counts build a view of absurd size natively.
    assert!(bsplslib::unit_weights(-3, 4).is_err());
    // Degrees above MaxDegree raise Standard_OutOfRange before any evaluation.
    let f = fixture();
    let span = SurfaceSpan {
        u_degree: 26,
        ..f.span(0.5, 0.5)
    };
    assert!(bsplslib::d0(&span, &f.poles, None).is_err());
    // A rational evaluation without weights dereferences null natively.
    let rational = SurfaceSpan {
        u_rat: true,
        ..f.span(0.5, 0.5)
    };
    assert!(bsplslib::d1(&rational, &f.poles, None).is_err());
}
