// SPDX-License-Identifier: GPL-2.0-only
//! Present native diagnostics cannot disappear behind an absent successful batch.
use super::{failure::*, request::*, work::*};
use rustycad_tkernel::message::Gravity;
pub(super) fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    let copy = request.bridge.mapped.copied.as_ref().ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "actual candidate copy stage",
        })
    })?;
    let batch = copy.completion.as_ref().map_err(|cause| {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "candidate construction outcome",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Candidate(cause));
        failure
    })?;
    if !copy.observation_failures.is_empty() {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "actual copy observation failures",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Copy(copy));
        return Err(failure);
    }
    for gravity in Gravity::ALL {
        if !batch.mapped_report.get_alerts(gravity).is_empty() {
            let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
                stage: "required candidate report alert payload",
            });
            failure.local = Some(TwoLoopLocalDiagnostic::Report {
                gravity,
                report: &batch.mapped_report,
            });
            return Err(failure);
        }
    }
    Ok(())
}
