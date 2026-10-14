// SPDX-License-Identifier: GPL-2.0-only
//! Conditional world-plane measure, retaining exact native integral and Gram determinant.
mod arithmetic;
mod error;
mod types;
use super::super::affine_support::placed_support_columns;
use super::*;
pub use error::{PolynomialPlaneError, PolynomialPlaneFailure};
pub use types::{CertifiedPolynomialPlaneMeasure, PolynomialPlaneEvidence};

/// Bound `abs(I) * norm(a cross b)` for the exact native Green integral and placed columns.
/// Units are world coordinates squared; floating frame axes are never presumed exactly unit.
/// This conditional scalar requires separate boundary coverage and validity proofs before
/// it denotes region area. Singular support gives scalar zero, not a valid planar region.
/// Policy limits are unchanged; the finite absolute error cap is in world area units.
/// Example: `certify_polynomial_plane_measure(&curves, frame, placement, options, || false)?`.
pub fn certify_polynomial_plane_measure(
    curves: &[OrientedMetricCurve<'_>],
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    options: PolynomialIntegralOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedPolynomialPlaneMeasure, PolynomialPlaneError> {
    plane_measure(curves, frame, placement, options, &mut cancelled).map_err(|cause| {
        PolynomialPlaneError {
            requested_options: options,
            cause,
        }
    })
}
fn plane_measure(
    curves: &[OrientedMetricCurve<'_>],
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    options: PolynomialIntegralOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<CertifiedPolynomialPlaneMeasure, PolynomialPlaneFailure> {
    let square = integrated_square(curves, frame, placement, options, cancelled)?;
    poll(cancelled)?;
    let report = arithmetic::rounded_plane_measure(&square, options)?;
    poll(cancelled)?;
    Ok(report)
}
fn integrated_square(
    curves: &[OrientedMetricCurve<'_>],
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    options: PolynomialIntegralOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactRatio, PolynomialPlaneFailure> {
    let request = IntegralRequest {
        options,
        operation: PolynomialIntegralOperation::SignedBoundaryIntegral,
    };
    let integral = request.integrate(curves, cancelled)?;
    let columns = placed_support_columns(frame, placement, cancelled)?;
    Ok(integral
        .multiply(&integral)
        .multiply(&arithmetic::gram_determinant(&columns)))
}
