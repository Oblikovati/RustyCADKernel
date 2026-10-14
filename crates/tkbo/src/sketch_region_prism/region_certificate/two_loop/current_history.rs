// SPDX-License-Identifier: GPL-2.0-only
//! Existing full generated-universe/history inspection follows the bounded live hierarchy capture.
use super::{failure::*, request::*, work::*};
use crate::sketch_region_partition_bindings::inspect_outputs;
pub(super) fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    generated_universe(request, work)?;
    let current = inspect_outputs(
        &request.bridge.mapped.arrangement.partition,
        request.bridge.mapped.copied.as_ref(),
        &request.bridge.census.inputs.originals,
        request.sources,
    )
    .map_err(|cause| work.failure(TwoLoopRegionCause::Native(cause)))?;
    if let Err(cause) = super::super::history::require_history(&current) {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "current complete candidate history",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Partition {
            cause: Box::new(cause),
            inspection: Box::new(current),
        });
        return Err(failure);
    }
    Ok(())
}

fn generated_universe<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    super::super::containment::generated_universe(request.bridge).map_err(|cause| {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "current generated candidate universe",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Region(Box::new(cause)));
        failure
    })?;
    Ok(())
}
