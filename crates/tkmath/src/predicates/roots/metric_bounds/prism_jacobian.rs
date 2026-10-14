// SPDX-License-Identifier: GPL-2.0-only
//! Exact affine sweep derivative; its nonzero determinant proves only this supplied map injective.
use super::{DyadicBound, MetricCurve, MetricEmbedding, ResidualError, coefficients};
use crate::predicates::triple_product::exact_triple_product;
use num_bigint::BigInt;

/// Compute det(dF/du, dF/dv, dF/dw) for F(u,v,w)=placement(frame(u,v))+w*extrusion.
/// A nonzero exact value proves this affine map is injective; zero is singular.
/// It does not certify a bounded region, actual swept surface charts or a valid solid.
/// All coefficients must be finite, including origins. Typed input errors are retained.
/// Cancellation is polled at each basis line and before publication (three fixed steps).
/// Example: `affine_prism_jacobian(frame, placement, [0.,0.,2.], || false)?`.
pub fn affine_prism_jacobian(
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    extrusion: [f64; 3],
    mut cancelled: impl FnMut() -> bool,
) -> Result<DyadicBound, ResidualError> {
    let first = placed_basis_line(frame, placement, [1., 0.], &mut cancelled)?;
    let second = placed_basis_line(frame, placement, [0., 1.], &mut cancelled)?;
    let vector = exact_extrusion(extrusion)?;
    super::super::error::poll(&mut cancelled)?;
    let numerator = exact_triple_product(&derivative(&first), &derivative(&second), &vector);
    let exponent = -1074 - denominator_exponent(&first) - denominator_exponent(&second);
    Ok(DyadicBound::new(numerator, exponent))
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
fn exact_extrusion(extrusion: [f64; 3]) -> Result<[BigInt; 3], ResidualError> {
    let scalar = |axis| coefficients::scalar(extrusion[axis], "prism extrusion", axis);
    Ok([scalar(0)?, scalar(1)?, scalar(2)?])
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
