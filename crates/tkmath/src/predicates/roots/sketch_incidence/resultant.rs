// SPDX-License-Identifier: GPL-2.0-only
//! Image equations of a rational map at the roots of a contact equation:
//! `R(T) = Res_v(N(v), P(v) - T Q(v))`.
//!
//! At every real root `v*` of `N` with `Q(v*) != 0`, `T* = P(v*) / Q(v*)` is a root
//! of `R`, so a source key, a target parameter or a fibre coordinate becomes the
//! root of one integer polynomial and can be ordered and tested for equality
//! exactly. `R` has degree at most `deg N` in `T`; it is recovered exactly by
//! Lagrange interpolation of Sylvester determinants at `T = 0..=deg N`, with
//! formal (not reduced) degrees in `v`.
use super::super::certificate_arithmetic::ExactRatio;
use super::rational_polynomial::{RationalPolynomial, common_denominator, ratio, zero};
use num_bigint::{BigInt, Sign};

/// Exact `R(T)`; None when it vanishes identically (a common complex base point).
pub(super) fn image_equation(
    contact: &RationalPolynomial,
    numerator: &RationalPolynomial,
    denominator: &RationalPolynomial,
) -> Option<RationalPolynomial> {
    // Positive common denominators change neither the roots of `N` nor `P/Q`, and
    // keep every sampled Sylvester determinant integral.
    let contact = &contact.scale(&common_denominator(&[contact]));
    let shared = common_denominator(&[numerator, denominator]);
    let (numerator, denominator) = (&numerator.scale(&shared), &denominator.scale(&shared));
    let degree = contact.degree()?;
    let formal = numerator
        .degree()
        .unwrap_or(0)
        .max(denominator.degree().unwrap_or(0));
    let samples: Vec<ExactRatio> = (0..=degree)
        .map(|point| {
            let shift = denominator.scale(&integer(point));
            let difference = numerator.add(&shift.scale(&ratio(-1.)));
            sylvester_determinant(contact, degree, &difference, formal)
        })
        .collect();
    let interpolated = interpolate(&samples);
    interpolated.degree().map(|_| interpolated)
}
pub(super) fn integer(value: usize) -> ExactRatio {
    ExactRatio::new(value.into(), 1.into())
}
pub(super) fn sylvester_determinant(
    first: &RationalPolynomial,
    first_degree: usize,
    second: &RationalPolynomial,
    second_degree: usize,
) -> ExactRatio {
    let size = first_degree + second_degree;
    if size == 0 {
        return ratio(1.);
    }
    let mut matrix = vec![vec![zero(); size]; size];
    fill_rows(&mut matrix[..second_degree], first, first_degree);
    fill_rows(&mut matrix[second_degree..], second, second_degree);
    determinant(matrix)
}
// Descending coefficients, shifted one column per row.
fn fill_rows(rows: &mut [Vec<ExactRatio>], polynomial: &RationalPolynomial, degree: usize) {
    for (shift, row) in rows.iter_mut().enumerate() {
        for power in 0..=degree {
            row[shift + degree - power] = polynomial.coefficient(power);
        }
    }
}
/// Exact determinant: rows are scaled to integers, then eliminated fraction-free
/// (Bareiss), whose every division is exact.
pub(super) fn determinant(matrix: Vec<Vec<ExactRatio>>) -> ExactRatio {
    if matrix.is_empty() {
        return ratio(1.);
    }
    let mut scale = BigInt::from(1);
    let mut rows: Vec<Vec<BigInt>> = matrix
        .iter()
        .map(|row| {
            let common = row.iter().fold(BigInt::from(1), |lcm, value| {
                let divisor = gcd(value.denominator().clone(), lcm.clone());
                lcm / divisor * value.denominator()
            });
            scale *= &common;
            row.iter()
                .map(|value| value.numerator() * (&common / value.denominator()))
                .collect()
        })
        .collect();
    let determinant = bareiss(&mut rows);
    ExactRatio::new(determinant, scale)
}
fn bareiss(rows: &mut [Vec<BigInt>]) -> BigInt {
    let size = rows.len();
    let mut previous = BigInt::from(1);
    let mut negated = false;
    for column in 0..size {
        let Some(pivot) = (column..size).find(|&row| rows[row][column].sign() != Sign::NoSign)
        else {
            return BigInt::from(0);
        };
        if pivot != column {
            rows.swap(pivot, column);
            negated = !negated;
        }
        eliminate_fraction_free(rows, column, &previous);
        previous = rows[column][column].clone();
    }
    let last = rows[size - 1][size - 1].clone();
    if negated { -last } else { last }
}
fn gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    while right.sign() != Sign::NoSign {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    if left.sign() == Sign::Minus {
        -left
    } else {
        left
    }
}
// `a[i][j] = (a[i][j] a[k][k] - a[i][k] a[k][j]) / previous pivot`, exactly.
fn eliminate_fraction_free(rows: &mut [Vec<BigInt>], column: usize, previous: &BigInt) {
    let (upper, lower) = rows.split_at_mut(column + 1);
    let pivot_row = &upper[column];
    for row in lower {
        let factor = row[column].clone();
        for index in column + 1..row.len() {
            row[index] =
                (&row[index] * &pivot_row[column] - &factor * &pivot_row[index]) / previous;
        }
        row[column] = BigInt::from(0);
    }
}
/// Exact Lagrange interpolation through `(k, samples[k])`.
pub(super) fn interpolate(samples: &[ExactRatio]) -> RationalPolynomial {
    let mut total = RationalPolynomial(vec![zero()]);
    for (node, value) in samples.iter().enumerate() {
        let mut basis = RationalPolynomial::constant(value.clone());
        for other in (0..samples.len()).filter(|other| *other != node) {
            let denominator = integer(node).subtract(&integer(other));
            let factor = RationalPolynomial::linear(integer(other).negated(), ratio(1.));
            basis = basis
                .multiply(&factor)
                .scale(&ratio(1.).divide(&denominator));
        }
        total = total.add(&basis);
    }
    total
}
