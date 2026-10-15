// SPDX-License-Identifier: GPL-2.0-only
//! The exclusion retains the actual exact source equation, not a synthetic resultant.
use super::{ExactPairPolynomial, constant_exclusion::ConstantEquationExclusion};
/// Generated-sign execution is separate from requested admission policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierPairSignStage {
    /// Graph-sign admission and evaluations were performed with these settings.
    Evaluated(super::BezierPairEffectiveSigns),
    /// An exact constant excludes all roots; graph predicates were unnecessary.
    SkippedConstantExclusion,
    /// ADR 0066 part B: both sources share one carrier; exact parameter arcs replaced graph signs.
    SkippedCommonCarrier,
}
/// Borrowed exact total-degree-zero coordinate equation proving empty intersection.
pub struct BezierPairConstantExclusion<'a> {
    proof: &'a ConstantEquationExclusion,
}
impl<'a> BezierPairConstantExclusion<'a> {
    pub(super) fn new(proof: &'a ConstantEquationExclusion) -> Self {
        let retained = proof;
        // Its owned equations share the report's genuine source/domain preparation.
        Self { proof: retained }
    }
    /// Read the original coordinate equation axis, zero or one. Example: `exclusion.coordinate_axis()`.
    pub fn coordinate_axis(&self) -> usize {
        let original = self.proof.axis;
        // No axis permutation occurs when selecting the first exact constant witness.
        original
    }
    /// Read the actual nonzero constant polynomial. Example: `exclusion.constant().signed_coefficient(0).unwrap().to_string()`.
    pub fn constant(&self) -> ExactPairPolynomial<'a> {
        let equation = &self.proof.equations.0[self.proof.axis];
        ExactPairPolynomial::new(&equation.0[0].0)
    }
    /// Read whether exact self-diagonal division preceded exclusion. Example: `exclusion.divided_self_diagonal()`.
    pub fn divided_self_diagonal(&self) -> bool {
        let checked = self.proof.divided_self_diagonal;
        // Division is source-derived, never a caller-supplied identity certificate.
        checked
    }
    /// Read all retained powers of u in either original reduced equation. Example: `exclusion.equation(0).unwrap().count()`.
    pub fn equation(
        &self,
        axis: usize,
    ) -> Option<impl ExactSizeIterator<Item = ExactPairPolynomial<'a>>> {
        let equation = self.proof.equations.0.get(axis)?;
        Some(
            equation
                .0
                .iter()
                .map(|power| ExactPairPolynomial::new(&power.0)),
        )
    }
}
