// SPDX-License-Identifier: GPL-2.0-only
//! Existing exact residual machinery checks frozen endpoint values; no new evaluator is introduced.
use super::{bindings::missing, graph::FrozenGraphIncidence, types::*};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::*};

pub(super) fn certify_endpoints(
    nodes: &[FrozenPrismNode],
    graph: &FrozenGraphIncidence,
) -> Result<(), PrismFreezeCause> {
    for (edge, first, last) in &graph.endpoints {
        let curve = node(nodes, *edge)?
            .curve
            .as_ref()
            .ok_or(missing("captured endpoint curve"))?;
        let first = node(nodes, *first)?
            .point
            .ok_or(missing("captured first vertex"))?;
        let last = node(nodes, *last)?
            .point
            .ok_or(missing("captured last vertex"))?;
        certify_endpoint(curve, curve.interval[0], first)?;
        certify_endpoint(curve, curve.interval[1], last)?;
    }
    Ok(())
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
) -> Result<(), PrismFreezeCause> {
    let query = TransportedResidualQuery {
        residual: endpoint_query(curve, point),
        maps: [
            MetricParameterMap::Endpoints([parameter, parameter]),
            MetricParameterMap::Identity,
        ],
    };
    let report = certify_transported_residual(query, || false);
    match report.decision.map_err(PrismFreezeCause::Endpoint)? {
        ResidualDecision::Within { .. } => Ok(()),
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
        pcurve: MetricCurve::Line {
            origin: [0.; 2],
            direction: [0.; 2],
        },
        curve_embedding: curve.placement,
        plane_embedding: MetricEmbedding {
            linear: [[0.; 2]; 3],
            origin: point,
            scale: 1.,
        },
        plane_placement: MetricEmbedding {
            linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            origin: [0.; 3],
            scale: 1.,
        },
        interval: [0., 1.],
        spatial_limit: 0.,
        options: RootIsolationOptions::new(1.),
    }
}
