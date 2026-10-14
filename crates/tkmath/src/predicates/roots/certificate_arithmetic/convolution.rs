// SPDX-License-Identifier: GPL-2.0-only
//! Raw products retain coefficient magnitude and cancellation order before normalization.
use super::super::{RootIsolationError, error::poll};
use num_bigint::BigInt;

pub(in crate::predicates::roots) fn raw_coefficient_product(
    left: &[BigInt],
    right: &[BigInt],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    if left.is_empty() || right.is_empty() {
        return Ok(Vec::new());
    }
    let mut coefficients = vec![BigInt::from(0); left.len() + right.len() - 1];
    accumulate_product(left, right, &mut coefficients, cancelled)?;
    Ok(coefficients)
}
fn accumulate_product(
    left: &[BigInt],
    right: &[BigInt],
    coefficients: &mut [BigInt],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), RootIsolationError> {
    for (i, left_term) in left.iter().enumerate() {
        for (j, right_term) in right.iter().enumerate() {
            poll(cancelled)?;
            coefficients[i + j] += left_term * right_term;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dense_product_retains_content_and_trailing_zero() {
        let result =
            raw_coefficient_product(&[2.into(), 2.into()], &[3.into(), 0.into()], &mut || false)
                .unwrap();
        assert_eq!(result, vec![6.into(), 6.into(), 0.into()]);
        assert!(
            raw_coefficient_product(&[], &[1.into()], &mut || panic!(
                "empty product must not poll"
            ))
            .unwrap()
            .is_empty()
        );
    }
    #[test]
    fn cancellation_polls_before_each_left_major_accumulation() {
        let mut polls = 0;
        let result =
            raw_coefficient_product(&[1.into(), 2.into()], &[3.into(), 4.into()], &mut || {
                polls += 1;
                polls == 3
            });
        assert_eq!(result, Err(RootIsolationError::Cancelled));
        assert_eq!(polls, 3);
    }
}
