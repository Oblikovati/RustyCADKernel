// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    RootIsolationError, certificate_arithmetic::world::EnvelopeFailure, signs::RootSignError,
};
use super::equations::DiagonalDivisionFailure;
use super::source::PairSourceFailure;
#[derive(Debug)]
/// Typed noncompletion; no successful partial geometry accompanies these causes.
pub enum PairOperationFailure {
    /// Actual input domain ordinal with its unchanged original typed cause.
    SourceDomain {
        /// Source curve ordinal, zero or one.
        source_index: usize,
        /// Original failure without reclassification or string conversion.
        cause: Box<PairOperationFailure>,
    },
    /// Original source capture/conversion cause.
    Source(PairSourceFailure),
    /// Original primary isolation or completed-work exhaustion cause.
    Root(RootIsolationError),
    /// Original exact algebraic-root sign cause.
    Sign(RootSignError),
    /// Rejected generated-sign policy or representability.
    SignPolicy(super::sign_policy::PairSignPolicyFailure),
    /// Exact self-diagonal division failed.
    Diagonal(DiagonalDivisionFailure),
    /// Actual denominator roots prevent regular source-domain completion.
    DenominatorRoots(super::super::RootIsolation),
    /// Actual alpha/beta common root leaves a whole fibre unresolved.
    ExceptionalFiber(super::super::signs::RootSignEvaluation),
    /// Generic ideal reduction did not establish a linear fibre.
    NonLinearFiber,
    /// Both cleared originals vanish; correspondence overlap remains unproved.
    PositiveDimensional,
    /// Original world construction cause.
    World(EnvelopeFailure),
    /// The current point construction cannot meet the request at this singleton root.
    WorldResolution {
        /// Unchanged offending construction-error request.
        requested_error: f64,
        /// Last exact certified root bracket.
        last_root: Box<super::super::RootBracket>,
        /// Last world bounds, constructed point and achieved error, if available.
        enclosure: Option<([[f64; 2]; 2], [f64; 2], f64)>,
    },
    /// Re-isolation failed unique-root containment or multiplicity invariants.
    RefinementInvariant(Box<super::super::RootIsolation>),
    /// Original paired-parameter graph enclosure cause.
    Graph(super::graph_enclosure::GraphEnvelopeFailure),
    /// Offending nonfinite or nonpositive world error request.
    WorldRequest(f64),
}

impl PairOperationFailure {
    /// Read the original typed cause through source-domain context. Example: `cause.original_cause()`.
    pub fn original_cause(&self) -> &Self {
        match self {
            Self::SourceDomain { cause, .. } => cause.original_cause(),
            other => other,
        }
    }
    fn typed_cause(&self) -> Option<&dyn std::fmt::Debug> {
        match self {
            Self::Source(cause) => Some(cause),
            Self::Root(cause) => Some(cause),
            Self::Sign(cause) => Some(cause),
            Self::SignPolicy(cause) => Some(cause),
            Self::Diagonal(cause) => Some(cause),
            Self::World(cause) => Some(cause),
            Self::Graph(cause) => Some(cause),
            Self::ExceptionalFiber(certificate) => Some(certificate),
            Self::DenominatorRoots(certificate) => Some(certificate),
            Self::WorldRequest(value) => Some(value),
            _ => self.refinement_cause(),
        }
    }
    fn refinement_cause(&self) -> Option<&dyn std::fmt::Debug> {
        match self {
            Self::SourceDomain { cause, .. } => Some(cause),
            Self::RefinementInvariant(isolation) => Some(isolation),
            Self::WorldResolution {
                requested_error,
                last_root,
                enclosure,
            } => {
                let _ = (requested_error, enclosure);
                Some(last_root)
            }
            _ => None,
        }
    }
}
impl std::fmt::Display for PairOperationFailure {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(cause) = self.typed_cause() {
            return write!(
                output,
                "{self:?}: {cause:?}; expected complete finite product-domain proof"
            );
        }
        write!(
            output,
            "{self:?}; expected complete finite product-domain proof"
        )
    }
}

pub(super) fn source_domain_failure(
    source_index: usize,
    cause: PairOperationFailure,
) -> PairOperationFailure {
    PairOperationFailure::SourceDomain {
        source_index,
        cause: Box::new(cause),
    }
}
