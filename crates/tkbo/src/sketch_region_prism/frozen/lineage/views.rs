// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed immutable views own all provenance values; selected geometry is historical/unproven.
use super::*;
impl PrismLineageInspection {
    /// Borrow member observations with exact parent classes; example: `inspection.members().len()`.
    pub(crate) fn members(&self) -> &[CopyMemberObservation] {
        &self.members
    }
    /// Separate member and directed coverage; example: `inspection.coverage()`.
    pub(crate) fn coverage(&self) -> (CopyLineageCoverage, CopyLineageCoverage) {
        (self.member_coverage, self.directed_coverage)
    }
    /// Borrow uniquely mapped directed uses; example: `inspection.directed().len()`.
    pub(crate) fn directed(&self) -> &[CopyDirectedObservation] {
        &self.directed
    }
    /// Borrow actual sweep observations; example: `inspection.sweep().len()`.
    pub(crate) fn sweep(&self) -> &[CopySweepObservation] {
        &self.sweep
    }
    /// Borrow copied-intermediate retirements, never original deletions; example: `inspection.retired().len()`.
    pub(crate) fn retired(&self) -> &[CopyRetirementObservation] {
        &self.retired
    }
    /// Borrow typed failures; example: `inspection.findings().is_empty()`.
    pub(crate) fn findings(&self) -> &[CopyLineageFinding] {
        &self.findings
    }
    /// Borrow the same-operation diagnostic projection; example: `inspection.projection().is_some()`.
    pub(crate) fn projection(&self) -> Option<&FrozenPrismProjection> {
        self.projection.as_deref()
    }
    /// This inspection cannot publish a valid body; example: `assert!(!inspection.is_valid_body())`.
    pub(crate) fn is_valid_body(&self) -> bool {
        false
    }
    /// Borrow unresolved publication obligations; example: `inspection.limitations()`.
    pub(crate) fn limitations(&self) -> &[CopyLineageLimitation] {
        &[
            CopyLineageLimitation::HistoricalSelectedGeometryUnproven,
            CopyLineageLimitation::UpstreamPartitionProvenanceUnbound,
            CopyLineageLimitation::OriginalDeletionCompositionIncomplete,
            CopyLineageLimitation::WholeBodyValidationIncomplete,
        ]
    }
    /// Stable exact owned bytes for replay; example: `inspection.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut result = self.lineage_bytes();
        result.extend(format!("findings={:?};", self.findings).bytes());
        if let Some(projection) = &self.projection {
            result.extend(projection.replay_bytes());
        }
        result
    }
    pub(super) fn lineage_bytes(&self) -> Vec<u8> {
        format!("prism-lineage-v1;operation={:?};coverage={:?};members={:?};contexts={:?};directed={:?};sweep={:?};retired={:?};limitations={:?};",
            self.operation, self.coverage(), self.members, self.contexts, self.directed, self.sweep, self.retired, self.limitations()).into_bytes()
    }
}
