// SPDX-License-Identifier: GPL-2.0-only
//! A Jordan-domain certificate binds one current boundary; it is not body validation.
use super::super::surface_charts::{CapturedPrismSources, PrismSurfaceCause};
use crate::sketch_region::OriginalMemberIdentity;

// Compatibility occurrence keys retain Location handles. This private receipt is thread-confined.
pub(crate) struct RegionCertificateInspection {
    pub version: u32,
    pub decision: Result<RegionCertificate, RegionCertificateCause>,
}
// This private certificate includes thread-confined compatibility identity keys.
pub(crate) struct RegionCertificate {
    pub original: OriginalMemberIdentity,
    pub census_occurrence: u64,
    pub contact: crate::sketch_region_contact_transport::ReportContactIdentity,
    pub source_candidate: CapturedPrismSources,
    pub complementary_candidate: CapturedPrismSources,
    pub copied: CapturedPrismSources,
    pub boundary_reversed: bool,
    pub selected_winding: i64,
}
#[derive(Debug)]
pub(crate) enum RegionCertificateCause {
    UnavailableCandidates,
    MissingEvidence {
        hypothesis: &'static str,
    },
    Hypothesis {
        name: &'static str,
        actual: usize,
        expected: usize,
    },
    Capture(std::sync::Arc<PrismSurfaceCause>),
    Mapped(rustycad_tkernel::standard::Failure),
}
// Private operation refusal; compatibility failures are not published Send/Sync results.
pub(crate) struct RegionPrismUnavailable {
    pub requested: super::super::PrismRequest,
    pub cause: RegionCertificateCause,
    pub missing: Vec<crate::sketch_region_candidates::MissingCertificate>,
}
pub(super) fn require(
    name: &'static str,
    actual: usize,
    expected: usize,
) -> Result<(), RegionCertificateCause> {
    if actual != expected {
        return Err(RegionCertificateCause::Hypothesis {
            name,
            actual,
            expected,
        });
    }
    Ok(())
}
pub(super) fn holds(name: &'static str, condition: bool) -> Result<(), RegionCertificateCause> {
    // Preserve the failed hypothesis without pretending that an unsupported domain is invalid.
    let actual = usize::from(condition);
    require(name, actual, 1)
}

pub(super) type RegionCheck<T = ()> = Result<T, RegionCertificateCause>;

pub(super) fn need<T>(value: Option<T>, hypothesis: &'static str) -> RegionCheck<T> {
    // Missing evidence is incomplete; it cannot be substituted with a previous status row.
    value.ok_or(RegionCertificateCause::MissingEvidence { hypothesis })
}
