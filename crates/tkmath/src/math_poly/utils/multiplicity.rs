//! `HasMultiplicity` and `HasComplexMultiplicity`: Newton refinement on the (m-1)-th
//! derivative, then a long-double check that derivatives 0..m-1 vanish at the refined root.
use super::preconditions::bounded_degree;
use super::{LongCoefficients, derivative, horner, widened};
use crate::math_poly::Complex;
use crate::math_poly::complex::long::LongComplex;
use crate::math_poly::extended::LongDouble;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
/// `THE_MULTIPLICITY_REFINEMENT_MAX_ITERATIONS`: Newton steps on the (m-1)-th derivative.
pub const MULTIPLICITY_REFINEMENT_MAX_ITERATIONS: u32 = 64;
/// The (m-1)-th derivative and its degree.
fn derivative_of_order(c: &LongCoefficients, degree: i32, order: usize) -> (LongCoefficients, i32) {
    (0..order).fold((*c, degree), |(current, d), _| {
        (derivative(&current, d), d - 1)
    })
}
fn computational() -> LongDouble {
    LongDouble::new(precision::computational())
}
/// Native code copies `c[0..=degree]` into a `THE_MAX_POLY_DEGREE + 1` array and reads
/// derivative `multiplicity - 1` at index `degree - multiplicity + 1`, so it needs
/// `0 <= degree <= MAX_POLY_DEGREE`, `degree + 1` coefficients and
/// `multiplicity <= degree + 1`.
fn check_multiplicity(
    member: &str,
    len: usize,
    degree: i32,
    multiplicity: usize,
) -> Result<(), Failure> {
    let d: usize = bounded_degree(member, len, degree, 0)?;
    if multiplicity > d + 1 {
        return Err(Failure::out_of_range(&format!(
            "{member}: multiplicity {multiplicity} of a degree {degree} polynomial; expected \
             multiplicity <= degree + 1"
        )));
    }
    Ok(())
}
/// `HasMultiplicity`: true when `root` is a root of multiplicity `multiplicity`; then the
/// refined root replaces `root`. Standard_OutOfRange outside the shape
/// `check_multiplicity` names; `utils::has_multiplicity(&c, 2, &mut root, 2)?`.
pub fn has_multiplicity(
    coefficients: &[f64],
    degree: i32,
    root: &mut f64,
    multiplicity: usize,
) -> Result<bool, Failure> {
    check_multiplicity(
        "MathPoly::Utils::HasMultiplicity",
        coefficients.len(),
        degree,
        multiplicity,
    )?;
    let polynomial: LongCoefficients = widened(coefficients, degree);
    let (target, target_degree) =
        derivative_of_order(&polynomial, degree, multiplicity.saturating_sub(1));
    let Some(refined) = refined_real(&target, target_degree, LongDouble::new(*root)) else {
        return Ok(false);
    };
    let (mut current, mut current_degree) = (polynomial, degree);
    for _ in 0..multiplicity {
        let value: LongDouble = horner(&current, current_degree, refined).0;
        let d: usize = current_degree as usize;
        let norm: LongDouble = current[..d]
            .iter()
            .rev()
            .fold(current[d].abs(), |s, x| s + x.abs());
        if !value.is_finite() || value.abs() > computational() * LongDouble::new(1.0).max(norm) {
            return Ok(false);
        }
        current = derivative(&current, current_degree);
        current_degree -= 1;
    }
    *root = refined.narrow();
    Ok(true)
}
/// Newton iteration; `None` when a value, derivative or iterate stops being finite.
fn refined_real(c: &LongCoefficients, degree: i32, mut root: LongDouble) -> Option<LongDouble> {
    for _ in 0..MULTIPLICITY_REFINEMENT_MAX_ITERATIONS {
        let (value, slope, _) = horner(c, degree, root);
        if !value.is_finite() || !slope.is_finite() {
            return None;
        }
        if slope == LongDouble::ZERO {
            break;
        }
        let next: LongDouble = root - value / slope;
        if !next.is_finite() {
            return None;
        }
        if next == root {
            break;
        }
        root = next;
    }
    Some(root)
}
/// Complex Horner value and derivative with GCC `__mulxc3` products.
fn evaluated_complex(
    c: &LongCoefficients,
    degree: i32,
    x: LongComplex,
) -> (LongComplex, LongComplex) {
    let d: usize = degree as usize;
    let mut value: LongComplex = LongComplex::real(c[d]);
    let mut slope: LongComplex = LongComplex::real(LongDouble::ZERO);
    for &coefficient in c[..d].iter().rev() {
        slope = slope * x + value;
        value = (value * x).plus_real(coefficient);
    }
    (value, slope)
}
fn refined_complex(
    c: &LongCoefficients,
    degree: i32,
    mut root: LongComplex,
) -> Option<LongComplex> {
    for _ in 0..MULTIPLICITY_REFINEMENT_MAX_ITERATIONS {
        let (value, slope) = evaluated_complex(c, degree, root);
        if !value.is_finite() || !slope.is_finite() {
            return None;
        }
        if slope.abs() == LongDouble::ZERO {
            break;
        }
        let next: LongComplex = root - value / slope;
        if !next.is_finite() {
            return None;
        }
        if next == root {
            break;
        }
        root = next;
    }
    Some(root)
}
/// Absolute value bound `sum |c_i| |x|^i`, evaluated in native Horner order.
fn complex_magnitude(c: &LongCoefficients, degree: i32, abs_root: LongDouble) -> LongDouble {
    let d: usize = degree as usize;
    c[..d]
        .iter()
        .rev()
        .fold(c[d].abs(), |m, x| m * abs_root + x.abs())
}
/// `HasComplexMultiplicity`: complex counterpart of [`has_multiplicity`], with the same
/// Standard_OutOfRange preconditions; refines `root` in place on success;
/// `utils::has_complex_multiplicity(&c, 4, &mut root, 2)?`.
pub fn has_complex_multiplicity(
    coefficients: &[f64],
    degree: i32,
    root: &mut Complex,
    multiplicity: usize,
) -> Result<bool, Failure> {
    check_multiplicity(
        "MathPoly::Utils::HasComplexMultiplicity",
        coefficients.len(),
        degree,
        multiplicity,
    )?;
    let polynomial: LongCoefficients = widened(coefficients, degree);
    let start: LongComplex =
        LongComplex::new(LongDouble::new(root.real()), LongDouble::new(root.imag()));
    let (target, target_degree) =
        derivative_of_order(&polynomial, degree, multiplicity.saturating_sub(1));
    let Some(refined) = refined_complex(&target, target_degree, start) else {
        return Ok(false);
    };
    let (mut current, mut current_degree) = (polynomial, degree);
    for _ in 0..multiplicity {
        let value: LongComplex = evaluated_complex(&current, current_degree, refined).0;
        let magnitude: LongDouble = complex_magnitude(&current, current_degree, refined.abs());
        if !value.is_finite() || value.abs() > computational() * LongDouble::new(1.0).max(magnitude)
        {
            return Ok(false);
        }
        current = derivative(&current, current_degree);
        current_degree -= 1;
    }
    *root = Complex::new(refined.re.narrow(), refined.im.narrow());
    Ok(true)
}
