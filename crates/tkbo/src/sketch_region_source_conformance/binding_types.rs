// SPDX-License-Identifier: GPL-2.0-only
use super::SourceConformanceInspection;
use crate::sketch_region::OriginalMemberIdentity;
pub(crate) struct BoundSourceConformance {
    pub(super) inspection: SourceConformanceInspection,
    pub(super) faults: Vec<SourceBindingFault>,
}
impl BoundSourceConformance {
    /// Borrow the owned immutable proof descriptors; `binding.inspection().entries`.
    pub(crate) fn inspection(&self) -> &SourceConformanceInspection {
        &self.inspection
    }
    /// Keep typed mismatch attribution; `binding.faults().is_empty()`.
    pub(crate) fn faults(&self) -> &[SourceBindingFault] {
        &self.faults
    }
    /// Only the full representation matrix can discharge its own numerical obligation.
    /// Example: `binding.representation_certified()` never implies contact identity.
    pub(crate) fn representation_certified(&self) -> bool {
        !self.inspection.entries.is_empty() && self.faults.is_empty()
    }
}
#[derive(Clone, Debug)]
pub(crate) enum SourceBindingFault {
    Policy,
    EmptyUniverse,
    OriginalSnapshot,
    ConstructionSnapshot,
    CurrentSegment(u64),
    CurrentGeometry {
        occurrence: u64,
        cause: std::sync::Arc<super::SourceConformanceCause>,
    },
    Matrix {
        occurrence: u64,
        original: OriginalMemberIdentity,
    },
    Stages {
        occurrence: u64,
        original: OriginalMemberIdentity,
    },
    WorkReceipt,
}
