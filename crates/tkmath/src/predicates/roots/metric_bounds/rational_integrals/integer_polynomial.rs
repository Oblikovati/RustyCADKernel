// SPDX-License-Identifier: GPL-2.0-only
//! Integer monomial arithmetic for the subinterval Taylor model; index `k` holds `x^k`.
//! Exact dyadic scales are carried beside the integers, so no gcd normalization runs
//! inside the subinterval loop.
use super::super::super::{RootIsolationError, error::poll};
use num_bigint::{BigInt, Sign};

/// `sum_i coefficients[i] * scale^(n-i) * (offset + x)^i` with `n = len - 1`, by Horner.
/// For `t = (offset + x) / scale` this is `scale^n * p(t)` as a polynomial in `x`.
pub(super) fn shifted_scaled(
    coefficients: &[BigInt],
    offset: &BigInt,
    scale: &BigInt,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    let mut result: Vec<BigInt> = Vec::new();
    let mut power = BigInt::from(1);
    for coefficient in coefficients.iter().rev() {
        poll(cancelled)?;
        result = multiply_linear(&result, offset);
        if result.is_empty() {
            result.push(BigInt::from(0));
        }
        result[0] += coefficient * &power;
        power *= scale;
    }
    Ok(result)
}
/// Multiply by `(offset + x)`.
fn multiply_linear(polynomial: &[BigInt], offset: &BigInt) -> Vec<BigInt> {
    if polynomial.is_empty() {
        return Vec::new();
    }
    let mut result = vec![BigInt::from(0); polynomial.len() + 1];
    for (power, coefficient) in polynomial.iter().enumerate() {
        result[power] += coefficient * offset;
        result[power + 1] += coefficient;
    }
    result
}
/// `p(2u - 1)` as a polynomial in `u`.
pub(super) fn unit_from_symmetric(
    polynomial: &[BigInt],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    let mut result: Vec<BigInt> = Vec::new();
    for coefficient in polynomial.iter().rev() {
        poll(cancelled)?;
        let mut next = vec![BigInt::from(0); result.len() + 1];
        for (power, value) in result.iter().enumerate() {
            next[power] -= value;
            next[power + 1] += value * 2;
        }
        next[0] += coefficient;
        result = next;
    }
    Ok(result)
}
/// Scaled Bernstein coefficients `beta_i = C(n,i) b_i = sum_{j<=i} C(n-j, i-j) a_j`.
/// The common factor `C(n,i)` cancels from every ratio `beta_i(R) / beta_i(q)`.
pub(super) fn scaled_bernstein(
    monomial: &[BigInt],
    degree: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    let mut result = vec![BigInt::from(0); degree + 1];
    for (power, coefficient) in monomial.iter().enumerate() {
        poll(cancelled)?;
        if coefficient.sign() == Sign::NoSign {
            continue;
        }
        for (offset, binomial) in binomial_row(degree - power).into_iter().enumerate() {
            result[power + offset] += coefficient * binomial;
        }
    }
    Ok(result)
}
fn binomial_row(degree: usize) -> Vec<BigInt> {
    let mut row = vec![BigInt::from(1)];
    for index in 0..degree {
        let next = &row[index] * (degree - index) / (index + 1);
        row.push(next);
    }
    row
}
/// `left - right * 2^shift` for a nonnegative shift.
pub(super) fn shifted_difference(left: &[BigInt], right: &[BigInt], shift: usize) -> Vec<BigInt> {
    let mut result = vec![BigInt::from(0); left.len().max(right.len())];
    for (index, coefficient) in left.iter().enumerate() {
        result[index] += coefficient;
    }
    for (index, coefficient) in right.iter().enumerate() {
        result[index] -= coefficient << shift;
    }
    result
}
/// Approximate `c * 2^-shift` in binary64; only an approximant is chosen from it.
pub(super) fn approximate(value: &BigInt, shift: u64) -> f64 {
    let bits = value.bits();
    let dropped = bits.saturating_sub(62);
    let top = i64::try_from(value >> dropped).unwrap_or(0) as f64;
    let exponent = dropped as i64 - shift as i64;
    top * 2f64.powi(exponent.clamp(-2000, 2000) as i32)
}
/// Exact `(signed mantissa, exponent)` with `value = mantissa * 2^exponent`.
pub(super) fn decompose(value: f64) -> (i64, i64) {
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i64;
    let fraction = (bits & ((1_u64 << 52) - 1)) as i64;
    let (mantissa, power) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), exponent - 1075)
    };
    (
        if value.is_sign_negative() {
            -mantissa
        } else {
            mantissa
        },
        power,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn integers(values: &[i64]) -> Vec<BigInt> {
        values.iter().copied().map(BigInt::from).collect()
    }
    #[test]
    fn shifted_scaled_matches_hand_expansion() {
        // P(t) = 1 + t^2 at t = (3 + x) / 4, times 4^2: 16 + 9 + 6x + x^2.
        let shifted = shifted_scaled(&integers(&[1, 0, 1]), &3.into(), &4.into(), &mut || false);
        assert_eq!(shifted.unwrap(), integers(&[25, 6, 1]));
        assert!(shifted_scaled(&integers(&[1]), &0.into(), &1.into(), &mut || true).is_err());
    }
    #[test]
    fn unit_composition_and_scaled_bernstein_match_hand_forms() {
        // x^2 at x = 2u - 1 is 1 - 4u + 4u^2; degree-2 scaled Bernstein (1, -2, 1).
        let unit = unit_from_symmetric(&integers(&[0, 0, 1]), &mut || false).unwrap();
        assert_eq!(unit, integers(&[1, -4, 4]));
        let bernstein = scaled_bernstein(&unit, 2, &mut || false).unwrap();
        assert_eq!(bernstein, integers(&[1, -2, 1]));
        // Elevation to degree 3 multiplies the ordinary coefficients (1,-1/3,-1/3,1) by C(3,i).
        let elevated = scaled_bernstein(&unit, 3, &mut || false).unwrap();
        assert_eq!(elevated, integers(&[1, -1, -1, 1]));
    }
    #[test]
    fn approximation_and_decomposition_round_trip() {
        assert_eq!(approximate(&BigInt::from(3), 1), 1.5);
        let huge = BigInt::from(5) << 300usize;
        assert_eq!(approximate(&huge, 300), 5.);
        for value in [1.5, -0.1, f64::from_bits(1), -f64::MAX] {
            let (mantissa, exponent) = decompose(value);
            let half = (exponent / 2) as i32;
            let rebuilt = mantissa as f64 * 2f64.powi(half) * 2f64.powi(exponent as i32 - half);
            assert_eq!(rebuilt, value);
        }
        assert_eq!(
            shifted_difference(&integers(&[5, 1]), &integers(&[1]), 2),
            integers(&[1, 1])
        );
    }
}
