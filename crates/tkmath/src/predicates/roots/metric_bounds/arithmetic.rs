// SPDX-License-Identifier: GPL-2.0-only
//! Raw products preserve coefficient magnitude until the final sign-only conversion.
pub(super) use super::super::certificate_arithmetic::convolution::raw_coefficient_product as product;
use num_bigint::BigInt;
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
