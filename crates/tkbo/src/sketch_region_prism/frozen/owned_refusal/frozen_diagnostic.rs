// SPDX-License-Identifier: GPL-2.0-only
//! An unknown alert payload remains local even when the outer frozen diagnostic is owned.
use super::*;
pub(super) fn project_capture(
    original: PrismFreezeFailure,
) -> ProjectionCheck<PrismFreezeFailure, PrismFreezeFailure> {
    let gap = unsupported_diagnostic(&original.cause);
    retain_or_complete(original, gap)
}
pub(super) fn project_link(
    original: PrismFreezeCause,
) -> ProjectionCheck<PrismFreezeCause, PrismFreezeCause> {
    let gap = unsupported_diagnostic(&original);
    retain_or_complete(original, gap)
}
fn retain_or_complete<T>(
    original: T,
    gap: Option<PrismRefusalProjectionGap>,
) -> ProjectionCheck<T, T> {
    match gap {
        Some(cause) => Err(RetainedProjection::gap(
            original,
            PrismRefusalPath::Frozen,
            cause,
        )),
        None => Ok(original),
    }
}
fn unsupported_diagnostic(cause: &PrismFreezeCause) -> Option<PrismRefusalProjectionGap> {
    match cause {
        PrismFreezeCause::Entities { cause, .. } => unsupported_diagnostic(cause),
        PrismFreezeCause::UnsupportedDiagnostic { stage, actual } => {
            Some(PrismRefusalProjectionGap::UnprojectedDiagnostic {
                stage: *stage,
                actual,
            })
        }
        _ => None,
    }
}
