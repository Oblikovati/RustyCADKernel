//! rustc_apfloat is isolated here to reproduce the pinned GCC x87 long-double profile.
use rustc_apfloat::ieee::{Double, X87DoubleExtended as Extended};
use rustc_apfloat::{Float, FloatConvert};
fn extend(value: f64) -> Extended {
    Double::from_bits(u128::from(value.to_bits()))
        .convert(&mut false)
        .value
}
fn narrow(value: Extended) -> f64 {
    let result: Double = value.convert(&mut false).value;
    f64::from_bits(result.to_bits() as u64)
}
pub(super) fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return f64::NAN;
    }
    let mut sum: Extended = Extended::ZERO;
    let mut correction: Extended = Extended::ZERO;
    for (&left, &right) in a.iter().zip(b) {
        if !left.is_finite() || !right.is_finite() {
            return f64::NAN;
        }
        let product: Extended = (extend(left) * extend(right)).value;
        let next: Extended = (sum + product).value;
        let residual: Extended = if sum.abs() >= product.abs() {
            ((sum - next).value + product).value
        } else {
            ((product - next).value + sum).value
        };
        correction = (correction + residual).value;
        sum = next;
    }
    narrow((sum + correction).value)
}
