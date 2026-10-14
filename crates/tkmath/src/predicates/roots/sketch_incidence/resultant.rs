// SPDX-License-Identifier: GPL-2.0-only
//! Line-parameter equation of a parametric target: `R(T) = Res_u(N(u), L(u) - m T W(u))`.
//!
//! At every real root `u*` of `N`, `t* = L(u*) / (m W(u*))` is a root of `R`, so all
//! contacts on the source line become roots of integer polynomials in one variable
//! and can be ordered and tested for equality exactly. `R` has degree at most
//! `deg N` in `T`; it is recovered exactly by Lagrange interpolation of Sylvester
//! determinants at `T = 0..=deg N`, with formal (not reduced) degrees in `u`.
use super::super::certificate_arithmetic::ExactRatio;
use super::rational_polynomial::{RationalPolynomial, ratio, zero};
use std::cmp::Ordering;

/// Exact `R(T)`; None when it vanishes identically (a common complex base point).
pub(super) fn line_parameter_equation(
    normal: &RationalPolynomial,
    along: &RationalPolynomial,
    weight: &RationalPolynomial,
    squared_length: &ExactRatio,
) -> Option<RationalPolynomial> {
    let degree = normal.degree()?;
    let formal = along
        .degree()
        .unwrap_or(0)
        .max(weight.degree().unwrap_or(0));
    let samples: Vec<ExactRatio> = (0..=degree)
        .map(|point| {
            let shift = weight.scale(&squared_length.multiply(&integer(point)));
            let difference = along.add(&shift.scale(&ratio(-1.)));
            sylvester_determinant(normal, degree, &difference, formal)
        })
        .collect();
    let interpolated = interpolate(&samples);
    interpolated.degree().map(|_| interpolated)
}
fn integer(value: usize) -> ExactRatio {
    ExactRatio::new(value.into(), 1.into())
}
fn sylvester_determinant(
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
fn determinant(mut matrix: Vec<Vec<ExactRatio>>) -> ExactRatio {
    let mut product = ratio(1.);
    for column in 0..matrix.len() {
        let Some(pivot) =
            (column..matrix.len()).find(|&row| matrix[row][column].sign() != Ordering::Equal)
        else {
            return zero();
        };
        if pivot != column {
            matrix.swap(pivot, column);
            product = product.negated();
        }
        product = product.multiply(&matrix[column][column]);
        eliminate_below(&mut matrix, column);
    }
    product
}
fn eliminate_below(matrix: &mut [Vec<ExactRatio>], column: usize) {
    let (upper, lower) = matrix.split_at_mut(column + 1);
    let pivot_row = &upper[column];
    for row in lower {
        let factor = row[column].divide(&pivot_row[column]);
        if factor.sign() == Ordering::Equal {
            continue;
        }
        for index in column..row.len() {
            row[index] = row[index].subtract(&factor.multiply(&pivot_row[index]));
        }
    }
}
/// Exact Lagrange interpolation through `(k, samples[k])`.
fn interpolate(samples: &[ExactRatio]) -> RationalPolynomial {
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
