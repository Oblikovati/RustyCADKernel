// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    RootIsolationError, certificate_arithmetic::world::EnvelopeFailure, signs::RootSignError,
};
use super::equations::DiagonalDivisionFailure;
use super::source::PairSourceFailure;
#[derive(Debug)]
pub(super) enum PairOperationFailure {
    Source(PairSourceFailure),
    Root(RootIsolationError),
    Sign(RootSignError),
    Diagonal(DiagonalDivisionFailure),
    DenominatorRoots,
    ExceptionalFiber,
    NonLinearFiber,
    PositiveDimensional,
    World(EnvelopeFailure),
    WorldResolution,
    Graph(super::graph_enclosure::GraphEnvelopeFailure),
    WorldRequest(f64),
}

impl PairOperationFailure {
    fn typed_cause(&self) -> Option<&dyn std::fmt::Debug> {
        match self {
            Self::Source(cause) => Some(cause),
            Self::Root(cause) => Some(cause),
            Self::Sign(cause) => Some(cause),
            Self::Diagonal(cause) => Some(cause),
            Self::World(cause) => Some(cause),
            Self::Graph(cause) => Some(cause),
            Self::WorldRequest(value) => Some(value),
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
