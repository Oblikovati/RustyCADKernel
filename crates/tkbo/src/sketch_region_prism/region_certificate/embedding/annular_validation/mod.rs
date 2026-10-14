// SPDX-License-Identifier: GPL-2.0-only
//! Recheck the sealed annular theorem table against its actual immutable binding.
mod checks;
mod error;
use super::super::{annular_binding::AnnularChartIdentity, hypothesis::RegionHypothesisCause};
use super::{annular_coverage::*, annular_domains::*, annular_operation::*, annular_table::*};
pub(in crate::sketch_region_prism) use error::AnnularTableFailure;
type TableCheck<T = ()> = Result<T, AnnularTableFailure>;

/// Check every retained domain and pair; `check_bound_annular_table(evidence, |_| false)`.
/// This reuses the existing embedding theorem; it grants no independent geometry authority.
pub(in crate::sketch_region_prism) fn check_bound_annular_table(
    evidence: &AnnularEmbeddingEvidence,
    mut cancelled: impl FnMut(usize) -> bool,
) -> TableCheck {
    checks::poll(&mut cancelled, 0)?;
    let expected = capture_annular_domains(evidence.binding()).map_err(reconstruction)?;
    checks::domains(evidence.domains(), &expected)?;
    checks::rows(&expected, evidence.incidences(), &mut cancelled)
}
fn reconstruction(cause: RegionHypothesisCause) -> AnnularTableFailure {
    AnnularTableFailure::Reconstruction(Box::new(cause))
}
#[cfg(test)]
mod tests;
