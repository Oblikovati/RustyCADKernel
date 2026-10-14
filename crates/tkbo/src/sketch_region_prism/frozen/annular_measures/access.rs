// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl AnnularPrismMeasures {
    /// Bind the whole operation, including options and history; `report.matches(join)`.
    pub(in crate::sketch_region_prism) fn matches(&self, join: &AnnularFrozenDomain) -> bool {
        self.binding.as_ref() == join.replay_bytes()
    }
    /// Borrow named complete per-face quantities; `report.faces()` keeps both sides separate.
    pub(in crate::sketch_region_prism) fn faces(&self) -> &[AnnularMeasuredFace] {
        &self.faces
    }
    /// Read requested limits without substituting child clamps; `report.requested()`.
    pub(in crate::sketch_region_prism) fn requested(&self) -> AnnularMeasureOptions {
        self.requested
    }
    /// Read the signed normalized-material sweep; `report.signed_sweep()` grants no body volume.
    pub(in crate::sketch_region_prism) fn signed_sweep(&self) -> &CertifiedPolynomialMeasure {
        &self.signed_sweep
    }
    /// Borrow the exact four-component sum; `report.total_area().components()`.
    pub(in crate::sketch_region_prism) fn total_area(&self) -> &CertifiedEnclosureSum {
        &self.total
    }
    /// Read all completed side work; `report.side_subdivisions()` includes both sources.
    pub(in crate::sketch_region_prism) fn side_subdivisions(&self) -> u32 {
        self.side_subdivisions
    }
    /// Borrow exact proof-bound replay; `report.replay_bytes()` performs no mutable read.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
impl AnnularFaceMeasure {
    /// Read the conditional area enclosure; `face.measure.enclosure()` retains its scalar kind.
    pub(in crate::sketch_region_prism) fn enclosure(&self) -> [f64; 2] {
        match self {
            Self::Cap(report) => report.enclosure(),
            Self::Side { report, .. } => report.enclosure(),
        }
    }
    /// Read the certified width for the lower representative; `face.measure.absolute_error_bound()`.
    pub(in crate::sketch_region_prism) fn absolute_error_bound(&self) -> f64 {
        match self {
            Self::Cap(report) => report.absolute_error_bound(),
            Self::Side { report, .. } => report.absolute_error_bound(),
        }
    }
}
