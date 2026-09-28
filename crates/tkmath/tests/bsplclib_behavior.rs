//! Regression and behaviour tests for the BSplCLib port: preserved native quirks,
//! documented rejections of undefined native accesses, and algebraic round trips.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::bsplclib::{self, CurveSpan, EvaluatorFunction, FunctionTarget};
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

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

fn points(values: &[[f64; 3]]) -> Array1<'static, Point3> {
    let mut array: Array1<'static, Point3> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, [x, y, z]) in values.iter().enumerate() {
        array[i as i32 + 1] = Point3::new(*x, *y, *z);
    }
    array
}

// A clamped cubic with an interior double knot.
fn cubic() -> (
    Array1<'static, f64>,
    Array1<'static, i32>,
    Array1<'static, Point3>,
) {
    let knots = reals(&[0.0, 1.0, 2.5, 4.0]);
    let mults = ints(&[4, 1, 2, 4]);
    let poles = points(&[
        [0., 0., 0.],
        [1., 2., 0.],
        [2., 3., 1.],
        [4., 1., 2.],
        [5., -1., 1.],
        [6., 0., 0.],
        [7., 2., -1.],
    ]);
    (knots, mults, poles)
}

fn span<'r, 'a>(
    u: f64,
    knots: &'r Array1<'a, f64>,
    mults: &'r Array1<'a, i32>,
) -> CurveSpan<'r, 'a> {
    CurveSpan {
        u,
        index: 0,
        degree: 3,
        periodic: false,
        knots,
        mults: Some(mults),
    }
}

fn close(a: Point3, b: Point3, tolerance: f64) -> bool {
    a.distance(&b) <= tolerance
}

#[test]
fn bohm_keeps_the_dimension_one_division_and_the_reciprocal_elsewhere() {
    // Coincident knots: dimension 1 stores +0.0, other dimensions multiply by 0.0.
    let knots: [f64; 2] = [1.0, 1.0];
    let mut one: [f64; 2] = [3.0, 1.0];
    bsplclib::bohm(0.5, 1, 1, &knots, 1, &mut one).unwrap();
    assert!(one[1] == 0.0 && one[1].is_sign_positive());
    let mut two: [f64; 4] = [3.0, 3.0, 1.0, 1.0];
    bsplclib::bohm(0.5, 1, 1, &knots, 2, &mut two).unwrap();
    assert!(two[2] == 0.0 && two[2].is_sign_negative());
}

#[test]
fn derivative_poles_follow_the_degree_over_span_formula() {
    // Pole(i-1) = -Degree / (knot[i+Degree] - knot[i]) * (Pole(i-1) - Pole(i)), i from 1.
    let knots: [f64; 5] = [0.0, 0.0, 1.0, 3.0, 3.0];
    let mut poles: [f64; 3] = [1.0, -2.0, 4.0];
    bsplclib::derivative(2, &knots, 1, 3, 1, &mut poles).unwrap();
    assert_eq!(poles[..2], [3.0 * (-2.0 / 3.0), -6.0 * (-2.0 / 2.0)]);
}

#[test]
fn insert_then_remove_restores_the_curve() {
    let (knots, mults, poles) = cubic();
    let add = reals(&[1.7]);
    let (mut np, mut nk) = (0, 0);
    assert!(
        bsplclib::prepare_insert_knots(
            3, false, &knots, &mults, &add, None, &mut np, &mut nk, 0.0, true
        )
        .unwrap()
    );
    assert_eq!((np, nk), (8, 5));
    let mut new_poles: Array1<'static, Point3> = Array1::with_bounds(1, np).unwrap();
    let mut new_knots: Array1<'static, f64> = Array1::with_bounds(1, nk).unwrap();
    let mut new_mults: Array1<'static, i32> = Array1::with_bounds(1, nk).unwrap();
    bsplclib::insert_knots(
        3,
        false,
        &poles,
        None,
        &knots,
        &mults,
        &add,
        None,
        &mut new_poles,
        None,
        &mut new_knots,
        &mut new_mults,
        0.0,
        true,
    )
    .unwrap();
    for u in [0.3, 1.7, 3.2] {
        let before: Point3 = bsplclib::d0(&span(u, &knots, &mults), &poles, None).unwrap();
        let after: Point3 =
            bsplclib::d0(&span(u, &new_knots, &new_mults), &new_poles, None).unwrap();
        assert!(close(before, after, 1e-12));
    }
    let mut back_poles: Array1<'static, Point3> = Array1::with_bounds(1, 7).unwrap();
    let mut back_knots: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    let mut back_mults: Array1<'static, i32> = Array1::with_bounds(1, 4).unwrap();
    let removed: bool = bsplclib::remove_knot(
        3,
        0,
        3,
        false,
        &new_poles,
        None,
        &new_knots,
        &new_mults,
        &mut back_poles,
        None,
        &mut back_knots,
        &mut back_mults,
        1e-9,
    )
    .unwrap();
    assert!(removed);
    for i in 1..=7 {
        assert!(close(back_poles[i], poles[i], 1e-9));
    }
    assert_eq!(back_mults.as_slice(), mults.as_slice());
}

#[test]
fn insert_knots_addresses_new_poles_from_the_knot_lower_bound() {
    // Native quirk: the scheme output starts at NewKnots.Lower() + (index+1)*Dimension.
    let knots = reals(&[0.0, 1.0]);
    let mults = ints(&[2, 2]);
    let poles = reals(&[0.0, 1.0]);
    let add = reals(&[0.5]);
    let run = |knot_lower: i32| -> Vec<f64> {
        let mut new_poles: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
        new_poles.init(91.0);
        let mut new_knots: Array1<'static, f64> =
            Array1::with_bounds(knot_lower, knot_lower + 2).unwrap();
        let mut new_mults: Array1<'static, i32> =
            Array1::with_bounds(knot_lower, knot_lower + 2).unwrap();
        let _ = bsplclib::insert_knots_flat(
            1,
            false,
            1,
            &poles,
            &knots,
            &mults,
            &add,
            None,
            &mut new_poles,
            &mut new_knots,
            &mut new_mults,
            0.0,
            true,
        );
        new_poles.as_slice().to_vec()
    };
    assert_eq!(run(1), vec![0.0, 0.5, 1.0]);
    assert_ne!(run(2), run(1));
}

#[test]
fn increase_degree_preserves_the_curve() {
    let (knots, mults, poles) = cubic();
    let nk: i32 = bsplclib::increase_degree_count_knots(3, 5, false, &mults).unwrap();
    assert_eq!(nk, 4);
    let mut new_poles: Array1<'static, Point3> = Array1::with_bounds(1, 7 + 2 * 3).unwrap();
    let mut new_knots: Array1<'static, f64> = Array1::with_bounds(1, nk).unwrap();
    let mut new_mults: Array1<'static, i32> = Array1::with_bounds(1, nk).unwrap();
    bsplclib::increase_degree(
        3,
        5,
        false,
        &poles,
        None,
        &knots,
        &mults,
        &mut new_poles,
        None,
        &mut new_knots,
        &mut new_mults,
    )
    .unwrap();
    assert_eq!(new_mults.as_slice(), &[6, 3, 4, 6]);
    for u in [0.0, 0.9, 2.5, 3.99] {
        let before: Point3 = bsplclib::d0(&span(u, &knots, &mults), &poles, None).unwrap();
        let elevated = CurveSpan {
            degree: 5,
            ..span(u, &new_knots, &new_mults)
        };
        let after: Point3 = bsplclib::d0(&elevated, &new_poles, None).unwrap();
        assert!(close(before, after, 1e-12));
    }
}

#[test]
fn eval_rational_accumulates_weight_derivatives_into_the_caller_values() {
    // Native quirk: rows after the first clear WeightsResults[row*Dimension] and add
    // into WeightsResults[row].
    let flat = reals(&[0.0, 0.0, 1.0, 1.0]);
    let poles: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let weights: [f64; 2] = [1.0, 2.0];
    let mut poles_results: [f64; 4] = [0.0; 4];
    let mut weights_results: [f64; 3] = [91.0; 3];
    bsplclib::eval_extrapolated_rational(
        0.5,
        false,
        1,
        &[1, 1],
        1,
        &flat,
        2,
        &poles,
        &weights,
        &mut poles_results,
        &mut weights_results,
    )
    .unwrap();
    assert_eq!(weights_results[0], 1.5);
    assert_eq!(weights_results[1], 91.0 + 1.0);
    assert_eq!(weights_results[2], 0.0);
}

#[test]
fn extrapolation_reads_zero_for_the_terms_native_code_leaves_uninitialized() {
    // Degree-1 extrapolation of a cubic: value + (u - first) * slope, exactly.
    let flat = reals(&[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
    let poles: [f64; 4] = [0.0, 1.0, 3.0, 2.0];
    let mut value: [f64; 1] = [0.0];
    bsplclib::eval_extrapolated(-0.5, false, 0, &[1, 3], 3, &flat, 1, &poles, &mut value).unwrap();
    assert_eq!(value[0], 0.0 + -0.5 * 3.0);
}

#[test]
fn eval_bspline_basis_locates_periodically_but_uses_the_raw_parameter() {
    let flat = reals(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    let mut basis = rustycad_tkmath::math::Matrix::filled(1, 1, 1, 2, 0.0).unwrap();
    let mut first: i32 = 0;
    assert_eq!(
        bsplclib::eval_bspline_basis(0, 2, &flat, 7.5, &mut first, &mut basis, true).unwrap(),
        0
    );
    // The span is found for 7.5 wrapped into [1, 4], the values use 7.5 itself.
    assert_ne!(basis[(1, 1)] + basis[(1, 2)], 0.0);
    assert!(basis[(1, 2)] > 1.0);
}

#[test]
fn interpolation_reproduces_the_data() {
    let flat = reals(&[0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]);
    let mut parameters: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    bsplclib::build_schoenberg_points(2, &flat, &mut parameters).unwrap();
    assert_eq!(parameters.as_slice(), &[0.0, 0.25, 0.75, 1.0]);
    let values: [f64; 4] = [1.0, 3.0, -2.0, 5.0];
    let mut poles: Vec<f64> = values.to_vec();
    let mut problem: i32 = -1;
    bsplclib::interpolate_flat(
        2,
        &flat,
        &parameters,
        &ints(&[0, 0, 0, 0]),
        1,
        &mut poles,
        &mut problem,
    )
    .unwrap();
    assert_eq!(problem, 0);
    for (i, expected) in values.iter().enumerate() {
        let mut result: [f64; 1] = [0.0];
        let t: f64 = parameters[i as i32 + 1];
        bsplclib::eval_extrapolated(t, false, 0, &[2, 2], 2, &flat, 1, &poles, &mut result)
            .unwrap();
        assert!((result[0] - expected).abs() <= 1e-12);
    }
}

#[test]
fn cache_matches_direct_evaluation_on_a_periodic_curve() {
    let knots = reals(&[0.0, 1.0, 2.0, 3.0]);
    let mults = ints(&[1, 1, 1, 1]);
    let poles = points(&[[1., 0., 0.], [0., 1., 0.], [-1., 0., 1.]]);
    let mut flat: Array1<'static, f64> =
        Array1::with_bounds(1, bsplclib::knot_sequence_length(&mults, 2, true).unwrap()).unwrap();
    bsplclib::knot_sequence_with_degree(&knots, &mults, 2, true, &mut flat).unwrap();
    assert_eq!(flat.as_slice(), &[-2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    let mut cache = bsplclib::Cache::new(2, true, &flat, &poles, None).unwrap();
    for u in [0.25, 1.5, 2.9, 4.2, -0.7] {
        cache.build_cache(u, &flat, &poles, None).unwrap();
        let cached: Point3 = cache.d0(u).unwrap();
        let direct_span = CurveSpan {
            u,
            index: 0,
            degree: 2,
            periodic: true,
            knots: &knots,
            mults: Some(&mults),
        };
        let direct: Point3 = bsplclib::d0(&direct_span, &poles, None).unwrap();
        assert!(close(cached, direct, 1e-12));
    }
}

#[test]
fn degree_zero_derivatives_are_zero_where_native_reads_uninitialized_memory() {
    let poles = points(&[[1., 2., 3.]]);
    let (point, tangent): (Point3, Vector3) = bsplclib::bezier_d1(0.3, &poles, None).unwrap();
    assert_eq!(point, Point3::new(1., 2., 3.));
    assert_eq!(tangent, Vector3::new(0., 0., 0.));
}

#[test]
fn undefined_native_accesses_are_rejected() {
    // Weights without output weights (a null dereference natively).
    let (knots, mults, poles) = cubic();
    let weights = reals(&[1.0; 7]);
    let mut new_poles: Array1<'static, Point3> = Array1::with_bounds(1, 7).unwrap();
    let error = bsplclib::unperiodize(
        3,
        &mults,
        &knots,
        &poles,
        Some(&weights),
        &mut ints(&[0; 4]),
        &mut reals(&[0.0; 4]),
        &mut new_poles,
        None,
    );
    assert_eq!(error.unwrap_err().exception_type(), "Standard_OutOfRange");
    // A negative unit-weight count builds a view of absurd size natively.
    assert!(bsplclib::unit_weights(-1).is_err());
    // Degrees above MaxDegree raise as natively.
    let span = CurveSpan {
        degree: 26,
        ..span(0.5, &knots, &mults)
    };
    assert_eq!(
        bsplclib::d0(&span, &poles, None)
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    assert!(bsplclib::flat_bezier_knots(0).is_err());
}

#[test]
fn shared_unit_weights_detach_on_write() {
    let mut weights: Array1<'static, f64> = bsplclib::unit_weights(3).unwrap();
    weights[2] = 5.0;
    assert!(weights.is_deletable());
    assert_eq!(bsplclib::unit_weights_data()[1], 1.0);
}

struct Scale(f64);
impl EvaluatorFunction for Scale {
    fn evaluate(&self, _: i32, _: &[f64; 2], _: f64, result: &mut f64, error: &mut i32) {
        *result = self.0;
        *error = 0;
    }
}

#[test]
fn function_multiply_by_a_constant_scales_the_poles() {
    let flat = reals(&[0.0, 0.0, 1.0, 2.0, 2.0]);
    let poles: [f64; 3] = [1.0, -2.0, 4.0];
    let target = FunctionTarget {
        bspline_degree: 1,
        bspline_flat_knots: &flat,
        flat_knots: &flat,
        new_degree: 1,
    };
    let mut new_poles: [f64; 3] = [0.0; 3];
    let mut status: i32 = -1;
    bsplclib::function_multiply_flat(&Scale(2.0), &target, 1, &poles, &mut new_poles, &mut status)
        .unwrap();
    assert_eq!(status, 0);
    assert_eq!(new_poles, [2.0, -4.0, 8.0]);
}

#[test]
fn two_dimensional_trimming_keeps_the_segment() {
    let knots = reals(&[0.0, 1.0, 2.0]);
    let mults = ints(&[3, 1, 3]);
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    for (i, (x, y)) in [(0.0, 0.0), (1.0, 2.0), (2.0, -1.0), (3.0, 0.5)]
        .into_iter()
        .enumerate()
    {
        poles[i as i32 + 1] = Point2::new(x, y);
    }
    let (nk, np) = bsplclib::prepare_trimming(2, false, &knots, &mults, 0.5, 1.5).unwrap();
    let mut new_knots: Array1<'static, f64> = Array1::with_bounds(1, nk).unwrap();
    let mut new_mults: Array1<'static, i32> = Array1::with_bounds(1, nk).unwrap();
    let mut new_poles: Array1<'static, Point2> = Array1::with_bounds(1, np).unwrap();
    bsplclib::trimming(
        2,
        false,
        &knots,
        &mults,
        &poles,
        None,
        0.5,
        1.5,
        &mut new_knots,
        &mut new_mults,
        &mut new_poles,
        None,
    )
    .unwrap();
    assert_eq!(new_knots.as_slice(), &[0.5, 1.0, 1.5]);
    let start = CurveSpan {
        u: 0.5,
        index: 0,
        degree: 2,
        periodic: false,
        knots: &knots,
        mults: Some(&mults),
    };
    let expected: Point2 = bsplclib::d0(&start, &poles, None).unwrap();
    assert!(expected.distance(new_poles[1]) <= 1e-12);
}
