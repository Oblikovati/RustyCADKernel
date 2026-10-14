// SPDX-License-Identifier: GPL-2.0-only
//! One constant-map residual query for captured world endpoints; no separate evaluator.
//! Callers checkpoint before/after this bounded constant-domain arithmetic; no inner callback
//! can mutate the captured scalar operands during the query.
use super::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::*};

/// Certify captured endpoint operands at an explicit budget; example: `certify_captured_endpoint(curve, 0., point, 0.)`.
/// The returned Within bound is certified and may be non-tight; no stored tolerance is consulted.
pub(crate) fn certify_captured_endpoint(
    curve: &CapturedSpaceRepresentation,
    parameter: f64,
    point: [f64; 3],
    spatial_limit: f64,
) -> TransportedResidualReport {
    certify_captured_placed_endpoint(curve, parameter, point, identity_placement(), spatial_limit)
}

/// Preserve local point transport as exact operands; example: `certify_captured_placed_endpoint(curve, 0., local, placement, 0.)`.
/// The caller establishes placement usability; this query preserves the original metric decisions.
pub(crate) fn certify_captured_placed_endpoint(
    curve: &CapturedSpaceRepresentation,
    parameter: f64,
    local_point: [f64; 3],
    placement: MetricEmbedding<3>,
    spatial_limit: f64,
) -> TransportedResidualReport {
    let query = transported_endpoint_query(curve, parameter, local_point, placement, spatial_limit);
    certify_transported_residual(query, || false)
}

/// Preserve the fixed constant-domain root policy; example: `captured_endpoint_root_options()`.
/// Constant parameter maps eliminate the search domain, so width one covers its artificial interval.
pub(crate) fn captured_endpoint_root_options() -> RootIsolationOptions {
    RootIsolationOptions::new(1.)
}

/// Reuse a source query's exact maps and bounds with raw point transport; example: `with_captured_point_comparator(query, local, placement, 0.)`.
/// Only the point comparator and its explicit spatial budget change; source operands remain intact.
pub(crate) fn with_captured_point_comparator(
    mut query: TransportedResidualQuery<'_>,
    local_point: [f64; 3],
    placement: MetricEmbedding<3>,
    spatial_limit: f64,
) -> TransportedResidualQuery<'_> {
    query.residual.pcurve = constant_origin();
    query.residual.plane_embedding = endpoint_embedding(local_point);
    query.residual.plane_placement = placement;
    query.residual.spatial_limit = spatial_limit;
    query
}

fn transported_endpoint_query(
    curve: &CapturedSpaceRepresentation,
    parameter: f64,
    point: [f64; 3],
    placement: MetricEmbedding<3>,
    spatial_limit: f64,
) -> TransportedResidualQuery<'_> {
    TransportedResidualQuery {
        residual: endpoint_query(curve, point, placement, spatial_limit),
        maps: [
            MetricParameterMap::Endpoints([parameter, parameter]),
            MetricParameterMap::Identity,
        ],
    }
}

fn endpoint_query(
    curve: &CapturedSpaceRepresentation,
    point: [f64; 3],
    placement: MetricEmbedding<3>,
    spatial_limit: f64,
) -> PlanarResidualQuery<'_> {
    PlanarResidualQuery {
        curve: curve.curve.borrow(),
        pcurve: constant_origin(),
        curve_embedding: curve.placement,
        plane_embedding: endpoint_embedding(point),
        plane_placement: placement,
        interval: [0., 1.],
        spatial_limit,
        options: captured_endpoint_root_options(),
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

fn endpoint_embedding(point: [f64; 3]) -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[0.; 2]; 3],
        origin: point,
        scale: 1.,
    }
}
