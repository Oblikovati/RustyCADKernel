//! Supplemental MathSys::Utils members the upstream suites do not call directly: the
//! fixed-size eliminations, merit/Armijo checks, box helpers, linear-algebra helpers and
//! the shared Newton/LM drivers and finalizers.
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};
use rustycad_tkmath::math_poly::LongDouble;
use rustycad_tkmath::math_sys::utils::*;
use rustycad_tkmath::math_sys::{
    LMConfig, NewtonBoundsN, NewtonOptions, NewtonResultN, SystemResult,
};
use rustycad_tkmath::math_utils::Status;

/// Residual `x - target` with identity Jacobian over two variables.
struct Shift {
    target: [f64; 2],
}
impl FunctionSet for Shift {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        for i in 0..2 {
            *f.change_at(i).unwrap() = x.at(i).unwrap() - self.target[i];
        }
        true
    }
}
impl FunctionSetWithDerivatives for Shift {
    fn derivatives(&mut self, _x: &Vector<'_>, j: &mut Matrix<'_>) -> bool {
        for r in 0..2 {
            for c in 0..2 {
                *j.change_at(r, c).unwrap() = if r == c { 1.0 } else { 0.0 };
            }
        }
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, j: &mut Matrix<'_>) -> bool {
        self.value(x, f) && self.derivatives(x, j)
    }
}
fn vector(values: &[f64]) -> Vector<'static> {
    let mut result: Vector<'static> = Vector::with_size(values.len());
    for (i, value) in values.iter().enumerate() {
        *result.change_at(i).unwrap() = *value;
    }
    result
}
fn bounds<const N: usize>(min: [f64; N], max: [f64; N]) -> NewtonBoundsN<N> {
    NewtonBoundsN {
        min,
        max,
        has_bounds: true,
    }
}

#[test]
fn constants_match_the_native_namespace() {
    assert_eq!(LINE_SEARCH_MAX, 8);
    assert_eq!(LM_DAMPING_MAX, 24);
    assert_eq!(ARMIJO_C1, 1.0e-4);
    assert_eq!(NEWTON_PIVOT_TOL, 64.0 * f64::EPSILON);
    assert_eq!(RANK_TOLERANCE, 64.0 * f64::EPSILON);
    assert_eq!((CRITICAL_GRAD, CRITICAL_GRAD_SQ), (1.0e-30, 1.0e-60));
}

#[test]
fn solve_2x2_scales_rows_and_refuses_singular_systems() {
    let (mut du, mut dv) = (91.0, 91.0);
    assert!(solve_2x2(
        &[[2.0, 0.0], [0.0, 4.0]],
        &[2.0, 4.0],
        &mut du,
        &mut dv
    ));
    assert_eq!((du, dv), (-1.0, -1.0));
    let (mut du, mut dv) = (91.0, 91.0);
    assert!(!solve_2x2(
        &[[1.0, 2.0], [2.0, 4.0]],
        &[1.0, 1.0],
        &mut du,
        &mut dv
    ));
    assert!(!solve_2x2(
        &[[0.0, 0.0], [0.0, 1.0]],
        &[1.0, 1.0],
        &mut du,
        &mut dv
    ));
    // A NaN row scale is refused like the native `!(scale > 0)`.
    assert!(!solve_2x2(
        &[[f64::NAN, 1.0], [0.0, 1.0]],
        &[1.0, 1.0],
        &mut du,
        &mut dv
    ));
    assert_eq!((du, dv), (91.0, 91.0));
}

#[test]
fn solve_3x3_and_4x4_pivot_and_write_every_component() {
    let mut delta3 = [91.0; 3];
    assert!(solve_3x3(
        &[[0.0, 1.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 4.0]],
        &[1.0, 2.0, 4.0],
        &mut delta3
    ));
    assert_eq!(delta3, [-1.0, -1.0, -1.0]);
    assert!(!solve_3x3(
        &[[1.0; 3], [1.0; 3], [0.0, 0.0, 1.0]],
        &[1.0; 3],
        &mut delta3
    ));
    let mut delta4 = [91.0; 4];
    let j4 = [
        [0.0, 0.0, 0.0, 2.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 4.0, 0.0, 0.0],
        [0.0, 0.0, 8.0, 0.0],
    ];
    assert!(solve_4x4(&j4, &[2.0, 1.0, 4.0, 8.0], &mut delta4));
    assert_eq!(delta4, [-1.0, -1.0, -1.0, -1.0]);
    assert!(!solve_4x4(&[[0.0; 4]; 4], &[1.0; 4], &mut delta4));
}

#[test]
fn symmetric_svd_drops_negligible_eigenvalues() {
    let (mut du, mut dv) = (91.0, 91.0);
    assert!(solve_symmetric_2x2_svd(
        4.0, 0.0, 2.0, 4.0, 2.0, &mut du, &mut dv, 1.0e-15
    ));
    assert_eq!((du, dv), (-1.0, -1.0));
    // Native quirk: for a diagonal matrix with j11 < j22 the eigenvector of the larger
    // eigenvalue is built from the zero first column, so the solve reports failure.
    assert!(!solve_symmetric_2x2_svd(
        2.0, 0.0, 4.0, 2.0, 4.0, &mut du, &mut dv, 1.0e-15
    ));
    // Rank one: only the eigen direction (1, 1) / sqrt(2) with lambda 2 is kept.
    assert!(solve_symmetric_2x2_svd(
        1.0, 1.0, 1.0, 2.0, 2.0, &mut du, &mut dv, 1.0e-15
    ));
    assert!(
        (du + 1.0).abs() < 1e-15 && (dv + 1.0).abs() < 1e-15,
        "{du} {dv}"
    );
    assert!(!solve_symmetric_2x2_svd(
        0.0, 0.0, 0.0, 1.0, 1.0, &mut du, &mut dv, 1.0e-15
    ));
}

#[test]
fn merit_helpers_follow_the_native_long_double_checks() {
    assert!(is_finite_array(&[1.0, 2.0]) && !is_finite_array(&[1.0, f64::INFINITY]));
    assert_eq!(safe_norm_n(&[3.0, 4.0]), 5.0);
    assert_eq!(safe_norm_n(&[1e300, 1e300]), 1e300 * 2.0_f64.sqrt());
    assert_eq!(safe_distance_n(&[3.0, 4.0], &[0.0, 0.0]), 5.0);
    assert!(is_step_within_tolerance(&[1.0, 2.0], &[1.5, 2.5], 0.5));
    assert!(!is_step_within_tolerance(&[1.0], &[2.0], 0.5));
    assert!(is_step_within_tolerance(&[f64::NAN], &[0.0], 0.5));
    let derivative =
        merit_directional_derivative(&[1.0, 2.0], &[[1.0, 0.0], [0.0, 1.0]], &[-1.0, -2.0]);
    assert_eq!(derivative, LongDouble::new(-5.0));
    assert!(is_armijo_accepted(1.0, 0.5, LongDouble::new(-1.0)));
    assert!(!is_armijo_accepted(1.0, 1.0, LongDouble::new(-1.0)));
    assert!(!is_armijo_accepted(0.0, 0.0, LongDouble::new(-1.0)));
    assert!(!is_armijo_accepted(1.0, 0.0, LongDouble::new(0.0)));
    assert!(!is_armijo_accepted(1.0, 0.0, LongDouble::new(-1.0e5)));
    assert!(is_finite_system_n(&[1.0], &[[2.0]]) && !is_finite_system_n(&[1.0], &[[f64::NAN]]));
}

#[test]
fn box_helpers_validate_limit_and_clamp() {
    let options = NewtonOptions::default();
    assert!(
        is_options_valid(&options)
            && is_options_valid_3d(&options)
            && is_options_valid_4d(&options)
    );
    let broken = NewtonOptions {
        max_step_ratio: 0.0,
        ..options
    };
    assert!(
        !is_options_valid(&broken)
            && !is_options_valid_3d(&broken)
            && !is_options_valid_4d(&broken)
    );
    assert!(is_bounds_valid_2d(&bounds([0.0; 2], [1.0; 2])));
    assert!(!is_bounds_valid_3d(&bounds([0.0, 2.0, 0.0], [1.0; 3])));
    assert!(!is_bounds_valid_4d(&bounds([f64::NAN; 4], [1.0; 4])));
    assert!(is_bounds_valid_4d(&NewtonBoundsN {
        has_bounds: false,
        ..bounds([f64::NAN; 4], [0.0; 4])
    }));
    assert_eq!(max_step_2d(&bounds([0.0; 2], [4.0, 1.0]), 0.5), Some(2.0));
    assert_eq!(max_step_3d(&bounds([0.0; 3], [0.1; 3]), 0.5), Some(0.5));
    assert_eq!(
        max_step_4d(
            &NewtonBoundsN {
                has_bounds: false,
                ..bounds([0.0; 4], [1.0; 4])
            },
            0.5
        ),
        None
    );
    let mut x2 = [-1.0, 5.0];
    clamp_2d(&mut x2, &bounds([0.0; 2], [2.0; 2]), false, 0.0);
    assert_eq!(x2, [0.0, 2.0]);
    let mut x3 = [-1.0, 5.0, 1.0];
    clamp_3d(&mut x3, &bounds([0.0; 3], [2.0; 3]), true, 0.25);
    assert_eq!(x3, [-0.5, 2.5, 1.0]);
    // GCC 13 `std::clamp` is `min(max(v, lo), hi)`: an inverted box yields `hi`.
    let mut x4 = [-10.0, 10.0, f64::NAN, 1.0];
    clamp_4d(&mut x4, &bounds([3.0; 4], [1.0; 4]), false, 0.0);
    assert_eq!(&x4[..2], &[1.0, 1.0]);
    assert!(x4[2].is_nan());
}

#[test]
fn linear_helpers_rank_and_solve() {
    let b = vector(&[3.0, 4.0]);
    assert_eq!(safe_norm(&b), 5.0);
    assert!(is_finite_vector(&b) && !is_finite_vector(&vector(&[f64::NAN])));
    let mut a: Matrix<'static> = Matrix::filled(0, 2, 0, 1, 0.0).unwrap();
    for (r, row) in [[1.0, 0.0], [0.0, 2.0], [0.0, 0.0]].iter().enumerate() {
        for (c, value) in row.iter().enumerate() {
            *a.change_at(r, c).unwrap() = *value;
        }
    }
    assert!(is_finite_matrix(&a));
    assert_eq!(scaled_rank(&a, RANK_TOLERANCE), Some(2));
    let rhs = vector(&[1.0, 4.0, 0.0]);
    let mut x = vector(&[91.0, 91.0]);
    assert!(solve_qr(&a, &rhs, &mut x, RANK_TOLERANCE));
    assert!((x.at(0).unwrap() - 1.0).abs() < 1e-15 && (x.at(1).unwrap() - 2.0).abs() < 1e-15);
    let step: LinearStep = solve_linearized(&a, &rhs, RANK_TOLERANCE);
    assert_eq!((step.status, step.rank), (Status::Ok, 2));
    let wide: Matrix<'static> = Matrix::filled(0, 0, 0, 1, 1.0).unwrap();
    assert!(!solve_qr(&wide, &vector(&[1.0]), &mut x, RANK_TOLERANCE));
    assert_eq!(
        solve_linearized(&wide, &rhs, RANK_TOLERANCE).status,
        Status::NumericalError
    );
}

#[test]
fn finalizers_keep_the_solution_bounds_and_fill_diagnostics() {
    let mut f = Shift { target: [1.0, 2.0] };
    let mut point: Vector<'static> = Vector::with_bounds(3, 4).unwrap();
    (point[3], point[4]) = (1.0, 2.0);
    let mut result = SystemResult::default();
    assert!(finalize_system_result(
        &mut f,
        &point,
        2,
        1e-10,
        &mut result
    ));
    assert!(result.is_root && result.rank == 2 && result.value == Some(0.0));
    assert_eq!(result.solution.as_ref().unwrap().lower(), 3);
    let mut lm = SystemResult::default();
    assert!(finish_lm(
        &mut f,
        &point,
        2,
        &LMConfig::default(),
        1.0,
        None,
        None,
        &mut lm
    ));
    assert!(lm.is_root && lm.is_stationary && lm.gradient.is_some());
    assert_eq!(lm.solution.as_ref().unwrap().lower(), 3);
}

#[test]
fn drivers_ignore_a_single_bound_like_the_native_pointers() {
    let mut f = Shift { target: [1.0, 2.0] };
    let start = vector(&[0.0, 0.0]);
    let tolerance = vector(&[1e-12, 1e-12]);
    let lower = vector(&[-1.0, -1.0]);
    let upper = vector(&[0.5, 0.5]);
    let free = newton_impl(&mut f, &start, &tolerance, 1e-10, 20, Some(&lower), None);
    assert!(free.is_done() && bool::from(&free));
    assert_eq!(free.solution.as_ref().unwrap().at(1), Ok(&2.0));
    let boxed = newton_impl(
        &mut f,
        &start,
        &tolerance,
        1e-10,
        20,
        Some(&lower),
        Some(&upper),
    );
    assert!(!boxed.is_done());
    let lm_free = lm_impl(&mut f, &start, &LMConfig::default(), None, Some(&upper));
    assert!(lm_free.is_done());
    let lm_boxed = lm_impl(
        &mut f,
        &start,
        &LMConfig::default(),
        Some(&lower),
        Some(&upper),
    );
    assert!(lm_boxed.is_stationary && lm_boxed.is_done());
}

#[test]
fn result_types_and_defaults_match_native_initializers() {
    let result: NewtonResultN<3> = NewtonResultN::default();
    assert_eq!(result.status, Status::NotConverged);
    assert!(!result.is_done() && !bool::from(&result));
    let options = NewtonOptions::default();
    assert_eq!(
        (options.solver.x_tolerance, options.solver.f_tolerance),
        (1e-16, 1e-10)
    );
    assert!(options.enable_line_search && !options.allow_soft_bounds);
    let config = LMConfig::default();
    assert_eq!(
        (config.lambda_init, config.rank_tolerance),
        (1e-3, RANK_TOLERANCE)
    );
    assert!(NewtonBoundsN::<2>::default().has_bounds);
}
