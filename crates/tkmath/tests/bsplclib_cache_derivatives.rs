//! Native-backed caller-buffer coverage for the protected cache derivative helper.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::gp::{Point2, Point3};

fn cache_2d() -> Cache {
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, 2).unwrap();
    poles[1] = Point2::new(0.0, 0.0);
    poles[2] = Point2::new(4.0, 8.0);
    let mut cache: Cache = Cache::new(1, false, &knots, &poles, None).unwrap();
    cache.build_cache(0.5, &knots, &poles, None).unwrap();
    cache
}

fn rational_cache_3d() -> Cache {
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 4).unwrap();
    knots.as_mut_slice().copy_from_slice(&[2.0, 2.0, 6.0, 6.0]);
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    poles[1] = Point3::new(0.0, 0.0, 0.0);
    poles[2] = Point3::new(4.0, 8.0, 12.0);
    let mut weights: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    weights.as_mut_slice().copy_from_slice(&[1.0, 2.0]);
    let mut cache: Cache = Cache::new(1, false, &knots, &poles, Some(&weights)).unwrap();
    cache
        .build_cache(3.0, &knots, &poles, Some(&weights))
        .unwrap();
    cache
}

fn quartic_cache_3d() -> Cache {
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 10).unwrap();
    knots.as_mut_slice()[..5].fill(2.0);
    knots.as_mut_slice()[5..].fill(6.0);
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, 5).unwrap();
    for index in 1..=4 {
        poles[index] = Point3::new(0.0, 0.0, 0.0);
    }
    poles[5] = Point3::new(1.0, 0.0, 0.0);
    let mut cache: Cache = Cache::new(4, false, &knots, &poles, None).unwrap();
    cache.build_cache(4.0, &knots, &poles, None).unwrap();
    cache
}

fn rational_quartic_cache_2d() -> Cache {
    let mut knots = Array1::with_bounds(1, 10).unwrap();
    knots.as_mut_slice()[..5].fill(2.0);
    knots.as_mut_slice()[5..].fill(6.0);
    let mut poles = Array1::with_bounds(1, 5).unwrap();
    let mut weights = Array1::with_bounds(1, 5).unwrap();
    for index in 1..=5 {
        let coordinate = f64::from(index - 1);
        poles[index] = Point2::new(coordinate, coordinate * coordinate);
        weights[index] = f64::from(index);
    }
    let mut cache = Cache::new(4, false, &knots, &poles, Some(&weights)).unwrap();
    cache
        .build_cache(3.0, &knots, &poles, Some(&weights))
        .unwrap();
    cache
}

#[test]
fn rational_2d_fourth_derivatives_match_native_and_preserve_tail() {
    let cache = rational_quartic_cache_2d();
    let mut global = [-77.0; 12];
    let mut local = [-77.0; 12];
    cache.derivatives_into(3.0, 4, &mut global).unwrap();
    cache.derivatives_local_into(0.25, 4, &mut local).unwrap();
    let expected = [
        1.375, 2.6875, 1.0625, 3.21875, -0.3125, 0.90625, 0.46875, -0.234375, -0.9375, 0.46875,
        -77.0, -77.0,
    ];
    assert_eq!(global, expected);
    assert_eq!(local, expected);
    let fixed = cache.derivatives(3.0, 4).unwrap();
    assert_eq!(&fixed[..10], &expected[..10]);
}

#[test]
fn rational_2d_rejects_temporary_overflow_and_short_output_before_writing() {
    let cache = rational_quartic_cache_2d();
    let mut output = [-77.0; 12];
    let failure = cache.derivatives_into(3.0, 5, &mut output).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("0..=4"));
    assert_eq!(output, [-77.0; 12]);
    let failure = cache
        .derivatives_local_into(0.25, 4, &mut output[..9])
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_DimensionMismatch");
    assert_eq!(output, [-77.0; 12]);
}

#[test]
fn local_raw_derivatives_match_native_and_preserve_extra_capacity() {
    let cache: Cache = cache_2d();
    let mut rows: [f64; 6] = [-77.0; 6];

    cache.derivatives_local_into(0.25, 1, &mut rows).unwrap();

    assert_eq!(rows, [1.0, 2.0, 4.0, 8.0, -77.0, -77.0]);
}

#[test]
fn parameter_raw_derivatives_use_the_cached_span() {
    let cache: Cache = cache_2d();
    let mut rows: [f64; 4] = [0.0; 4];

    cache.derivatives_into(0.25, 1, &mut rows).unwrap();

    assert_eq!(rows, [1.0, 2.0, 4.0, 8.0]);
}

#[test]
fn raw_derivatives_clear_rows_above_curve_degree() {
    let cache: Cache = cache_2d();
    let mut rows: [f64; 10] = [-77.0; 10];

    cache.derivatives_local_into(0.25, 3, &mut rows).unwrap();

    assert_eq!(rows, [1.0, 2.0, 4.0, 8.0, 0.0, 0.0, 0.0, 0.0, -77.0, -77.0]);
}

#[test]
fn non_rational_cache_writes_defined_order_five_rows() {
    let cache: Cache = quartic_cache_3d();
    let mut global: [f64; 20] = [-77.0; 20];
    let mut local: [f64; 20] = [-77.0; 20];

    cache.derivatives_into(4.0, 5, &mut global).unwrap();
    cache.derivatives_local_into(0.5, 5, &mut local).unwrap();

    assert_eq!(
        global,
        [
            0.0625, 0.0, 0.0, 0.125, 0.0, 0.0, 0.1875, 0.0, 0.0, 0.1875, 0.0, 0.0, 0.09375, 0.0,
            0.0, 0.0, 0.0, 0.0, -77.0, -77.0,
        ]
    );
    assert_eq!(local, global);
}

#[test]
fn rational_3d_global_and_local_buffers_match_native() {
    let cache: Cache = rational_cache_3d();
    let mut global: [f64; 8] = [-77.0; 8];
    let mut local: [f64; 8] = [-77.0; 8];

    cache.derivatives_into(3.0, 1, &mut global).unwrap();
    cache.derivatives_local_into(0.25, 1, &mut local).unwrap();

    assert_near(&global, &[1.6, 3.2, 4.8, 1.28, 2.56, 3.84, -77.0, -77.0]);
    assert_near(&local, &[1.6, 3.2, 4.8, 1.28, 2.56, 3.84, -77.0, -77.0]);
}

#[test]
fn raw_derivatives_reject_short_buffers_without_writing() {
    let cache: Cache = cache_2d();
    let mut rows: [f64; 3] = [-77.0; 3];

    let error = cache
        .derivatives_local_into(0.25, 1, &mut rows)
        .unwrap_err();

    assert_eq!(error.exception_type(), "Standard_DimensionMismatch");
    assert_eq!(rows, [-77.0; 3]);
}

#[test]
fn local_rational_derivatives_reject_orders_above_fixed_native_capacity() {
    let cache: Cache = rational_cache_3d();
    let mut rows: [f64; 16] = [-77.0; 16];

    for derivative in [-1, 4] {
        let error = cache
            .derivatives_local_into(0.25, derivative, &mut rows)
            .unwrap_err();
        assert_eq!(error.exception_type(), "Standard_OutOfRange");
        assert_eq!(rows, [-77.0; 16]);
    }
}

#[test]
fn global_raw_derivatives_reject_negative_order_without_writing() {
    let cache: Cache = quartic_cache_3d();
    let mut rows: [f64; 20] = [-77.0; 20];

    let error = cache.derivatives_into(4.0, -1, &mut rows).unwrap_err();

    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(rows, [-77.0; 20]);
}

#[test]
fn rational_cache_keeps_four_row_temporary_limit() {
    let cache: Cache = rational_cache_3d();
    let mut rows: [f64; 20] = [-77.0; 20];

    let error = cache
        .derivatives_local_into(0.25, 4, &mut rows)
        .unwrap_err();

    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(rows, [-77.0; 20]);
}

#[test]
fn high_order_global_output_requires_all_rows_before_writing() {
    let cache: Cache = quartic_cache_3d();
    let mut global: [f64; 17] = [-77.0; 17];
    let mut local: [f64; 17] = [-77.0; 17];

    let error = cache.derivatives_into(4.0, 5, &mut global).unwrap_err();
    let local_error = cache
        .derivatives_local_into(0.5, 5, &mut local)
        .unwrap_err();

    assert_eq!(error.exception_type(), "Standard_DimensionMismatch");
    assert_eq!(local_error.exception_type(), "Standard_DimensionMismatch");
    assert_eq!(global, [-77.0; 17]);
    assert_eq!(local, [-77.0; 17]);
}

#[test]
fn fixed_array_derivatives_reject_high_order_without_panicking() {
    let cache: Cache = quartic_cache_3d();

    let error = cache.derivatives(4.0, 4).unwrap_err();

    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

fn assert_near(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (left, right) in actual.iter().zip(expected) {
        assert!((left - right).abs() <= 1.0e-14, "{left} != {right}");
    }
}
