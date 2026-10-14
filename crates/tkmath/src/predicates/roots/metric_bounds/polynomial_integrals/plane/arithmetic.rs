// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{DyadicBound, affine_support::ExactSupportColumns};
use super::*;
use crate::predicates::roots::certificate_arithmetic::rounding::{
    outward_binary64, outward_sqrt_bounds,
};
use num_bigint::BigInt;

pub(super) fn gram_determinant(columns: &ExactSupportColumns) -> ExactRatio {
    let [a, b] = &columns.columns;
    let mixed = dot_product(a, b);
    let determinant = dot_product(a, a) * dot_product(b, b) - &mixed * &mixed;
    let dyadic = DyadicBound::new(determinant, -2 * columns.product_denominator_exponent);
    ExactRatio::from_dyadic(&dyadic)
}
fn dot_product(first: &[BigInt; 3], second: &[BigInt; 3]) -> BigInt {
    // Integer products preserve cancellation in nearly dependent support columns.
    first.iter().zip(second).map(|(a, b)| a * b).sum()
}
pub(super) fn rounded_plane_measure(
    square: &ExactRatio,
    options: PolynomialIntegralOptions,
) -> Result<CertifiedPolynomialPlaneMeasure, PolynomialPlaneFailure> {
    let enclosure = outward_sqrt_bounds(square).ok_or_else(|| {
        PolynomialPlaneFailure::UnrepresentableSquaredMeasure(format!("{square:?}"))
    })?;
    let absolute_error_bound = enclosure_error(enclosure, options.absolute_error_limit)?;
    Ok(CertifiedPolynomialPlaneMeasure {
        enclosure,
        absolute_error_bound,
        options,
    })
}
fn enclosure_error(enclosure: [f64; 2], requested: f64) -> Result<f64, PolynomialPlaneFailure> {
    let width =
        ExactRatio::from_finite(enclosure[1]).subtract(&ExactRatio::from_finite(enclosure[0]));
    // Finite nonnegative adjacent endpoints have a representable width, including subnormals.
    let absolute_error_bound = outward_binary64(&width).expect("finite adjacent endpoint width")[1];
    if absolute_error_bound > requested {
        return Err(PolynomialPlaneFailure::ErrorLimit {
            requested,
            achieved: absolute_error_bound,
        });
    }
    Ok(absolute_error_bound)
}
