// SPDX-License-Identifier: GPL-2.0-only
//! Closure point contexts need an explicit stage-use dictionary; preserve them locally until provided.
use super::*;
pub(super) fn project(
    cause: RegionCertificateCause,
) -> ProjectionCheck<OwnedPrismRegionCause, RegionCertificateCause> {
    use OwnedPrismRegionCause as O;
    use RegionCertificateCause as L;
    Ok(match cause {
        L::UnavailableCandidates => O::UnavailableCandidates,
        L::MissingEvidence { hypothesis } => O::MissingEvidence { hypothesis },
        L::Hypothesis {
            name,
            actual,
            expected,
        } => hypothesis(name, actual, expected),
        L::Capture(value) => O::Capture(value),
        L::Mapped(value) => O::Mapped(value),
        L::ClosurePoint(value) => return pending_closure(value),
    })
}

fn hypothesis(name: &'static str, actual: usize, expected: usize) -> OwnedPrismRegionCause {
    OwnedPrismRegionCause::Hypothesis {
        name,
        actual,
        expected,
    }
}
fn pending_closure(
    value: Box<ClosurePointEvidence>,
) -> ProjectionCheck<OwnedPrismRegionCause, RegionCertificateCause> {
    let retained = RetainedProjection::pending(
        RegionCertificateCause::ClosurePoint(value),
        PrismRefusalPath::Region,
        PendingPrismRefusalPayload::ClosurePoint,
    );
    Err(retained)
}
