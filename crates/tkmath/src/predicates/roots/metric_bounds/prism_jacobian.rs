// SPDX-License-Identifier: GPL-2.0-only
//! Exact affine sweep derivative; its nonzero determinant proves only this supplied map injective.
use super::affine_support::placed_support_columns;
use super::{DyadicBound, MetricEmbedding, ResidualError, coefficients};
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
    let support = placed_support_columns(frame, placement, &mut cancelled)?;
    let vector = exact_extrusion(extrusion)?;
    super::super::error::poll(&mut cancelled)?;
    let numerator = exact_triple_product(&support.columns[0], &support.columns[1], &vector);
    let exponent = -1074 - support.product_denominator_exponent;
    Ok(DyadicBound::new(numerator, exponent))
}
fn exact_extrusion(extrusion: [f64; 3]) -> Result<[BigInt; 3], ResidualError> {
    let scalar = |axis| coefficients::scalar(extrusion[axis], "prism extrusion", axis);
    Ok([scalar(0)?, scalar(1)?, scalar(2)?])
}
