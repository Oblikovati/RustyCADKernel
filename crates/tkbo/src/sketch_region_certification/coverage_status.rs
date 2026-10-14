// SPDX-License-Identifier: GPL-2.0-only
use super::{BoundaryCensus, EndpointEvidence, EndpointGap};
pub(super) fn missing_states(
    census: &BoundaryCensus,
    bound: bool,
    vertices: &[EndpointEvidence],
) -> Vec<EndpointGap> {
    let mut missing = vec![
        EndpointGap::DualAxisParameterUncertified,
        EndpointGap::CanonicalContactIdentityUncertified,
        EndpointGap::World3dErrorUncertified,
        EndpointGap::PartitionUncertified,
    ];
    if !super::source_status::representation_matches(census) {
        missing.push(EndpointGap::RepresentationUncertified);
    }
    if !bound {
        missing.push(EndpointGap::IncidenceSourceUnbound);
    }
    if vertices.is_empty() || vertices.iter().any(|v| !v.parameter_in_bracket) {
        missing.push(EndpointGap::NativeCurveParameterUncertified);
    }
    missing
}
