// SPDX-License-Identifier: GPL-2.0-only
//! Conditional certified polynomial extrusion-side integral, behavior version 1 (ADR0024).
mod arithmetic;
mod bounds;
mod error;
mod execution;
mod restriction;
mod types;
use super::super::certificate_arithmetic::ExactRatio;
use super::{MetricCurve, MetricEmbedding};
pub use arithmetic::admit_policy as validate_polynomial_side_options;
pub use error::PolynomialSideFailure;
pub use types::*;
type Vector = [ExactRatio; 3];

/// Bound `integral_0^1 |C'(t) cross extrusion| dt` for the supplied affine polynomial.
/// `C` is the curve after embedding and placement; their coefficients remain exact.
/// The result counts parameter multiplicity and certifies no face, body or regularity.
/// Explicit rational weights are refused. Finite limits are never silently widened.
/// ```
/// use rustycad_tkmath::predicates::roots::metric_bounds::*;
/// let frame = MetricEmbedding { linear: [[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]], origin: [0.;3], scale: 1. };
/// let line = MetricCurve::Line { origin: [0.;3], direction: [3.,0.,0.] };
/// let report = certify_polynomial_side_integral(line, frame, frame, [0.,0.,2.], Default::default(), || false)?;
/// assert_eq!(report.enclosure(), [6.,6.]);
/// # Ok::<(), PolynomialSideError>(())
/// ```
pub fn certify_polynomial_side_integral<const DIM: usize>(
    curve: MetricCurve<'_, DIM>,
    embedding: MetricEmbedding<DIM>,
    placement: MetricEmbedding<3>,
    extrusion: [f64; 3],
    options: PolynomialSideOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedPolynomialSideIntegral, PolynomialSideError> {
    let mut execution = execution::Execution::new(options);
    let result = (|| {
        checkpoint(&mut cancelled)?;
        arithmetic::admit_policy(options)?;
        execution.admitted = true;
        arithmetic::admit_curve(curve, options)?;
        let controls =
            arithmetic::cross_controls(curve, embedding, placement, extrusion, &mut cancelled)?;
        execution.integrate(controls, &mut cancelled)
    })();
    result.map_err(|cause| execution.failure(cause))
}
fn checkpoint(cancelled: &mut impl FnMut() -> bool) -> Result<(), PolynomialSideFailure> {
    if cancelled() {
        return Err(PolynomialSideFailure::Cancelled);
    }
    Ok(())
}

#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod invariant_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod policy_tests;
