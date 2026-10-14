// SPDX-License-Identifier: GPL-2.0-only
//! Reuse one scalar proof per physical endpoint and retain all directed obligations.
use super::{census_types::*, inspection_types::*, policy::PrismToleranceRequest};
use crate::sketch_region_source_conformance::{
    captured_endpoint_root_options, certify_captured_endpoint,
};
use rustycad_tkmath::predicates::roots::DyadicBound;

pub(super) fn endpoint_evidence(
    census: &PrismToleranceCensus,
    request: PrismToleranceRequest,
    progress: &mut impl ToleranceInspectionProgress,
) -> InspectionCheck<Vec<ToleranceEndpointEvidence>> {
    let mut result = Vec::new();
    for endpoint in &census.endpoints {
        poll(progress, ToleranceInspectionPhase::Endpoints)?;
        result.push(certify_operand(endpoint, request.world_spatial_budget()));
    }
    Ok(result)
}
fn certify_operand(endpoint: &ToleranceEndpointCensus, budget: f64) -> ToleranceEndpointEvidence {
    let report =
        certify_captured_endpoint(&endpoint.space, endpoint.parameter, endpoint.point, budget);
    ToleranceEndpointEvidence {
        edge: endpoint.edge.clone(),
        vertex: endpoint.vertex.clone(),
        parameter: endpoint.parameter,
        contexts: endpoint.contexts.clone(),
        requested_spatial: budget,
        effective_spatial: budget,
        effective_roots: captured_endpoint_root_options(),
        report,
    }
}
pub(super) fn trim_evidence(census: &PrismToleranceCensus) -> Vec<ToleranceTrimEvidence> {
    // Fresh shared binding proves zero source construction residual plus actual chart/trim identities.
    // The complete directed trim census composes those identities; it does not sample a surface.
    census
        .trims
        .iter()
        .map(|trim| ToleranceTrimEvidence {
            context: trim.context.clone(),
            squared_upper: DyadicBound::from_f64(0.).unwrap(),
            oracle: ToleranceTrimOracle::ExactSourceAndChartIdentities,
        })
        .collect()
}
