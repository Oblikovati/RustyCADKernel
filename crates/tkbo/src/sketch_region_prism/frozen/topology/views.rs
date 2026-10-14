// SPDX-License-Identifier: GPL-2.0-only
use super::super::FrozenPrismProjection;
use super::types::*;
impl FrozenTopologyCertificate {
    /// Behavior identity for exact incidence only; example: `assert_eq!(report.version(), 1)`.
    pub(crate) fn version(&self) -> u32 {
        // Certificate behavior is independent of its required capture V2.
        1
    }
    /// Check the exact immutable input join; example: `report.binds(&projection)`.
    pub(crate) fn binds(&self, projection: &FrozenPrismProjection) -> bool {
        let actual = projection.replay_bytes();
        self.source.as_ref() == actual
    }
    /// Metric tolerances cannot repair incidence; example: `report.achieved_error()`.
    pub(crate) fn achieved_error(&self) -> TopologyNumericalError {
        // Metric error cannot describe discrete incidence equality.
        TopologyNumericalError::NotApplicable
    }
    /// This evidence uses exact discrete incidence; example: `report.evidence_kind()`.
    pub(crate) fn evidence_kind(&self) -> TopologyEvidenceKind {
        // No sampled geometry contributes to this discrete certificate.
        TopologyEvidenceKind::ExactCombinatorial
    }
    /// Borrow the completed obligations; example: `assert_eq!(report.invariants().len(), 6)`.
    pub(crate) fn invariants(&self) -> [TopologyInvariant; 6] {
        use TopologyInvariant::*;
        [
            ClosedContainment,
            CyclicWires,
            OppositeEdgeUses,
            CyclicVertexLinks,
            ConnectedFaces,
            SphereEuler,
        ]
    }
    /// Replay the certificate and its complete source binding; example: `report.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = format!("frozen-topology-v{};operation={:?};root={:?};counts={:?};evidence={:?};error={:?};checks={:?};source-length={};",
            self.version(), self.operation, self.root, self.counts, self.evidence_kind(), self.achieved_error(), self.invariants(), self.source.len()).into_bytes();
        bytes.extend_from_slice(&self.source);
        bytes
    }
}
