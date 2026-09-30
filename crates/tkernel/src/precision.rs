//! OCCT `Precision`: the modelling tolerances shared by every toolkit.
//!
//! Each function keeps the native constexpr expression (for example
//! `PConfusion() = Confusion() * 0.01`, not a literal `1e-9`), so every value has
//! the same bits as OCCT. C++ overloads take distinct Rust names: the one-argument
//! parametric forms divide by a curve speed `t` and carry a `_with_speed` suffix.

/// Precision::Angular(): tolerance for comparing angles, in radians.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::angular(), 1.0e-12);
/// ```
pub const fn angular() -> f64 {
    1.0e-12
}

/// Precision::Confusion(): distance below which two points are coincident.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::confusion(), 1.0e-7);
/// ```
pub const fn confusion() -> f64 {
    1.0e-7
}

/// Precision::SquareConfusion(): `confusion()` squared, for squared-distance tests.
///
/// ```
/// use rustycad_tkernel::precision::{confusion, square_confusion};
/// assert_eq!(square_confusion(), confusion() * confusion());
/// ```
pub const fn square_confusion() -> f64 {
    confusion() * confusion()
}

/// Precision::Computational(): the machine epsilon (`RealEpsilon()`).
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::computational(), f64::EPSILON);
/// ```
pub const fn computational() -> f64 {
    f64::EPSILON
}

/// Precision::SquareComputational(): `computational()` squared.
///
/// ```
/// assert!(rustycad_tkernel::precision::square_computational() > 0.0);
/// ```
pub const fn square_computational() -> f64 {
    computational() * computational()
}

/// Precision::Intersection(): tolerance used inside intersection algorithms.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::intersection(), 1.0e-7 * 0.01);
/// ```
pub const fn intersection() -> f64 {
    confusion() * 0.01
}

/// Precision::Approximation(): tolerance used by approximation algorithms.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::approximation(), 1.0e-7 * 10.0);
/// ```
pub const fn approximation() -> f64 {
    confusion() * 10.0
}

/// Precision::Parametric(P, T): converts a 3D tolerance `p` to parameter space
/// for a curve of speed `t`.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::parametric_with_speed(1.0, 4.0), 0.25);
/// ```
pub const fn parametric_with_speed(p: f64, t: f64) -> f64 {
    p / t
}

/// Precision::PConfusion(T): `confusion()` in parameter space for speed `t`.
///
/// ```
/// use rustycad_tkernel::precision::{confusion, p_confusion_with_speed};
/// assert_eq!(p_confusion_with_speed(2.0), confusion() / 2.0);
/// ```
pub const fn p_confusion_with_speed(t: f64) -> f64 {
    parametric_with_speed(confusion(), t)
}

/// Precision::SquarePConfusion(): `p_confusion()` squared.
///
/// ```
/// use rustycad_tkernel::precision::{p_confusion, square_p_confusion};
/// assert_eq!(square_p_confusion(), p_confusion() * p_confusion());
/// ```
pub const fn square_p_confusion() -> f64 {
    p_confusion() * p_confusion()
}

/// Precision::PIntersection(T): `intersection()` in parameter space for speed `t`.
///
/// ```
/// use rustycad_tkernel::precision::{intersection, p_intersection_with_speed};
/// assert_eq!(p_intersection_with_speed(2.0), intersection() / 2.0);
/// ```
pub const fn p_intersection_with_speed(t: f64) -> f64 {
    parametric_with_speed(intersection(), t)
}

/// Precision::PApproximation(T): `approximation()` in parameter space for speed `t`.
///
/// ```
/// use rustycad_tkernel::precision::{approximation, p_approximation_with_speed};
/// assert_eq!(p_approximation_with_speed(2.0), approximation() / 2.0);
/// ```
pub const fn p_approximation_with_speed(t: f64) -> f64 {
    parametric_with_speed(approximation(), t)
}

/// Precision::Parametric(P): converts `p` to parameter space with OCCT's
/// default speed factor of 100.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::parametric(1.0), 0.01);
/// ```
pub const fn parametric(p: f64) -> f64 {
    p * 0.01
}

/// Precision::PConfusion(): the default parametric confusion.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::p_confusion(), 1.0e-7 * 0.01);
/// ```
pub const fn p_confusion() -> f64 {
    confusion() * 0.01
}

/// Precision::PIntersection(): the default parametric intersection tolerance.
///
/// ```
/// use rustycad_tkernel::precision::{intersection, p_intersection};
/// assert_eq!(p_intersection(), intersection() * 0.01);
/// ```
pub const fn p_intersection() -> f64 {
    intersection() * 0.01
}

/// Precision::PApproximation(): the default parametric approximation tolerance.
///
/// ```
/// use rustycad_tkernel::precision::{approximation, p_approximation};
/// assert_eq!(p_approximation(), approximation() * 0.01);
/// ```
pub const fn p_approximation() -> f64 {
    approximation() * 0.01
}

/// Precision::IsInfinite(R): true when `|r|` reaches half of `infinite()`.
/// NaN is never infinite, as in the native comparison.
///
/// ```
/// use rustycad_tkernel::precision::is_infinite;
/// assert!(is_infinite(-1.0e100) && !is_infinite(f64::NAN));
/// ```
pub const fn is_infinite(r: f64) -> bool {
    r.abs() >= 0.5 * infinite()
}

/// Precision::IsPositiveInfinite(R).
///
/// ```
/// assert!(rustycad_tkernel::precision::is_positive_infinite(1.0e100));
/// ```
pub const fn is_positive_infinite(r: f64) -> bool {
    r >= 0.5 * infinite()
}

/// Precision::IsNegativeInfinite(R).
///
/// ```
/// assert!(rustycad_tkernel::precision::is_negative_infinite(-1.0e100));
/// ```
pub const fn is_negative_infinite(r: f64) -> bool {
    r <= -(0.5 * infinite())
}

/// Precision::Infinite(): OCCT's stand-in for an unbounded value.
///
/// ```
/// assert_eq!(rustycad_tkernel::precision::infinite(), 2.0e100);
/// ```
pub const fn infinite() -> f64 {
    2.0e100
}
