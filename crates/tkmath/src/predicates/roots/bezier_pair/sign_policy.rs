// SPDX-License-Identifier: GPL-2.0-only
//! Generated sign admission is explicit; it never widens primary isolation admission.
use super::projection::LinearFiberProjection;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Declared generated-polynomial degree admission, separate from primary isolation.
pub enum PairSignAdmission {
    /// Conservative bound derived from the actual original-ideal projection.
    DerivedProjection,
    /// Explicit maximum generated-sign degree; never silently widened.
    Explicit(u16),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Versioned generated-sign admission request.
pub struct PairSignOptions {
    /// Sign policy version; expected one.
    pub version: u32,
    /// Source-derived or explicit generated-sign degree admission.
    pub degree_limit: PairSignAdmission,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Requested admission plus measured conservative bound and effective cap.
pub struct EffectivePairSigns {
    /// Original requested sign policy.
    pub requested: PairSignOptions,
    /// Source-bound maximum of retained original/domain/diagonal sign degrees.
    pub conservative_generated_degree: usize,
    /// Actual generated-sign admission cap.
    pub effective_limit: u16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Generated-sign policy could not be admitted.
pub enum PairSignPolicyFailure {
    /// Offending policy version; expected one.
    Version(u32),
    /// Actual derived degree exceeds u16 policy representation.
    DerivedDegreeOverflow(usize),
}
impl PairSignOptions {
    /// Admit source-derived sign degrees explicitly; primary isolation admission stays independent. Example: `BezierPairSignOptions::derived()`.
    pub fn derived() -> Self {
        Self {
            version: 1,
            degree_limit: PairSignAdmission::DerivedProjection,
        }
    }
    pub(super) fn effective(
        self,
        row: &LinearFiberProjection,
        self_domain: bool,
    ) -> Result<EffectivePairSigns, PairSignPolicyFailure> {
        if self.version != 1 {
            return Err(PairSignPolicyFailure::Version(self.version));
        }
        let degree = generated_degree_bound(row, self_domain);
        let effective_limit = match self.degree_limit {
            PairSignAdmission::Explicit(limit) => limit,
            PairSignAdmission::DerivedProjection => u16::try_from(degree)
                .map_err(|_| PairSignPolicyFailure::DerivedDegreeOverflow(degree))?,
        };
        Ok(EffectivePairSigns {
            requested: self,
            conservative_generated_degree: degree,
            effective_limit,
        })
    }
}
fn generated_degree_bound(row: &LinearFiberProjection, self_domain: bool) -> usize {
    let alpha = row.alpha.degree().unwrap_or(0);
    let beta = row.beta.degree().unwrap_or(0);
    let original = row
        .cleared_originals
        .iter()
        .filter_map(|p| p.degree())
        .max()
        .unwrap_or(0);
    let domain = alpha + alpha.max(beta);
    let diagonal = if self_domain {
        alpha + (alpha + 1).max(beta)
    } else {
        0
    };
    original.max(beta).max(domain).max(diagonal)
}
