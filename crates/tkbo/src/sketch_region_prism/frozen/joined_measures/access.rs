// SPDX-License-Identifier: GPL-2.0-only
use super::*;

impl JoinedPrismMeasures {
    /// Check the full owned source binding; `measures.matches_join(join)` never compares only digests.
    pub(crate) fn matches_join(&self, join: &JoinedPrismProof) -> bool {
        self.binding.as_ref() == join.replay_bytes()
    }
    /// Borrow full source evidence; `measures.binding_bytes()` includes geometry, identities and requested operation limits.
    pub(crate) fn binding_bytes(&self) -> &[u8] {
        &self.binding
    }
    /// Read the unchanged integration policy; `measures.requested().scalars` separates all quantity budgets.
    pub(crate) fn requested(&self) -> JoinedPrismMeasureOptions {
        self.requested
    }
    /// Borrow authored face/trim parents; `measures.named_faces()` never assigns identity from array position.
    pub(crate) fn named_faces(&self) -> &[JoinedPrismFace; 3] {
        &self.names
    }
    /// Find the report by an authored occurrence; `measures.face_area(&key)` excludes foreign contexts.
    pub(crate) fn face_area(&self, key: &FrozenUseKey) -> Option<NamedPrismArea<'_>> {
        let face = self.names.iter().find(|face| &face.occurrence == key)?;
        let scalar = match face.role {
            PrismImageRole::First => PrismAreaReport::Plane(&self.measures.subset.first_cap),
            PrismImageRole::Last => PrismAreaReport::Plane(&self.measures.subset.last_cap),
            PrismImageRole::Generated => PrismAreaReport::Side(&self.measures.side),
        };
        Some(NamedPrismArea { face, scalar })
    }
    /// Borrow the certified conditional sweep integral; `measures.signed_volume()` still requires body validation.
    pub(crate) fn signed_volume(&self) -> &CertifiedPolynomialMeasure {
        &self.measures.subset.signed_volume
    }
    /// Borrow the three-component area sum; `measures.total_area().components()` retains both caps and the side.
    pub(crate) fn total_area(&self) -> &CertifiedEnclosureSum {
        &self.measures.total
    }
}
impl PrismAreaReport<'_> {
    /// Read the certified world-area bounds; `named.scalar.enclosure()` includes rounding and integration error.
    pub(crate) fn enclosure(&self) -> [f64; 2] {
        match self {
            Self::Plane(report) => report.enclosure(),
            Self::Side(report) => report.enclosure(),
        }
    }
    /// Read upward-rounded area error; `named.scalar.absolute_error_bound()` is not a sampled estimate.
    pub(crate) fn absolute_error_bound(&self) -> f64 {
        match self {
            Self::Plane(report) => report.absolute_error_bound(),
            Self::Side(report) => report.absolute_error_bound(),
        }
    }
}
