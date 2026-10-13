// SPDX-License-Identifier: GPL-2.0-only
//! Nonmutating complete per-representation metric decisions do not certify every body context.
use super::outcome::{SourceRelation, attributed_occurrence};
use super::tolerance_capture::capture_pair;
use super::tolerance_types::{PlanarMetricCause, PlanarMetricInspection};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    metric_bounds::{ResidualDecision, certify_planar_residual},
};
/// Inspect an actual stored edge/pcurve pair without changing geometry or tolerance.
/// Example: `inspect_planar_edge_residual(&edge,&face,&sources,0.001,policy,||false)`.
pub(crate) fn inspect_planar_edge_residual(
    edge: &Edge,
    face: &Face,
    sources: &[SourceRelation],
    limit: f64,
    policy: RootIsolationOptions,
    cancelled: impl FnMut() -> bool,
) -> PlanarMetricInspection {
    let (decision, coefficient_snapshot) = inspect_edge(edge, face, limit, policy, cancelled);
    PlanarMetricInspection {
        edge: attributed_occurrence(edge, sources),
        face: attributed_occurrence(face, sources),
        requested_spatial_limit: limit,
        effective_spatial_limit: limit,
        root_policy: policy,
        decision,
        failed_subdivision_work: None,
        coefficient_snapshot,
    }
}
fn inspect_edge(
    edge: &Edge,
    face: &Face,
    limit: f64,
    policy: RootIsolationOptions,
    cancelled: impl FnMut() -> bool,
) -> (
    Result<ResidualDecision, PlanarMetricCause>,
    Option<super::tolerance_capture::CapturedPlanarPair>,
) {
    let captured = match capture_pair(edge, face) {
        Ok(captured) => captured,
        Err(cause) => return (Err(cause), None),
    };
    let decision = certify_planar_residual(captured.query(limit, policy), cancelled)
        .map_err(PlanarMetricCause::Exact);
    (decision, Some(captured))
}
