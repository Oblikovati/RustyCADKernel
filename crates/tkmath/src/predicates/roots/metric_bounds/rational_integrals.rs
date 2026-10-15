// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0066 part A: certified Green and sweep integrals for positive-weight rational boundaries.
//! Polynomial and equal-weight curves stay exact; other curves are enclosed by exact
//! Taylor models with certified Bernstein remainders. No quadrature estimate is used.
mod curve;
mod error;
mod face;
mod integer_polynomial;
mod publish;
mod subinterval;
mod types;
use super::super::{certificate_arithmetic::ExactRatio, error::poll};
use super::affine_prism_jacobian;
use super::{MetricEmbedding, OrientedMetricCurve, PolynomialIntegralOperation};
pub use error::{RationalIntegralError, RationalIntegralFailure};
pub use face::certify_rational_face_area;
use publish::reversed_bounds;
pub use types::*;

/// Certify the ordered signed Green integral of rational or polynomial full `[0,1]` curves.
/// Traversal is explicit; no closure, coverage, orientation or injectivity is checked, so
/// the scalar is region area only under a separate boundary proof (as for polynomials).
/// ```
/// use rustycad_tkmath::predicates::roots::metric_bounds::*;
/// // Exact quarter of the unit circle: x = (1-t^2)/(1+t^2), y = 2t/(1+t^2).
/// let poles = [[1., 0.], [1., 1.], [0., 1.]];
/// let weights = [1., 1., 2.];
/// let curve = MetricCurve::Bezier(MetricBezier { poles: &poles, weights: Some(&weights) });
/// let occurrences = [OrientedMetricCurve { curve, reversed: false }];
/// let report = certify_rational_boundary_integral(&occurrences, Default::default(), || false)?;
/// let quarter = std::f64::consts::FRAC_PI_4; // sector area pi/4
/// assert!((report.representative() - quarter).abs() <= report.absolute_error_bound() + 1e-16);
/// # Ok::<(), RationalIntegralError>(())
/// ```
pub fn certify_rational_boundary_integral(
    curves: &[OrientedMetricCurve<'_>],
    options: RationalIntegralOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedRationalMeasure, RationalIntegralError> {
    let request = RationalRequest {
        options,
        operation: PolynomialIntegralOperation::SignedBoundaryIntegral,
    };
    request.admit(curves, &mut cancelled)?;
    let one = ExactRatio::from_finite(1.);
    let sum = request.integrate(curves, &one, &mut cancelled)?;
    request.publish(sum, &mut cancelled)
}

/// Multiply the certified rational Green enclosure by the exact affine sweep determinant.
/// The integration budget is divided by `abs(det)` so the published error cap still holds.
/// Volume interpretation needs the same separate coverage and injectivity proofs.
/// Example: `certify_rational_sweep_integral(&occurrences, frame, placement, vector, options, || false)?`.
pub fn certify_rational_sweep_integral(
    curves: &[OrientedMetricCurve<'_>],
    frame: MetricEmbedding<2>,
    placement: MetricEmbedding<3>,
    extrusion: [f64; 3],
    options: RationalIntegralOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedRationalMeasure, RationalIntegralError> {
    let request = RationalRequest {
        options,
        operation: PolynomialIntegralOperation::SignedSweepIntegral,
    };
    request.admit(curves, &mut cancelled)?;
    let jacobian = affine_prism_jacobian(frame, placement, extrusion, &mut cancelled)
        .map_err(|cause| request.failure(None, cause.into()))?;
    let factor = ExactRatio::from_dyadic(&jacobian);
    let sum = request.integrate(curves, &factor, &mut cancelled)?;
    request.publish(sum.scaled(&factor), &mut cancelled)
}

#[derive(Clone, Copy)]
struct RationalRequest {
    options: RationalIntegralOptions,
    operation: PolynomialIntegralOperation,
}
impl RationalRequest {
    fn failure(
        self,
        curve_index: Option<usize>,
        cause: RationalIntegralFailure,
    ) -> RationalIntegralError {
        RationalIntegralError {
            operation: self.operation,
            requested_options: self.options,
            curve_index,
            cause,
        }
    }
    fn admit(
        self,
        curves: &[OrientedMetricCurve<'_>],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), RationalIntegralError> {
        poll(cancelled).map_err(|cause| self.failure(None, cause.into()))?;
        publish::admit_boundary(curves, self.options).map_err(|cause| self.failure(None, cause))
    }
    /// Sum exact per-curve bounds; each curve's width share keeps `abs(factor) * width <= cap`.
    fn integrate(
        self,
        curves: &[OrientedMetricCurve<'_>],
        factor: &ExactRatio,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<publish::BoundsSum, RationalIntegralError> {
        let share = publish::width_share(self.options, curves.len(), factor);
        let mut sum = publish::BoundsSum::zero();
        for (index, occurrence) in curves.iter().enumerate() {
            let part = curve::curve_integral(occurrence.curve, self.options, &share, cancelled)
                .map_err(|cause| self.failure(Some(index), cause))?;
            let bounds = reversed_bounds(part.bounds, occurrence.reversed);
            sum.accumulate(bounds, part.subintervals);
        }
        Ok(sum)
    }
    fn publish(
        self,
        sum: publish::BoundsSum,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<CertifiedRationalMeasure, RationalIntegralError> {
        poll(cancelled).map_err(|cause| self.failure(None, cause.into()))?;
        let report = publish::rounded_measure(sum, self.options, self.operation)
            .map_err(|cause| self.failure(None, cause))?;
        poll(cancelled).map_err(|cause| self.failure(None, cause.into()))?;
        Ok(report)
    }
}

#[cfg(test)]
mod closed_form_tests;
#[cfg(test)]
mod tests;
