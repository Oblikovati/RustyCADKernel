// SPDX-License-Identifier: GPL-2.0-only
//! Reuse the original captured source query; never infer an endpoint from candidate order.
use super::scalar::*;
use crate::sketch_region_source_conformance::with_captured_point_comparator;
use rustycad_tkmath::predicates::roots::{DyadicBound, metric_bounds::*};

pub(super) enum ExactClosureFailure {
    SourceBounds { actual: [DyadicBound; 2] },
    Endpoint(Box<ExactEndpointFailure>),
}

impl From<ExactEndpointFailure> for ExactClosureFailure {
    fn from(cause: ExactEndpointFailure) -> Self {
        Self::Endpoint(Box::new(cause))
    }
}

/// Check raw closure transport on the original singleton query; example: `certify_exact_source_point(query, bounds, local, placement)`.
/// Maps, interval and root settings remain those of the joined source query; the separate exact budget is zero.
pub(super) fn certify_exact_source_point(
    query: TransportedResidualQuery<'_>,
    bounds: [DyadicBound; 2],
    local_point: [f64; 3],
    placement: MetricEmbedding<3>,
) -> Result<ExactEndpointCertificate, ExactClosureFailure> {
    require_source_singleton(&bounds)?;
    let source = endpoint_frame(
        query.residual.curve_embedding,
        EndpointFrameRole::CurveRepresentation,
    )?;
    let vertex = endpoint_frame(placement, EndpointFrameRole::VertexOccurrence)?;
    let roots = query.residual.options;
    let query = with_captured_point_comparator(query, local_point, placement, 0.);
    let report = certify_transported_residual_on_bounds(query, bounds, || false);
    complete_zero_report(source, vertex, roots, report).map_err(ExactClosureFailure::from)
}

fn require_source_singleton(bounds: &[DyadicBound; 2]) -> Result<(), ExactClosureFailure> {
    let zero = DyadicBound::from_f64(0.).expect("finite exact zero");
    if bounds.iter().any(|bound| *bound != zero) {
        return Err(ExactClosureFailure::SourceBounds {
            actual: bounds.clone(),
        });
    }
    Ok(())
}
