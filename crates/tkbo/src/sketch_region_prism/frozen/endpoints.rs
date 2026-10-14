// SPDX-License-Identifier: GPL-2.0-only
//! Existing exact residual machinery checks frozen endpoint values; no new evaluator is introduced.
//! Exact zero equality admission is independent of requested body tolerances/error limits.
//! Constant parameter maps eliminate the search domain; fixed width 1 bounds that constant query.
//! Evidence/Commit checkpoints surround this bounded coefficient arithmetic; no interior callback
//! mutates its frozen scalar operands. This is not a claim of general body tolerance validation.
use super::{bindings::missing, graph::FrozenGraphIncidence, types::*};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*};

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
) -> Result<Vec<FrozenEndpointProof>, PrismFreezeCause> {
    let mut result = Vec::new();
    for (edge, first, last) in &graph.endpoints {
        let curve = node(nodes, *edge)?
            .curve
            .as_ref()
            .ok_or(missing("captured endpoint curve"))?;
        for (vertex, parameter) in [(*first, curve.interval[0]), (*last, curve.interval[1])] {
            let point = node(nodes, vertex)?
                .point
                .ok_or(missing("captured endpoint vertex"))?;
            result.push(FrozenEndpointProof {
                edge: *edge,
                vertex,
                parameter,
                requested_spatial: 0.,
                proven_squared_upper: certify_endpoint(curve, parameter, point)?,
            });
        }
    }
    Ok(result)
}
fn node(
    nodes: &[FrozenPrismNode],
    key: FrozenPrismKey,
) -> Result<&FrozenPrismNode, PrismFreezeCause> {
    nodes
        .iter()
        .find(|node| node.key == key)
        .ok_or(missing("stable endpoint incidence"))
}
fn certify_endpoint(
    curve: &CapturedSpaceRepresentation,
    parameter: f64,
    point: [f64; 3],
) -> Result<DyadicBound, PrismFreezeCause> {
    let query = TransportedResidualQuery {
        residual: endpoint_query(curve, point),
        maps: [
            MetricParameterMap::Endpoints([parameter, parameter]),
            MetricParameterMap::Identity,
        ],
    };
    let report = certify_transported_residual(query, || false);
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
fn endpoint_query(curve: &CapturedSpaceRepresentation, point: [f64; 3]) -> PlanarResidualQuery<'_> {
    PlanarResidualQuery {
        curve: curve.curve.borrow(),
        pcurve: constant_origin(),
        curve_embedding: curve.placement,
        plane_embedding: MetricEmbedding {
            linear: [[0.; 2]; 3],
            origin: point,
            scale: 1.,
        },
        plane_placement: identity_placement(),
        interval: [0., 1.],
        spatial_limit: 0.,
        options: RootIsolationOptions::new(1.),
    }
}
fn constant_origin() -> MetricCurve<'static, 2> {
    MetricCurve::Line {
        origin: [0.; 2],
        direction: [0.; 2],
    }
}
fn identity_placement() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
