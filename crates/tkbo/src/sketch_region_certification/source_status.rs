// SPDX-License-Identifier: GPL-2.0-only
use super::BoundaryCensus;
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::{
    RootIsolationError, metric_bounds::ResidualError, signs::RootSignError,
};
pub(super) fn representation_matches(census: &BoundaryCensus) -> bool {
    let Some(binding) = &census.source_binding else {
        return false;
    };
    let Some(policy) = census.source_policy else {
        return false;
    };
    binding.representation_certified()
        && source_binding_faults(&census.inputs, policy, binding.inspection()).is_empty()
}
pub(super) fn source_cancelled(report: &SourceConformanceInspection) -> bool {
    report
        .entries
        .iter()
        .flat_map(|row| &row.stages)
        .any(|stage| {
            matches!(
                &stage.decision,
                Err(SourceConformanceCause::Exact(ResidualError::Roots(
                    RootIsolationError::Cancelled
                ))) | Err(SourceConformanceCause::Exact(ResidualError::Signs(
                    RootSignError::Root {
                        cause: RootIsolationError::Cancelled,
                        ..
                    }
                )))
            )
        })
}
