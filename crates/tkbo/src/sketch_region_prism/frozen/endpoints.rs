// SPDX-License-Identifier: GPL-2.0-only
//! Existing exact residual machinery checks frozen endpoint values; no new evaluator is introduced.
//! Exact zero equality admission is independent of requested body tolerances/error limits.
//! Constant parameter maps eliminate the search domain; fixed width 1 bounds that constant query.
//! Evidence/Commit checkpoints surround this bounded coefficient arithmetic; no interior callback
//! mutates its frozen scalar operands. This is not a claim of general body tolerance validation.
use super::{bindings::missing, graph::FrozenGraphIncidence, types::*};
use crate::sketch_region_source_conformance::{
    CapturedSpaceRepresentation, certify_captured_endpoint,
};
use rustycad_tkmath::predicates::roots::{DyadicBound, metric_bounds::*};

/// Certified bound for exactly captured values, not an ordered whole-body validation pass.
#[derive(Clone, Debug)]
pub(crate) struct FrozenEndpointProof {
    pub(crate) edge: FrozenPrismKey,
    pub(crate) vertex: FrozenPrismKey,
    pub(crate) parameter: f64,
    pub(crate) requested_spatial: f64,
    pub(crate) proven_squared_upper: DyadicBound,
}
pub(super) fn certify_endpoints(
    nodes: &[FrozenPrismNode],
    graph: &FrozenGraphIncidence,
) -> CaptureResult<Vec<FrozenEndpointProof>> {
    let mut result = Vec::new();
    for (edge, first, last) in &graph.endpoints {
        let curve = node(nodes, *edge)
            .and_then(|n| n.curve.as_ref().ok_or(missing("captured endpoint curve")))
            .map_err(|c| c.at_endpoint(*edge, *first))?;
        for (vertex, parameter) in [(*first, curve.interval[0]), (*last, curve.interval[1])] {
            result.push(endpoint_proof(nodes, *edge, vertex, parameter, curve)?);
        }
    }
    Ok(result)
}
fn endpoint_proof(
    nodes: &[FrozenPrismNode],
    edge: FrozenPrismKey,
    vertex: FrozenPrismKey,
    parameter: f64,
    curve: &CapturedSpaceRepresentation,
) -> CaptureResult<FrozenEndpointProof> {
    let point = node(nodes, vertex)
        .map_err(|c| c.at_endpoint(edge, vertex))?
        .point
        .ok_or(missing("captured endpoint vertex"))
        .map_err(|c| c.at_endpoint(edge, vertex))?;
    Ok(FrozenEndpointProof {
        edge,
        vertex,
        parameter,
        requested_spatial: 0.,
        proven_squared_upper: certify_endpoint(curve, parameter, point)
            .map_err(|c| c.at_endpoint(edge, vertex))?,
    })
}

fn node(nodes: &[FrozenPrismNode], key: FrozenPrismKey) -> CaptureResult<&FrozenPrismNode> {
    nodes
        .iter()
        .find(|node| node.key == key)
        .ok_or(missing("stable endpoint incidence"))
}
fn certify_endpoint(
    curve: &CapturedSpaceRepresentation,
    parameter: f64,
    point: [f64; 3],
) -> CaptureResult<DyadicBound> {
    let report = certify_captured_endpoint(curve, parameter, point, 0.);
    match report.decision.map_err(PrismFreezeCause::Endpoint)? {
        ResidualDecision::Within {
            proven_squared_upper,
        } => Ok(proven_squared_upper),
        decision => Err(PrismFreezeCause::EndpointMismatch {
            parameter,
            point,
            decision,
        }),
    }
}
