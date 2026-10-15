// SPDX-License-Identifier: GPL-2.0-only
//! Read-only ADR 0066 part B common-carrier evidence of a completed pair report.
use super::carrier::{BezierPairCarrierKind, CommonCarrierEvidence, ProjectiveParameter};

/// Exact first-source parameter, in lowest terms, or the point at infinity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BezierPairCarrierParameter {
    /// Finite exact rational `numerator / denominator` with a positive denominator.
    Finite {
        /// Signed decimal numerator.
        numerator: String,
        /// Positive decimal denominator.
        denominator: String,
    },
    /// The projective parameter at infinity.
    Infinite,
}
/// Borrowed carrier class and parameter correspondence of a common-carrier proof.
pub struct BezierPairCommonCarrier<'a> {
    evidence: &'a CommonCarrierEvidence,
}
impl<'a> BezierPairCommonCarrier<'a> {
    pub(super) fn new(evidence: &'a CommonCarrierEvidence) -> Self {
        Self { evidence }
    }
    /// Read the shared carrier class. Example: `carrier.kind() == BezierPairCarrierKind::Conic`.
    pub fn kind(&self) -> BezierPairCarrierKind {
        self.evidence.kind
    }
    /// Read the first-source preimages of the second source's start, midpoint and end.
    /// Example: `let [start, middle, end] = carrier.preimages();`.
    pub fn preimages(&self) -> [BezierPairCarrierParameter; 3] {
        self.evidence.preimages.each_ref().map(|value| match value {
            ProjectiveParameter::Infinite => BezierPairCarrierParameter::Infinite,
            ProjectiveParameter::Finite(ratio) => BezierPairCarrierParameter::Finite {
                numerator: ratio.numerator().to_string(),
                denominator: ratio.denominator().to_string(),
            },
        })
    }
}
