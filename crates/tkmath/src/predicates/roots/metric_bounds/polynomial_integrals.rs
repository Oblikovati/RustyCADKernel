// SPDX-License-Identifier: GPL-2.0-only
//! Version 1 conditional measures for supplied full-range polynomial boundaries.
//! Exact metric coefficients and arithmetic are shared with residual certification;
//! no Gauss error estimate is reclassified and no geometric validity is inferred.
mod arithmetic;
mod error;
mod plane;
mod types;
use super::super::{certificate_arithmetic::ExactRatio, error::poll};
use super::{MetricEmbedding, affine_prism_jacobian};
pub use error::{PolynomialIntegralError, PolynomialIntegralFailure};
pub use plane::*;
pub use types::*;

/// Certify the ordered signed Green integral on every supplied full native `[0,1]` range.
/// Traversal is explicit; no closure, coverage, orientation validity or injectivity is checked.
/// An open boundary's complete integral is not accepted region area. Rational weights
/// and excess admission/error limits return typed incomplete failures without partial results.
/// ```
/// use rustycad_tkmath::predicates::roots::metric_bounds::*;
/// let poles = [[0.,0.], [4.,0.], [0.,4.], [0.,0.]];
/// let curve = MetricCurve::Bezier(MetricBezier { poles: &poles, weights: None });
/// let occurrences = [OrientedMetricCurve { curve, reversed: false }];
/// let report = certify_polynomial_boundary_integral(&occurrences, Default::default(), || false)?;
/// assert!(report.enclosure()[0] <= 2.4 && report.enclosure()[1] >= 2.4);
/// # Ok::<(), PolynomialIntegralError>(())
/// ```
pub fn certify_polynomial_boundary_integral(
    curves: &[OrientedMetricCurve<'_>],
    options: PolynomialIntegralOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedPolynomialMeasure, PolynomialIntegralError> {
    let request = IntegralRequest {
        options,
        operation: PolynomialIntegralOperation::SignedBoundaryIntegral,
    };
    let exact = request.integrate(curves, &mut cancelled)?;
    request.publish(exact, &mut cancelled)
}

/// Multiply the unrounded signed Green integral by the exact affine sweep determinant.
/// Interpreting this scalar as signed volume requires separately proven oriented boundary
/// coverage and injective support/sweep. Zero from a singular map is a scalar, not a valid body.
/// There is no intermediate rounding of area or determinant and no inferred healing/history.
/// Example: `certify_polynomial_sweep_integral(&occurrences, frame, placement, vector, options, || false)?`.
pub fn certify_polynomial_sweep_integral(
    curves: &[OrientedMetricCurve<'_>],
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    extrusion: [f64; 3],
    options: PolynomialIntegralOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedPolynomialMeasure, PolynomialIntegralError> {
    let request = IntegralRequest {
        options,
        operation: PolynomialIntegralOperation::SignedSweepIntegral,
    };
    let exact = request.integrate(curves, &mut cancelled)?;
    let jacobian = affine_prism_jacobian(frame, placement, extrusion, &mut cancelled)
        .map_err(|cause| request.failure(None, cause.into()))?;
    request.publish(
        exact.multiply(&ExactRatio::from_dyadic(&jacobian)),
        &mut cancelled,
    )
}

#[derive(Clone, Copy)]
struct IntegralRequest {
    options: PolynomialIntegralOptions,
    operation: PolynomialIntegralOperation,
}
impl IntegralRequest {
    fn failure(
        self,
        curve_index: Option<usize>,
        cause: PolynomialIntegralFailure,
    ) -> PolynomialIntegralError {
        PolynomialIntegralError {
            operation: self.operation,
            requested_options: self.options,
            curve_index,
            cause,
        }
    }
    fn integrate(
        self,
        curves: &[OrientedMetricCurve<'_>],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<ExactRatio, PolynomialIntegralError> {
        poll(cancelled).map_err(|cause| self.failure(None, cause.into()))?;
        arithmetic::admit_boundary(curves, self.options)
            .map_err(|cause| self.failure(None, cause))?;
        let mut sum = ExactRatio::from_finite(0.);
        for (index, occurrence) in curves.iter().enumerate() {
            let contribution = self.contribution(index, *occurrence, cancelled)?;
            sum = sum.add(&contribution);
        }
        Ok(sum)
    }
    fn contribution(
        self,
        index: usize,
        occurrence: OrientedMetricCurve<'_>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<ExactRatio, PolynomialIntegralError> {
        let integral = arithmetic::curve_integral(occurrence.curve, self.options, cancelled)
            .map_err(|cause| self.failure(Some(index), cause))?;
        Ok(if occurrence.reversed {
            integral.negated()
        } else {
            integral
        })
    }
    fn publish(
        self,
        exact: ExactRatio,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<CertifiedPolynomialMeasure, PolynomialIntegralError> {
        poll(cancelled).map_err(|cause| self.failure(None, cause.into()))?;
        let report = arithmetic::rounded_measure(exact, self.options, self.operation)
            .map_err(|cause| self.failure(None, cause))?;
        poll(cancelled).map_err(|cause| self.failure(None, cause.into()))?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests;
