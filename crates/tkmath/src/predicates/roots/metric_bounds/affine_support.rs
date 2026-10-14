// SPDX-License-Identifier: GPL-2.0-only
//! Shared exact placed plane columns; extraction preserves the Jacobian's two ordered polls.
use super::{MetricCurve, MetricEmbedding, ResidualError, coefficients};
use num_bigint::BigInt;

pub(super) struct ExactSupportColumns {
    pub columns: [[BigInt; 3]; 2],
    // The cross product divides by this power of two; individual columns can differ in scale.
    pub product_denominator_exponent: i32,
}
pub(super) fn placed_support_columns(
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactSupportColumns, ResidualError> {
    let first = placed_basis_line(frame, placement, [1., 0.], cancelled)?;
    let second = placed_basis_line(frame, placement, [0., 1.], cancelled)?;
    Ok(ExactSupportColumns {
        columns: [derivative(&first), derivative(&second)],
        product_denominator_exponent: denominator_exponent(&first) + denominator_exponent(&second),
    })
}
fn placed_basis_line(
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    direction: [f64; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<coefficients::HomogeneousCurve, ResidualError> {
    let line = MetricCurve::Line {
        origin: [0.; 2],
        direction,
    };
    let embedded = coefficients::embedded(line, frame, "prism support", cancelled)?;
    coefficients::place(embedded, placement)
}
fn derivative(line: &coefficients::HomogeneousCurve) -> [BigInt; 3] {
    // These are affine power coefficients, not samples or endpoint subtractions.
    std::array::from_fn(|axis| line[axis][1].clone())
}
fn denominator_exponent(line: &coefficients::HomogeneousCurve) -> i32 {
    // Unit-weight affine lines and binary64 embeddings retain a positive power-of-two denominator.
    line[3][0]
        .trailing_zeros()
        .expect("unit-weight affine denominator is nonzero") as i32
}
