// SPDX-License-Identifier: GPL-2.0-only
//! Exact supplied-representation geometry; no mapped tolerance algorithm is replaced.
mod arithmetic;
mod curve_identity;
pub use curve_identity::{CurveIdentityDecision, MetricCurveChain, compare_curve_chains};
mod affine_support;
#[cfg(test)]
mod curve_identity_tests;
mod mapped;
mod parameter;
mod prism_jacobian;
pub use prism_jacobian::affine_prism_jacobian;
#[cfg(test)]
mod prism_jacobian_tests;
pub use mapped::{
    TransportedResidualQuery, TransportedResidualReport, certify_transported_residual,
    certify_transported_residual_on_bounds,
};
pub use parameter::MetricParameterMap;
mod coefficients;
mod inequality;
mod proof;
mod types;
use super::{DyadicBound, error::poll};
use num_bigint::BigInt;
pub use types::*;

/// Certify the full declared same-parameter residual without mutating representations.
/// Example: `certify_planar_residual(query, || false)?` yields a proven squared bound.
/// The bound is not a sampled estimate or necessarily the tight maximum.
pub fn certify_planar_residual(
    query: PlanarResidualQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<ResidualDecision, ResidualError> {
    certify_query(query, None, &mut cancelled)
}
/// Certify an exact subinterval inside the original declared representation domain.
/// Example: `certify_planar_residual_on_bounds(query, root_bounds, || false)?`.
/// Use original unsplit source parameters; this proves metric conformance, not contact identity.
pub fn certify_planar_residual_on_bounds(
    query: PlanarResidualQuery<'_>,
    bounds: [DyadicBound; 2],
    mut cancelled: impl FnMut() -> bool,
) -> Result<ResidualDecision, ResidualError> {
    certify_query(query, Some(bounds), &mut cancelled)
}
fn certify_query(
    query: PlanarResidualQuery<'_>,
    exact_bounds: Option<[DyadicBound; 2]>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ResidualDecision, ResidualError> {
    poll(cancelled)?;
    validate_query(&query)?;
    let (declared, width) = super::checked_policy_interval(query.interval, query.options)?;
    let bounds = exact_bounds.unwrap_or_else(|| declared.clone());
    admit_subinterval(&bounds, &declared, query.interval)?;
    let (coefficients, zero, square) = inequality::threshold(&query, cancelled)?;
    let upper = DyadicBound::new(if zero { BigInt::from(0) } else { square }, -2148);
    let domain = proof::MetricProofDomain {
        bounds,
        width,
        options: query.options,
        upper,
    };
    proof::decide(coefficients, domain, cancelled)
}
fn admit_subinterval(
    bounds: &[DyadicBound; 2],
    declared: &[DyadicBound; 2],
    original: [f64; 2],
) -> Result<(), ResidualError> {
    if bounds[0] > bounds[1] || bounds[0] < declared[0] || bounds[1] > declared[1] {
        return Err(ResidualError::ExactDomain {
            supplied: bounds.clone(),
            declared: original,
        });
    }
    Ok(())
}
fn validate_query(query: &PlanarResidualQuery<'_>) -> Result<(), ResidualError> {
    let [first, last] = query.interval;
    let bezier = matches!(query.curve, MetricCurve::Bezier(_))
        || matches!(query.pcurve, MetricCurve::Bezier(_));
    if !first.is_finite()
        || !last.is_finite()
        || first > last
        || (bezier && (first < 0. || last > 1.))
    {
        return Err(ResidualError::Domain(query.interval));
    }
    if !query.spatial_limit.is_finite() || query.spatial_limit < 0. {
        return Err(ResidualError::Value {
            role: "spatial_limit",
            index: 0,
            value: query.spatial_limit,
        });
    }
    Ok(())
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod parameter_tests;

#[cfg(test)]
mod transported_contact_bounds_tests;

#[cfg(test)]
mod integral_baseline_tests;

mod polynomial_integrals;
pub use polynomial_integrals::{
    CertifiedPolynomialMeasure, CertifiedPolynomialPlaneMeasure, OrientedMetricCurve,
    PolynomialIntegralError, PolynomialIntegralEvidence, PolynomialIntegralFailure,
    PolynomialIntegralOperation, PolynomialIntegralOptions, PolynomialPlaneError,
    PolynomialPlaneEvidence, PolynomialPlaneFailure, certify_polynomial_boundary_integral,
    certify_polynomial_plane_measure, certify_polynomial_sweep_integral,
};

mod rational_integrals;
pub use rational_integrals::{
    CertifiedRationalMeasure, RATIONAL_TAYLOR_ORDER, RationalIntegralError,
    RationalIntegralEvidence, RationalIntegralFailure, RationalIntegralOptions,
    certify_rational_boundary_integral, certify_rational_face_area,
    certify_rational_sweep_integral,
};

mod polynomial_side;
pub use polynomial_side::{
    CertifiedPolynomialSideIntegral, PolynomialSideError, PolynomialSideEvidence,
    PolynomialSideFailure, PolynomialSideOptions, PolynomialSideWork,
    certify_polynomial_side_integral, validate_polynomial_side_options,
};

#[cfg(test)]
mod regularity_baseline_tests;

mod bezier_regularity;
pub use bezier_regularity::{
    BezierRegularityCause, BezierRegularityCompletion, BezierRegularityDecision,
    BezierRegularityFailure, BezierRegularityOptions, BezierRegularityQuery,
    BezierRegularityReport, BezierRegularitySource, BezierRegularityStage,
    certify_bezier_regularity,
};

mod enclosure_sum;
pub use enclosure_sum::{
    CertifiedEnclosureSum, EnclosureSumCause, EnclosureSumError, EnclosureSumEvidence,
    EnclosureSumOperation, EnclosureSumOptions, sum_nonnegative_enclosures,
};
