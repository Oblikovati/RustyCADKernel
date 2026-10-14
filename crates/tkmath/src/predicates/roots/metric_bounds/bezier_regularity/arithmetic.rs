// SPDX-License-Identifier: GPL-2.0-only
//! Construct magnitudes before reducing polynomial content for root isolation.
use super::super::super::{RootIsolationError, polynomial::IntegerPolynomial};
use super::super::{
    arithmetic::{accumulate, difference, product},
    coefficients::HomogeneousCurve,
};
use num_bigint::BigInt;

pub(super) fn squared_speed(
    curve: &HomogeneousCurve,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    let denominator = &curve[3];
    let derivative = IntegerPolynomial(denominator.clone()).derivative();
    let mut square = vec![BigInt::from(0)];
    for numerator in &curve[..3] {
        let tangent = derivative_numerator(numerator, denominator, &derivative.0, cancelled)?;
        accumulate(&mut square, product(&tangent, &tangent, cancelled)?);
    }
    Ok(square)
}
fn derivative_numerator(
    numerator: &[BigInt],
    denominator: &[BigInt],
    denominator_derivative: &[BigInt],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    let derivative = IntegerPolynomial(numerator.to_vec()).derivative();
    let first = product(&derivative.0, denominator, cancelled)?;
    let second = product(numerator, denominator_derivative, cancelled)?;
    Ok(difference(first, second))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn regularity_raw_square_preserves_independent_cubic_identity() {
        // C=(12t-24t²+12t³,12t²-12t³,0), D=1.
        let curve = [
            vec![0.into(), 12.into(), (-24).into(), 12.into()],
            vec![0.into(), 0.into(), 12.into(), (-12).into()],
            vec![0.into()],
            vec![1.into()],
        ];
        let actual = squared_speed(&curve, &mut || false).unwrap();
        let expected: Vec<BigInt> = [144, -1152, 3744, -5184, 2592]
            .into_iter()
            .map(BigInt::from)
            .collect();
        assert_eq!(actual, expected);
        // Uniform homogeneous scaling multiplies Q by scale^4, never changes its roots.
        let scaled = curve.map(|axis| axis.into_iter().map(|n| n * -3).collect());
        assert_eq!(
            squared_speed(&scaled, &mut || false).unwrap(),
            expected.into_iter().map(|n| n * 81).collect::<Vec<_>>()
        );
    }
    #[test]
    fn regularity_common_rational_factor_does_not_cancel_before_derivation() {
        // N=t(2-t), D=2-t represents t; H=D², Q=D^4.
        let curve = [
            vec![0.into(), 2.into(), (-1).into()],
            vec![0.into()],
            vec![0.into()],
            vec![2.into(), (-1).into()],
        ];
        assert_eq!(
            squared_speed(&curve, &mut || false).unwrap(),
            [16, -32, 24, -8, 1].map(BigInt::from)
        );
    }
}
