// SPDX-License-Identifier: GPL-2.0-only
//! Generated sign admission is explicit; it never widens primary isolation admission.
use super::projection::LinearFiberProjection;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PairSignAdmission { DerivedProjection, Explicit(u16) }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PairSignOptions {
    pub version: u32,
    pub degree_limit: PairSignAdmission,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EffectivePairSigns {
    pub requested: PairSignOptions,
    pub conservative_generated_degree: usize,
    pub effective_limit: u16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PairSignPolicyFailure {
    Version(u32), DerivedDegreeOverflow(usize),
}
impl PairSignOptions {
    pub fn derived() -> Self {
        Self { version: 1, degree_limit: PairSignAdmission::DerivedProjection }
    }
    pub fn effective(self, row: &LinearFiberProjection, self_domain: bool) -> Result<EffectivePairSigns, PairSignPolicyFailure> {
        if self.version != 1 { return Err(PairSignPolicyFailure::Version(self.version)); }
        let degree = generated_degree_bound(row, self_domain);
        let effective_limit = match self.degree_limit {
            PairSignAdmission::Explicit(limit) => limit,
            PairSignAdmission::DerivedProjection => u16::try_from(degree).map_err(|_| PairSignPolicyFailure::DerivedDegreeOverflow(degree))?,
        };
        Ok(EffectivePairSigns { requested: self, conservative_generated_degree: degree, effective_limit })
    }
}
fn generated_degree_bound(row: &LinearFiberProjection, self_domain: bool) -> usize {
    let alpha = row.alpha.degree().unwrap_or(0);
    let beta = row.beta.degree().unwrap_or(0);
    let original = row.cleared_originals.iter().filter_map(|p| p.degree()).max().unwrap_or(0);
    let domain = alpha + alpha.max(beta);
    let diagonal = if self_domain { alpha + (alpha + 1).max(beta) } else { 0 };
    original.max(beta).max(domain).max(diagonal)
}
