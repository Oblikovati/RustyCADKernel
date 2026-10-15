// SPDX-License-Identifier: GPL-2.0-only
//! Exact elimination of `u` from two polynomials in `u` with coefficients in `s`.
//!
//! The resultant `Res_u(f, g)(s)` and the first subresultant `alpha(s) u + beta(s)`
//! are Sylvester-type determinants whose entries are polynomials in `s`. They are
//! recovered exactly by Lagrange interpolation of the specialized determinants at
//! integer samples, one more than a degree bound from the formal degrees. With
//! `lc_u f(s*) != 0`, the specializations commute with the subresultants, so at
//! a common root `s*` with `alpha(s*) != 0` the two equations share exactly the
//! one root `u* = -beta(s*) / alpha(s*)`.
use super::super::certificate_arithmetic::ExactRatio;
use super::rational_polynomial::{RationalPolynomial, zero};
use super::resultant::{determinant, integer, interpolate, sylvester_determinant};

/// Polynomial in `u` whose coefficients (ascending in `u`) are polynomials in `s`.
#[derive(Clone, Debug)]
pub(super) struct Bivariate(pub Vec<RationalPolynomial>);
/// The fibre row `alpha(s) u + beta(s)` of a pair, as polynomials in `s`.
pub(super) struct FibreRow {
    pub alpha: RationalPolynomial,
    pub beta: RationalPolynomial,
}
impl Bivariate {
    /// Formal degree in `u`; None when every coefficient vanishes.
    pub fn degree(&self) -> Option<usize> {
        self.0
            .iter()
            .rposition(|coefficient| coefficient.degree().is_some())
    }
    fn s_degree(&self) -> usize {
        self.0
            .iter()
            .filter_map(RationalPolynomial::degree)
            .max()
            .unwrap_or(0)
    }
    /// The univariate polynomial in `u` at `s = value`.
    fn at(&self, value: &ExactRatio) -> RationalPolynomial {
        RationalPolynomial(
            self.0
                .iter()
                .map(|coefficient| coefficient.evaluate(value))
                .collect(),
        )
    }
    /// Leading coefficient in `u` at the formal degree.
    pub fn leading(&self, degree: usize) -> RationalPolynomial {
        self.0
            .get(degree)
            .cloned()
            .unwrap_or(RationalPolynomial(Vec::new()))
    }
}
/// `Res_u(f, g)` with formal degrees `m` and `n` in `u`.
pub(super) fn resultant(
    first: &Bivariate,
    [m, n]: [usize; 2],
    second: &Bivariate,
) -> RationalPolynomial {
    let bound = m * second.s_degree() + n * first.s_degree();
    let samples: Vec<ExactRatio> = (0..=bound)
        .map(|point| {
            let value = integer(point);
            sylvester_determinant(&first.at(&value), m, &second.at(&value), n)
        })
        .collect();
    interpolate(&samples)
}
/// The fibre row: a linear equation itself, or the first subresultant for `m, n >= 2`.
pub(super) fn fibre_row(
    first: &Bivariate,
    [m, n]: [usize; 2],
    second: &Bivariate,
) -> Option<FibreRow> {
    let linear = |equation: &Bivariate| FibreRow {
        alpha: equation.leading(1),
        beta: equation.leading(0),
    };
    match (m, n) {
        (1, _) => Some(linear(first)),
        (_, 1) => Some(linear(second)),
        (0, _) | (_, 0) => None,
        _ => Some(first_subresultant(first, [m, n], second)),
    }
}
fn first_subresultant(first: &Bivariate, [m, n]: [usize; 2], second: &Bivariate) -> FibreRow {
    let bound = (m + n - 2) * first.s_degree().max(second.s_degree());
    let mut alpha = Vec::with_capacity(bound + 1);
    let mut beta = Vec::with_capacity(bound + 1);
    for point in 0..=bound {
        let value = integer(point);
        let rows = subresultant_rows(&first.at(&value), m, &second.at(&value), n);
        alpha.push(determinant(with_last_column(&rows, m + n - 3, 1)));
        beta.push(determinant(with_last_column(&rows, m + n - 3, 0)));
    }
    FibreRow {
        alpha: interpolate(&alpha),
        beta: interpolate(&beta),
    }
}
// Rows `u^(n-2) f, ..., f, u^(m-2) g, ..., g` as descending coefficients of
// `u^(m+n-2) .. u^0`.
fn subresultant_rows(
    first: &RationalPolynomial,
    m: usize,
    second: &RationalPolynomial,
    n: usize,
) -> Vec<Vec<ExactRatio>> {
    let width = m + n - 1;
    let shifted = |polynomial: &RationalPolynomial, degree: usize, shift: usize| {
        let mut row = vec![zero(); width];
        for power in 0..=degree {
            row[width - 1 - (power + shift)] = polynomial.coefficient(power);
        }
        row
    };
    let mut rows: Vec<Vec<ExactRatio>> = (0..n - 1)
        .rev()
        .map(|shift| shifted(first, m, shift))
        .collect();
    rows.extend((0..m - 1).rev().map(|shift| shifted(second, n, shift)));
    rows
}
// The first `leading` columns (powers `u^(m+n-2) .. u^2`) and the column of `u^power`.
fn with_last_column(
    rows: &[Vec<ExactRatio>],
    leading: usize,
    power: usize,
) -> Vec<Vec<ExactRatio>> {
    let width = rows[0].len();
    rows.iter()
        .map(|row| {
            let mut kept = row[..leading].to_vec();
            kept.push(row[width - 1 - power].clone());
            kept
        })
        .collect()
}
