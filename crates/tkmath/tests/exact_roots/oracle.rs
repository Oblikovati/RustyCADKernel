// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use num_bigint::{BigInt, Sign};

#[test]
fn every_root_matches_independent_jacobian_identity_and_exact_endpoint_certificate() {
    let corpus = include_str!("oracle.tsv");
    assert_eq!(corpus.lines().count(), 22);
    for line in corpus.lines() {
        verify_fixture(line);
    }
}
fn verify_fixture(line: &str) {
    let fields: Vec<&str> = line.split('\t').collect();
    let coefficients: Vec<f64> = fields[1]
        .split(',')
        .map(|bits| f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
        .collect();
    let roots: Vec<&str> = fields[2]
        .split('|')
        .filter(|root| !root.is_empty())
        .collect();
    let result = isolate(&coefficients, [-16., 16.]);
    assert_eq!(result.roots().len(), roots.len(), "{}", fields[0]);
    verify_order_and_width(&result, 0.0001);
    for scale in [-1., 2_f64.powi(900), 2_f64.powi(-900)] {
        let scaled: Vec<f64> = coefficients.iter().map(|value| value * scale).collect();
        assert_eq!(
            isolate(&scaled, [-16., 16.]),
            result,
            "{} scaled by {scale}",
            fields[0]
        );
    }
    for (root, expected) in result.roots().iter().zip(roots) {
        certify_interval(root, expected);
    }
}
pub(super) fn certify_interval(root: &RootBracket, expected: &str) {
    let (coefficients, multiplicity) = expected.split_once(':').unwrap();
    let coefficients: Vec<BigInt> = coefficients
        .split(',')
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(root.multiplicity(), multiplicity.parse::<usize>().unwrap());
    let lower = endpoint_sign(&coefficients, root.bounds().0);
    let upper = endpoint_sign(&coefficients, root.bounds().1);
    if root.bounds().0 == root.bounds().1 {
        assert_eq!(lower, Sign::NoSign);
        return;
    }
    assert_ne!(lower, Sign::NoSign);
    assert_ne!(upper, Sign::NoSign);
    assert_ne!(
        lower, upper,
        "minimal polynomial must cross each certified interval"
    );
}
fn endpoint_sign(coefficients: &[BigInt], bound: &DyadicBound) -> Sign {
    let text = bound.to_string();
    let (significand, exponent) = text.split_once(" * 2^").unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let integer: BigInt = significand.parse().unwrap();
    let numerator = if exponent > 0 {
        integer << exponent as usize
    } else {
        integer
    };
    let denominator = BigInt::from(1) << (-exponent).max(0) as usize;
    // Independent direct power sum, not the production homogeneous Horner path.
    coefficients
        .iter()
        .enumerate()
        .map(|(index, coefficient)| {
            coefficient
                * numerator.pow((coefficients.len() - index - 1) as u32)
                * denominator.pow(index as u32)
        })
        .sum::<BigInt>()
        .sign()
}
