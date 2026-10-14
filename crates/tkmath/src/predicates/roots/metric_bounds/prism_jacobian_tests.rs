// SPDX-License-Identifier: GPL-2.0-only
//! Independent dyadic determinants, including exact placement composition and singular maps.
use super::*;
use std::cmp::Ordering;

fn support() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
fn placement() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
fn exact_value(
    frame: MetricEmbedding<2>,
    map: MetricEmbedding<3>,
    vector: [f64; 3],
) -> DyadicBound {
    let result = affine_prism_jacobian(frame, map, vector, || false);
    result.expect("finite affine inputs have an exact determinant")
}
#[test]
fn composed_oblique_frame_uses_independent_exact_determinant() {
    let mut frame = support();
    frame.linear = [[2., 7.], [3., 11.], [5., 13.]];
    frame.scale = 2.;
    frame.origin = [1e200, -1e200, 7.];
    let mut map = placement();
    map.linear = [[1., 2., 0.], [0., -1., 1.], [2., 0., 1.]];
    map.scale = 0.5;
    map.origin = [-1e300, 1e300, -9.];
    // FLINT QQ determinant of columns (8,2,9),(29,2,27),(17,19,23) is 501.
    assert_eq!(
        exact_value(frame, map, [17., 19., 23.]).compare_f64(501.),
        Ok(Ordering::Equal)
    );
    assert_eq!(
        exact_value(frame, map, [-17., -19., -23.]).compare_f64(-501.),
        Ok(Ordering::Equal)
    );
}
#[test]
fn exact_composition_keeps_derivative_lost_by_rounded_matrix_vector_multiply() {
    let mut frame = support();
    let n = 4_503_599_627_370_496.;
    frame.linear = [[n, 0.], [n + 1., 0.], [0., 1.]];
    let mut map = placement();
    map.linear = [[n, -(n - 1.), 0.], [0., 0., 1.], [0., 1., 0.]];
    assert_eq!(n * n - (n - 1.) * (n + 1.), 0.);
    // Exact transformed U.x is one; V is (0,1,0), so this determinant is one.
    assert_eq!(
        exact_value(frame, map, [0., 0., 1.]).compare_f64(1.),
        Ok(Ordering::Equal)
    );
}
#[test]
fn every_finite_power_of_two_retains_exact_nonzero_jacobian() {
    let powers = (0..52)
        .map(|i| (f64::from_bits(1u64 << i), i - 1074))
        .chain((1..=2046).map(|i| (f64::from_bits((i as u64) << 52), i - 1023)));
    for (scale, exponent) in powers {
        let mut frame = support();
        frame.scale = scale;
        let mut map = placement();
        map.scale = scale;
        let actual = exact_value(frame, map, [0., 0., scale]);
        assert_eq!(actual, DyadicBound::new(BigInt::from(1), 5 * exponent));
    }
}
#[test]
fn tangent_sweep_and_degenerate_support_are_exactly_singular() {
    for vector in [[0.; 3], [1., 2., 0.], [-3., 4., -0.]] {
        assert_eq!(
            exact_value(support(), placement(), vector).compare_f64(0.),
            Ok(Ordering::Equal)
        );
    }
    let mut frame = support();
    frame.linear = [[1., 2.], [2., 4.], [3., 6.]];
    assert_eq!(
        exact_value(frame, placement(), [0., 0., 1.]).compare_f64(0.),
        Ok(Ordering::Equal)
    );
    let mut map = placement();
    map.scale = 0.;
    assert_eq!(
        exact_value(support(), map, [0., 0., 1.]).compare_f64(0.),
        Ok(Ordering::Equal)
    );
}
#[test]
fn invalid_origins_and_extrusion_remain_typed_even_for_singular_maps() {
    let mut map = placement();
    map.origin[2] = f64::INFINITY;
    let error = affine_prism_jacobian(support(), map, [0.; 3], || false).unwrap_err();
    assert!(
        matches!(error,ResidualError::Value {role:"origin",index:2,value} if value==f64::INFINITY)
    );
    let error =
        affine_prism_jacobian(support(), placement(), [0., 0., f64::NAN], || false).unwrap_err();
    assert!(
        matches!(error,ResidualError::Value {role:"prism extrusion",index:2,value} if value.is_nan())
    );
}
#[test]
fn cancellation_at_each_fixed_step_does_not_poison_the_next_request() {
    for stop_at in 1..=3 {
        let mut polls = 0;
        let failed = affine_prism_jacobian(support(), placement(), [0., 0., 2.], || {
            polls += 1;
            polls == stop_at
        });
        assert!(matches!(
            failed,
            Err(ResidualError::Roots(
                super::super::RootIsolationError::Cancelled
            ))
        ));
        assert_eq!(
            exact_value(support(), placement(), [0., 0., 2.]).compare_f64(2.),
            Ok(Ordering::Equal)
        );
    }
}
#[test]
fn independent_threads_match_serial_exact_results_without_shared_operation_state() {
    let expected = exact_value(support(), placement(), [3., 4., 2.]);
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let expected = &expected;
            scope.spawn(move || repeat_exact_result(expected));
        }
    });
}
fn repeat_exact_result(expected: &DyadicBound) {
    for _ in 0..32 {
        assert_eq!(&exact_value(support(), placement(), [3., 4., 2.]), expected);
    }
}
