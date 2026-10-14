// SPDX-License-Identifier: GPL-2.0-only
//! Present native diagnostics cannot disappear behind an absent successful batch.
use super::{failure::*, request::*, work::*};
use rustycad_tkernel::message::Gravity;
pub(super) fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    arrangement(&request.bridge.mapped.arrangement.completion, work)?;
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
    copy_observations(copy, work)?;
    report(&batch.mapped_report, work)
}
fn report<'a>(
    report: &'a rustycad_tkernel::message::Report,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    for gravity in Gravity::ALL {
        if !report.get_alerts(gravity).is_empty() {
            let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
                stage: "required candidate report alert payload",
            });
            failure.local = Some(TwoLoopLocalDiagnostic::Report { gravity, report });
            return Err(failure);
        }
    }
    Ok(())
}

fn arrangement<'a>(
    completion: &'a crate::sketch_region::ArrangementCompletion,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    if matches!(
        completion,
        crate::sketch_region::ArrangementCompletion::Arranged(_)
    ) {
        return Ok(());
    }
    let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
        stage: "required actual arrangement completion",
    });
    failure.local = Some(TwoLoopLocalDiagnostic::Arrangement(completion));
    Err(failure)
}

fn copy_observations<'a>(
    copy: &'a crate::sketch_region_candidates::CandidateCopyInspection,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, ()> {
    if !copy.observation_failures.is_empty() {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "actual copy observation failures",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::Copy(copy));
        return Err(failure);
    }
    Ok(())
}
