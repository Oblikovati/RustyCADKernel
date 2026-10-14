// SPDX-License-Identifier: GPL-2.0-only
use super::{parameter::admit_map, *};
/// New proof query; legacy same-parameter calls keep their original validation/order.
/// Example: native line `t=[a,b]` uses `Endpoints([a,b])`, captured endpoints use `Identity`.
pub struct TransportedResidualQuery<'a> {
    /// Captured coefficients, placements, requested spatial limit and root policy.
    pub residual: PlanarResidualQuery<'a>,
    /// Native parameter maps for the space curve and pcurve respectively.
    pub maps: [MetricParameterMap; 2],
}
/// Terminal evidence preserves actual work even for cancellation or exhausted search.
/// Example: inspect `report.bisections` even when `report.decision` is incomplete.
pub struct TransportedResidualReport {
    /// Complete inequality decision or original typed incomplete cause.
    pub decision: Result<ResidualDecision, ResidualError>,
    /// Actually completed splits, never inferred from an error or refunded.
    pub bisections: u64,
}
/// Certify representation conformance, not contact identity or region completion.
/// Example: `certify_transported_residual(query, || false).decision`.
pub fn certify_transported_residual(
    query: TransportedResidualQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> TransportedResidualReport {
    let mut bisections = 0;
    let decision = execute(query, None, &mut cancelled, &mut || bisections += 1);
    TransportedResidualReport {
        decision,
        bisections,
    }
}
/// Certify on an actual exact contact subinterval inside the original declared domain.
/// Example: `certify_transported_residual_on_bounds(query, exact_root_bounds, ||false)`.
pub fn certify_transported_residual_on_bounds(
    query: TransportedResidualQuery<'_>,
    bounds: [DyadicBound; 2],
    mut cancelled: impl FnMut() -> bool,
) -> TransportedResidualReport {
    let mut bisections = 0;
    let decision = execute(query, Some(bounds), &mut cancelled, &mut || bisections += 1);
    TransportedResidualReport {
        decision,
        bisections,
    }
}

fn execute(
    query: TransportedResidualQuery<'_>,
    exact_bounds: Option<[DyadicBound; 2]>,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<ResidualDecision, ResidualError> {
    poll(cancelled)?;
    let domain = prepare(&query, exact_bounds)?;
    let (polynomial, zero, square) =
        inequality::mapped_threshold(&query.residual, query.maps, cancelled)?;
    let upper = DyadicBound::new(if zero { BigInt::from(0) } else { square }, -2148);
    proof::decide_accounted(
        polynomial,
        proof::MetricProofDomain { upper, ..domain },
        cancelled,
        completed_split,
    )
}
fn prepare(
    query: &TransportedResidualQuery<'_>,
    exact_bounds: Option<[DyadicBound; 2]>,
) -> Result<proof::MetricProofDomain, ResidualError> {
    let q = &query.residual;
    let (bounds, width) = checked_contact_domain(q, exact_bounds)?;
    admit_map(q.curve, query.maps[0], &bounds)?;
    admit_map(q.pcurve, query.maps[1], &bounds)?;
    checked_spatial_limit(q.spatial_limit)?;
    Ok(proof::MetricProofDomain {
        bounds,
        width,
        options: q.options,
        upper: DyadicBound::new(BigInt::from(0), 0),
    })
}

fn checked_contact_domain(
    q: &PlanarResidualQuery<'_>,
    exact_bounds: Option<[DyadicBound; 2]>,
) -> Result<([DyadicBound; 2], DyadicBound), ResidualError> {
    let (declared, width) = super::super::checked_policy_interval(q.interval, q.options)?;
    let bounds = match exact_bounds {
        None => declared,
        Some(bounds) => {
            super::admit_subinterval(&bounds, &declared, q.interval)?;
            bounds
        }
    };
    Ok((bounds, width))
}
fn checked_spatial_limit(value: f64) -> Result<(), ResidualError> {
    if !value.is_finite() || value < 0. {
        return Err(ResidualError::Value {
            role: "spatial_limit",
            index: 0,
            value,
        });
    }
    Ok(())
}
