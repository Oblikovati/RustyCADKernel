//! Regression tests for the parity harness, not translated OCCT tests.

use rustycad_parity::Tolerance;

#[test]
fn absolute_and_relative_bounds() {
    let tolerance = Tolerance::new(1e-9, 1e-6).unwrap();
    assert!(tolerance.matches(0.0, 5e-10));
    assert!(tolerance.matches(1e6, 1e6 + 0.5));
    assert!(!tolerance.matches(1e6, 1e6 + 2.0));
    assert!(!tolerance.matches(0.0, 1e-8));
}

#[test]
fn rejects_invalid_tolerances() {
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Tolerance::new(invalid, 0.0).is_none());
        assert!(Tolerance::new(0.0, invalid).is_none());
    }
}

#[test]
fn special_values_have_explicit_semantics() {
    let exact = Tolerance::new(0.0, 0.0).unwrap();
    assert!(exact.matches(f64::INFINITY, f64::INFINITY));
    assert!(exact.matches(0.0, -0.0));
    assert!(!exact.matches(f64::INFINITY, f64::NEG_INFINITY));
    assert!(!exact.matches(f64::NAN, f64::NAN));
    assert!(!exact.matches(1.0, f64::INFINITY));
    assert!(!exact.matches(1.0, 1.0 + f64::EPSILON));
}

#[test]
fn finite_extremes_do_not_pass_due_to_overflow() {
    assert!(
        !Tolerance::new(0.0, 1.5)
            .unwrap()
            .matches(f64::MAX, -f64::MAX)
    );
    assert!(
        Tolerance::new(0.0, 2.0)
            .unwrap()
            .matches(f64::MAX, -f64::MAX)
    );
    assert!(
        !Tolerance::new(0.0, 0.0)
            .unwrap()
            .matches(0.0, f64::from_bits(1))
    );
}
