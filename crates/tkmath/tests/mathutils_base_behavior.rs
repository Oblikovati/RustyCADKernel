//! Native modern utility defaults, guards, extended arithmetic and domain policies.
use rustycad_tkmath::{math::Vector, math_utils::*};
fn vector(lower: i32, values: &[f64]) -> Vector<'static> {
    let mut v: Vector<'static> =
        Vector::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    for (i, &x) in values.iter().enumerate() {
        v[lower + i as i32] = x;
    }
    v
}
#[test]
fn extended_dot_preserves_cancellation_and_large_finite_products() {
    assert_eq!(
        dot_product(&vector(1, &[1e16, 1.0, -1e16]), &vector(-2, &[1.0; 3])),
        1.0
    );
    assert_eq!(
        dot_product(
            &vector(1, &[1e308, -1e308, 1.0]),
            &vector(3, &[1e308, 1e308, 1.0])
        ),
        1.0
    );
    assert!(dot_product(&vector(0, &[1.0]), &vector(0, &[])).is_nan());
    assert!(dot_product(&vector(0, &[f64::INFINITY]), &vector(0, &[1.0])).is_nan());
    assert_eq!(dot_product(&vector(0, &[]), &vector(0, &[])), 0.0);
}
#[test]
fn stable_norms_preserve_extremes_and_native_nonfinite_order() {
    let large: Vector<'static> = vector(2, &[3e300, 4e300]);
    assert_eq!(norm(&large), 5e300);
    assert_eq!(norm2(&large), f64::INFINITY);
    let small: Vector<'static> = vector(2, &[3e-300, 4e-300]);
    assert!((norm(&small) / 5e-300 - 1.0).abs() < 1e-15);
    assert_eq!(vector_inf_norm(&vector(0, &[f64::NAN, -3.0])), 3.0);
    assert!(vector_norm(&vector(0, &[f64::NAN, f64::INFINITY])).is_nan());
    assert_eq!(
        vector_norm(&vector(0, &[f64::INFINITY, f64::NAN])),
        f64::INFINITY
    );
    assert_eq!(vector_norm(&vector(0, &[0.0, -0.0])), 0.0);
    assert_eq!(infinity_norm(&large), 4e300);
    assert_eq!(euclidean_norm(&large), norm(&large));
}
#[test]
fn scalar_guards_and_signed_arithmetic_match_native_conditions() {
    assert_eq!(clamp(2.0, -1.0, 1.0), 1.0);
    assert_eq!(clamp(-2.0, -1.0, 1.0), -1.0);
    assert!(clamp(f64::NAN, -1.0, 1.0).is_nan());
    assert!(is_zero(ZERO_TOL, ZERO_TOL));
    assert!(!is_zero(0.0, -1.0));
    assert!(!is_zero(f64::INFINITY, 1.0));
    assert!(is_equal(1e12, 1e12 + 1.0, 1e-10));
    assert!(!is_equal(f64::INFINITY, f64::INFINITY, 1.0));
    assert!(!is_within_tolerance(-f64::MAX, f64::MAX, 0.0, 2.0));
    assert!(is_within_tolerance(0.0, 0.0, 0.0, 0.0));
    assert_eq!(safe_div(1.0, 0.0, 7.0), 7.0);
    assert_eq!(safe_div(4.0, 2.0, 7.0), 2.0);
    assert_eq!(
        (sign(-1.0), sign(0.0), sign(1.0), sign(f64::NAN)),
        (-1, 0, 1, 0)
    );
    assert_eq!(sign_transfer(-2.0, -0.0), 2.0);
    assert_eq!(sign_transfer(2.0, f64::NAN), -2.0);
    assert_eq!(square(-3.0), 9.0);
    assert_eq!(cube(-3.0), -27.0);
    assert_eq!(cube_root(-8.0), -2.0);
    assert_eq!(cube_root(8.0), 2.0);
    assert!(is_finite(1.0));
    assert!(!is_finite(f64::INFINITY));
}
#[test]
fn coefficient_scaling_retains_extreme_power_of_two_results() {
    assert!(compute_scale_factor(&[]).is_nan());
    assert!(compute_scale_factor(&[f64::NAN]).is_nan());
    assert_eq!(compute_scale_factor(&[0.0]), 1.0);
    assert_eq!(compute_scale_factor(&[2.0]), 1.0);
    assert_eq!(
        compute_scale_factor(&[2.0_f64.powi(100)]),
        2.0_f64.powi(-100)
    );
    assert_eq!(
        compute_scale_factor(&[2.0_f64.powi(-100)]),
        2.0_f64.powi(100)
    );
    assert_eq!(compute_scale_factor(&[f64::from_bits(1)]), f64::INFINITY);
    assert_eq!(compute_scale_factor(&[f64::MAX]), 2.0_f64.powi(-1023));
}
#[test]
fn convergence_uses_finite_absolute_and_relative_tests() {
    let mut config: Config = Config {
        x_tolerance: 1e-12,
        relative_tolerance: 1e-8,
        ..Default::default()
    };
    assert!(is_converged(1e12, 1e12 + 1.0, 1.0, &config));
    assert!(!is_converged(0.0, 1e-6, 1.0, &config));
    assert!(is_x_converged(1.0, 1.0 + 1e-9, 1e-8));
    assert!(is_f_converged(1e-9, 1e-8));
    assert!(is_min_converged(0.0, 1.0, 2.0, 2.0, &config));
    assert!(!is_min_converged(0.0, 1.0, f64::NAN, 2.0, &config));
    assert!(is_vector_converged(
        &vector(2, &[1.0]),
        &vector(-3, &[1.0]),
        1e-8
    ));
    assert!(!is_vector_converged(
        &vector(2, &[1.0]),
        &vector(-3, &[]),
        1e-8
    ));
    assert!(is_gradient_converged(&vector(0, &[0.0]), 0.0));
    assert!(!is_gradient_converged(&vector(0, &[f64::INFINITY]), 1.0));
    config.relative_tolerance = f64::NAN;
    assert!(!config.is_valid());
    assert!(!is_converged(1.0, 1.0, 0.0, &config));
}
#[test]
fn configuration_defaults_and_validation() {
    let c: Config = Config::new(1e-8, 7);
    assert!(c.is_valid());
    assert_eq!(c.step_min, f64::EPSILON);
    assert!(!Config::new(-1.0, 7).is_valid());
    assert!(!Config::new(1e-8, 0).is_valid());
    assert!(BoundedConfig::default().is_valid());
    assert!(!BoundedConfig::new(2.0, 1.0, 1e-8, 7).is_valid());
    assert!(NDimConfig::default().is_valid());
    assert!(NDimConfig::new(1e-8, 7, true).use_bounds);
    assert!(IntegConfig::default().is_valid());
    assert!(!IntegConfig::new(f64::INFINITY, 7).is_valid());
    assert!(LinConfig::default().is_valid());
    assert!(
        !LinConfig {
            singularity_tolerance: -1.0,
            use_pivoting: true
        }
        .is_valid()
    );
}
#[test]
fn domains_preserve_unchecked_arithmetic_and_checked_containment() {
    let a: Domain1D = Domain1D::new(-1.0, 3.0);
    assert!(a.is_valid());
    assert_eq!(a.length(), 4.0);
    assert_eq!(a.mid(), 1.0);
    assert!(a.contains(3.1, 0.2));
    assert!(!a.contains(f64::INFINITY, 0.0));
    assert_eq!(a.clamp(4.0), 3.0);
    assert!(a.is_large(3.0));
    assert!(a.is_full_period(4.1, 0.2));
    assert_eq!(a.lerp(0.5), 1.0);
    assert_eq!(a.normalize(1.0), 0.5);
    assert!(a.is_finite(4.0));
    assert!(!a.is_finite(3.0));
    assert!(a.is_equal(&a, 0.0));
    let b: Domain1D = Domain1D::new(2.0, -2.0);
    assert!(!b.is_valid());
    assert!(!b.contains(0.0, 0.0));
    assert_eq!(b.normalize(0.0), 0.5);
    let d: Domain2D = Domain2D::from_domains(a, Domain1D::new(2.0, 6.0));
    assert!(d.is_valid());
    assert_eq!(d.u_length(), 4.0);
    assert_eq!(d.v_length(), 4.0);
    assert_eq!(d.u_mid(), 1.0);
    assert_eq!(d.v_mid(), 4.0);
    assert!(d.contains(1.0, 4.0, 0.0));
    let (mut u, mut v): (f64, f64) = (-9.0, 9.0);
    d.clamp(&mut u, &mut v);
    assert_eq!((u, v), (-1.0, 6.0));
    assert!(d.is_large(3.0));
    assert!(d.is_u_full_period(4.0, 0.0));
    assert!(d.is_v_full_period(4.0, 0.0));
    assert!(d.is_finite(7.0));
    assert_eq!(d.u().min, -1.0);
    assert_eq!(d.v().max, 6.0);
    assert!(Domain1D::default().is_valid());
    assert!(Domain2D::default().is_valid());
}
#[test]
fn result_scalarresult_defaults_to_not_converged() {
    let mut result: ScalarResult = ScalarResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_polyresult_defaults_to_not_converged() {
    let mut result: PolyResult = PolyResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_vectorresult_defaults_to_not_converged() {
    let mut result: VectorResult = VectorResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_linearresult_defaults_to_not_converged() {
    let mut result: LinearResult = LinearResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_linearmultipleresult_defaults_to_not_converged() {
    let mut result: LinearMultipleResult = LinearMultipleResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_eigenresult_defaults_to_not_converged() {
    let mut result: EigenResult = EigenResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_decompresult_defaults_to_not_converged() {
    let mut result: DecompResult = DecompResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_integresult_defaults_to_not_converged() {
    let mut result: IntegResult = IntegResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn result_inverseresult_defaults_to_not_converged() {
    let mut result: InverseResult = InverseResult::default();
    assert!(!result.is_done());
    assert!(!bool::from(&result));
    result.status = Status::Ok;
    assert!(result.is_done());
    assert!(bool::from(&result));
}
#[test]
fn polynomial_result_indexes_storage_independently_of_root_count() {
    let result: PolyResult = PolyResult {
        roots: [1.0, 2.0, 3.0, 4.0],
        ..PolyResult::default()
    };
    assert_eq!(result[2], 3.0);
}
