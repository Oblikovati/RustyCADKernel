//! Standard_ShortReal, Standard_Time, Standard_Atomic and Standard_VersionInfo.
use rustycad_tkernel::standard::{atomic, short_real, time, version_info};
use std::sync::atomic::{AtomicI32, Ordering};

#[test]
fn short_real_limits_and_std_comparisons() {
    assert_eq!(short_real::short_real_small(), 1.1754944e-38);
    assert_eq!(short_real::short_real_digits(), 6);
    assert_eq!(short_real::short_real_epsilon(), 1.1920929e-7);
    assert_eq!(short_real::short_real_first(), -3.4028235e38);
    assert_eq!(short_real::short_real_first10_exp(), -37);
    assert_eq!(short_real::short_real_last(), 3.4028235e38);
    assert_eq!(short_real::short_real_last10_exp(), 38);
    assert_eq!(short_real::short_real_mantissa(), 24);
    assert_eq!(short_real::short_real_radix(), 2);
    assert_eq!(short_real::short_real_size(), 32);
    assert_eq!(short_real::abs(-0.0).to_bits(), 0);
    assert!(short_real::max(f32::NAN, 1.0).is_nan());
    assert_eq!(short_real::min(1.0, f32::NAN), 1.0);
}

#[test]
fn time_equality() {
    assert!(time::is_equal(i64::MIN, i64::MIN));
    assert!(!time::is_equal(0, -1));
}

#[test]
fn atomic_operations_return_new_values_and_wrap() {
    let value = AtomicI32::new(i32::MAX);
    assert_eq!(atomic::atomic_increment(&value), i32::MIN);
    assert_eq!(atomic::atomic_decrement(&value), i32::MAX);
    assert!(!atomic::atomic_compare_and_swap(&value, 0, 1));
    assert!(atomic::atomic_compare_and_swap(&value, i32::MAX, 7));
    assert_eq!(value.load(Ordering::SeqCst), 7);
}

#[test]
fn atomic_increment_is_atomic_across_threads() {
    let value = AtomicI32::new(0);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| (0..1000).for_each(|_| _ = atomic::atomic_increment(&value)));
        }
    });
    assert_eq!(value.load(Ordering::SeqCst), 4000);
}

#[test]
fn version_is_the_pinned_revision() {
    assert_eq!(version_info::occt_development_version(), "dev1-3d097a0328");
    assert_eq!(version_info::occt_version_double(), 8.1);
    assert_eq!(version_info::occt_version_string(), "8.1");
    assert_eq!(version_info::occt_version_string_complete(), "8.1.0");
    assert_eq!(
        version_info::occt_version_string_extended(),
        "8.1.0.dev1-3d097a0328"
    );
}
