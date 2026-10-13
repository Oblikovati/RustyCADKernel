// SPDX-License-Identifier: GPL-2.0-only
use super::{parameter::admit_map, *};
/// New proof query; legacy same-parameter calls keep their original validation/order.
/// Example: native line t=[a,b] uses Endpoints([a,b]), captured endpoints use Identity.
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
    let decision = execute(query, &mut cancelled, &mut || bisections += 1);
    TransportedResidualReport {
        decision,
        bisections,
    }
}
fn execute(
    query: TransportedResidualQuery<'_>,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<ResidualDecision, ResidualError> {
    poll(cancelled)?;
    let domain = prepare(&query)?;
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
) -> Result<proof::MetricProofDomain, ResidualError> {
    let q = &query.residual;
    let (bounds, width) = super::super::checked_policy_interval(q.interval, q.options)?;
    admit_map(q.curve, query.maps[0], &bounds)?;
    admit_map(q.pcurve, query.maps[1], &bounds)?;
    if !q.spatial_limit.is_finite() || q.spatial_limit < 0. {
        return Err(ResidualError::Value {
            role: "spatial_limit",
            index: 0,
            value: q.spatial_limit,
        });
    }
    Ok(proof::MetricProofDomain {
        bounds,
        width,
        options: q.options,
        upper: DyadicBound::new(BigInt::from(0), 0),
    })
}
