//! GoogleTest assertion helpers shared by the translated TKHelix suites.
#![allow(dead_code)]
use rustycad_tkernel::standard::Failure;

/// GoogleTest's biased representation of a double (sign and magnitude mapped onto an
/// ordered unsigned range), as `FloatingPoint::SignAndMagnitudeToBiased` does.
fn biased(value: f64) -> u64 {
    let bits: u64 = value.to_bits();
    let sign_mask: u64 = 1 << 63;
    if bits & sign_mask != 0 {
        (!bits).wrapping_add(1)
    } else {
        sign_mask | bits
    }
}

/// `EXPECT_DOUBLE_EQ(a, b)`: equal within 4 units in the last place, NaN never equal.
pub fn expect_double_eq(a: f64, b: f64) {
    let (x, y) = (biased(a), biased(b));
    let distance: u64 = x.max(y) - x.min(y);
    assert!(!a.is_nan() && !b.is_nan() && distance <= 4, "{a} != {b}");
}

/// `EXPECT_NEAR(a, b, t)`.
pub fn expect_near(a: f64, b: f64, t: f64) {
    assert!((a - b).abs() <= t, "|{a} - {b}| > {t}");
}

/// `EXPECT_THROW(statement, Type)`: the statement raised the native exception `kind`.
pub fn expect_throw<T>(result: Result<T, Failure>, kind: &str) {
    match result {
        Ok(_) => panic!("expected {kind}, got no exception"),
        Err(failure) => assert_eq!(failure.exception_type(), kind),
    }
}
