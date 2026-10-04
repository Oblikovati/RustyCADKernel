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
fn raw_derivatives_reject_orders_above_native_cache_capacity() {
    let cache: Cache = cache_2d();
    let mut rows: [f64; 16] = [-77.0; 16];

    let error = cache
        .derivatives_local_into(0.25, 4, &mut rows)
        .unwrap_err();

    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert_eq!(rows, [-77.0; 16]);
}

fn assert_near(actual: &[f64], expected: &[f64]) {
    for (left, right) in actual.iter().zip(expected) {
        assert!((left - right).abs() <= 1.0e-14, "{left} != {right}");
    }
}
