// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl PrismEmbeddingEvidence {
    /// Borrow unchanged policy; `evidence.requested()` records both behavior versions and budgets.
    pub(crate) fn requested(&self) -> PrismEmbeddingOptions {
        self.requested
    }
    /// Borrow all fresh source/chart operands; `evidence.binding()` includes the exact determinant.
    pub(crate) fn binding(&self) -> &CurrentPrismBinding {
        &self.binding
    }
    /// Borrow actual-source derivative evidence; `evidence.regularity()` never certifies C1 closure.
    pub(crate) fn regularity(&self) -> &BezierRegularityReport {
        &self.regularity
    }
    /// Borrow three full trimmed domains; `evidence.domains()` excludes untrimmed support surfaces.
    pub(crate) fn domains(&self) -> &[PrismFaceDomain; 3] {
        &self.domains
    }
    /// Borrow exhaustive three self/three pair rows; `evidence.incidences()` grants no body status.
    pub(crate) fn incidences(&self) -> &[PrismFaceIncidence; 6] {
        &self.incidences
    }
}
