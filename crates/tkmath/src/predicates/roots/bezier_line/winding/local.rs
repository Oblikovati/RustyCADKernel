// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::RootBracket;
use super::{BezierRayError, DyadicBound, IntegerPolynomial, ray};
use num_bigint::Sign;
pub(super) fn contribution(
    normal: &IntegerPolynomial,
    denominator: &IntegerPolynomial,
    root: &RootBracket,
    bounds: &[DyadicBound; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<i32, BezierRayError> {
    let after = right_sign(normal, denominator, root, cancelled)?;
    let before = if root.multiplicity().is_multiple_of(2) {
        after
    } else {
        -after
    };
    if root.bounds().0 == &bounds[0] && root.bounds().0 == root.bounds().1 {
        return Ok(i32::from(after > 0));
    }
    if root.bounds().1 == &bounds[1] && root.bounds().0 == root.bounds().1 {
        return Ok(-i32::from(before > 0));
    }
    Ok((after - before) / 2)
}
fn right_sign(
    normal: &IntegerPolynomial,
    denominator: &IntegerPolynomial,
    root: &RootBracket,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<i32, BezierRayError> {
    let polynomial = if root.bounds().0 == root.bounds().1 {
        derivative_at_order(normal, root.multiplicity(), cancelled)?
    } else {
        normal.clone()
    };
    let normal_sign = integer_sign(polynomial.sign_at(root.bounds().1));
    let denominator_sign = integer_sign(denominator.sign_at(root.bounds().1));
    let after = normal_sign * denominator_sign;
    if after == 0 {
        return Err(BezierRayError::LocalNormalSign(root.clone()));
    }
    Ok(after)
}
fn derivative_at_order(
    normal: &IntegerPolynomial,
    order: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<IntegerPolynomial, BezierRayError> {
    let mut derivative = normal.clone();
    for _ in 0..order {
        super::super::super::error::poll(cancelled).map_err(ray::normal_error)?;
        derivative = derivative.derivative();
    }
    Ok(derivative)
}
fn integer_sign(sign: Sign) -> i32 {
    match sign {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[test]
    fn inconsistent_root_certificate_refuses_zero_local_sign() {
        let bound = DyadicBound::from_f64(0.).unwrap();
        let root = RootBracket {
            lower: bound.clone(),
            upper: bound,
            multiplicity: 1,
        };
        let normal = IntegerPolynomial::from_binary64(&[1.], 1).unwrap();
        let error = right_sign(&normal, &normal, &root, &mut || false).unwrap_err();
        assert_eq!(error, BezierRayError::LocalNormalSign(root));
        assert!(error.to_string().contains("expected nonzero"));
        assert!(error.source().is_none());
    }
}
