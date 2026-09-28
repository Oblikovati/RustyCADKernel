//! Null-interval success paths supplement the legacy multiple-root declarations.
use rustycad_tkmath::{math::Vector, math_root::*, math_utils::Status};
#[test]
fn entire_domain_null_interval_preserves_native_states() {
    let mut f = |_: f64, y: &mut f64| {
        *y = 0.0;
        true
    };
    let r: AllRootsResult = find_all_roots_with_intervals(&mut f, 1.0, 2.0, 4, [1e-10; 3]);
    assert!(bool::from(&r));
    assert_eq!(r.nb_roots(), 0);
    assert_eq!(r.nb_intervals(), 1);
    assert_eq!(
        (
            r.null_intervals[0].a,
            r.null_intervals[0].b,
            r.null_intervals[0].state
        ),
        (1.0, 2.0, 0)
    );
    assert!(!bool::from(&AllRootsResult::default()));
}
#[test]
fn isolated_root_and_arbitrary_sample_bounds() {
    let mut f = |x: f64, y: &mut f64| {
        *y = x - 0.37;
        true
    };
    let mut samples: Vector<'static> = Vector::with_bounds(-2, 2).unwrap();
    for i in 0..5 {
        *samples.change_at(i).unwrap() = i as f64 / 4.0;
    }
    let r: AllRootsResult = find_all_roots_at_samples(&mut f, &samples, [1e-10; 3]);
    assert!(r.is_done());
    assert_eq!(r.nb_roots(), 1);
    assert_eq!(r.nb_intervals(), 0);
    assert!((r.roots[0] - 0.37).abs() < 1e-10);
    assert_eq!(r.root_states, [0]);
}
#[test]
fn null_interval_boundaries_are_refined() {
    let mut f = |x: f64, y: &mut f64| {
        *y = (x.abs() - 0.25).max(0.0);
        true
    };
    let r: AllRootsResult =
        find_all_roots_with_intervals(&mut f, -1.0, 1.0, 21, [1e-10, 1e-10, 1e-8]);
    assert!(r.is_done());
    assert_eq!(r.nb_intervals(), 1);
    assert!((r.null_intervals[0].a + 0.25000001).abs() < 1e-8);
    assert!((r.null_intervals[0].b - 0.25000001).abs() < 1e-8);
}
#[test]
fn invalid_samples_are_rejected_before_callbacks() {
    let mut calls: usize = 0;
    let mut f = |_: f64, y: &mut f64| {
        calls += 1;
        *y = 1.0;
        true
    };
    let samples: Vector<'static> = Vector::filled(0, 2, 1.0).unwrap();
    assert_eq!(
        find_all_roots_at_samples(&mut f, &samples, [1e-10; 3]).status,
        Status::InvalidInput
    );
    assert_eq!(
        find_all_roots_with_intervals(&mut f, 1.0, 0.0, 20, [1e-10; 3]).status,
        Status::InvalidInput
    );
    assert_eq!(calls, 0);
}
#[test]
fn multiple_results_retain_original_values_with_offset() {
    let mut f = |x: f64, y: &mut f64| {
        *y = x * x;
        true
    };
    let cfg: MultipleConfig = MultipleConfig {
        offset: 1.0,
        ..Default::default()
    };
    let r: MultipleResult = find_all_roots(&mut f, -2.0, 2.0, &cfg);
    assert!(bool::from(&r));
    assert_eq!(r.nb_roots(), 2);
    assert_eq!((r[0], r[1]), (-1.0, 1.0));
    assert_eq!(r.values, [1.0, 1.0]);
    assert!(!bool::from(&MultipleResult::default()));
}
