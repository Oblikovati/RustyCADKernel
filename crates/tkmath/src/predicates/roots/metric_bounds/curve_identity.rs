// SPDX-License-Identifier: GPL-2.0-only
//! Full coefficient identities bind actual sweep charts without sampling or rounded placement.
use super::{MetricCurve, MetricEmbedding, ResidualError, arithmetic, coefficients};
use num_bigint::Sign;

/// An analytic curve followed by exact affine embeddings in application order.
/// Example: append a translation embedding to compare an actual cap with its source.
#[derive(Clone, Copy)]
pub struct MetricCurveChain<'a, const DIM: usize> {
    /// Stored line or positive-weight Bézier coefficients.
    pub curve: MetricCurve<'a, DIM>,
    /// First embedding from curve coordinates into three-dimensional space.
    pub embedding: MetricEmbedding<DIM>,
    /// Subsequent placements; their coefficients are never pre-multiplied in binary64.
    pub placements: &'a [MetricEmbedding<3>],
}
/// Exact formal same-parameter identity wherever both rational representations are defined.
/// This does not prove matching trims, topology, surface regularity or bounded-body validity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurveIdentityDecision {
    /// Every cross-multiplied numerator coefficient vanishes exactly.
    Identical,
    /// First nonzero coefficient in coordinate-major, increasing-power order.
    Different {
        /// Zero-based world coordinate.
        coordinate: usize,
        /// Zero-based power coefficient of the homogeneous cross difference.
        coefficient: usize,
    },
}
/// Compare complete rational functions after the supplied placement chains.
/// Example: `compare_curve_chains(source, actual, || false)?` retains a typed mismatch.
/// Finite coefficients and positive weights are required; cancellation returns no identity.
pub fn compare_curve_chains<const FIRST: usize, const SECOND: usize>(
    first: MetricCurveChain<'_, FIRST>,
    second: MetricCurveChain<'_, SECOND>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CurveIdentityDecision, ResidualError> {
    let first = chain_coefficients(first, &mut cancelled)?;
    let second = chain_coefficients(second, &mut cancelled)?;
    for coordinate in 0..3 {
        let difference = cross_difference(&first, &second, coordinate, &mut cancelled)?;
        if let Some(coefficient) = difference.iter().position(|n| n.sign() != Sign::NoSign) {
            return Ok(CurveIdentityDecision::Different {
                coordinate,
                coefficient,
            });
        }
    }
    Ok(CurveIdentityDecision::Identical)
}
fn chain_coefficients<const DIM: usize>(
    chain: MetricCurveChain<'_, DIM>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<coefficients::HomogeneousCurve, ResidualError> {
    let mut curve = coefficients::embedded(chain.curve, chain.embedding, "chart curve", cancelled)?;
    for placement in chain.placements {
        super::super::error::poll(cancelled)?;
        curve = coefficients::place(curve, *placement)?;
    }
    Ok(curve)
}
fn cross_difference(
    first: &coefficients::HomogeneousCurve,
    second: &coefficients::HomogeneousCurve,
    coordinate: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<num_bigint::BigInt>, ResidualError> {
    let left = arithmetic::product(&first[coordinate], &second[3], cancelled)?;
    let right = arithmetic::product(&second[coordinate], &first[3], cancelled)?;
    Ok(arithmetic::difference(left, right))
}
