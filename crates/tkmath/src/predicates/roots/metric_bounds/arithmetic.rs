// SPDX-License-Identifier: GPL-2.0-only
//! Raw products preserve coefficient magnitude until the final sign-only conversion.
use super::super::{RootIsolationError, error::poll};
use num_bigint::BigInt;
pub(super) fn product(
    left: &[BigInt],
    right: &[BigInt],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    let mut coefficients = vec![BigInt::from(0); left.len() + right.len() - 1];
    for (i, first) in left.iter().enumerate() {
        for (j, second) in right.iter().enumerate() {
            poll(cancelled)?;
            coefficients[i + j] += first * second;
        }
    }
    Ok(coefficients)
}
pub(super) fn difference(left: Vec<BigInt>, right: Vec<BigInt>) -> Vec<BigInt> {
    let mut result = vec![BigInt::from(0); left.len().max(right.len())];
    for (index, coefficient) in left.into_iter().enumerate() {
        result[index] += coefficient;
    }
    for (index, coefficient) in right.into_iter().enumerate() {
        result[index] -= coefficient;
    }
    result
}
pub(super) fn accumulate(target: &mut Vec<BigInt>, contribution: Vec<BigInt>) {
    target.resize(target.len().max(contribution.len()), BigInt::from(0));
    for (index, coefficient) in contribution.into_iter().enumerate() {
        target[index] += coefficient;
    }
}
