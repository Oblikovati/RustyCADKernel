// SPDX-License-Identifier: GPL-2.0-only
//! Completion classifies preserved causes; it never erases them or creates partial success.
use super::super::{RootIsolationError as Root, signs::RootSignError};
use super::{
    BezierPairCaptureFailure as Source, BezierPairDiagonalFailure as Diagonal,
    BezierPairGraphFailure as Graph, BezierPairOperationFailure as Cause,
    BezierPairSignFailure as Policy, BezierPairWorldFailure as World,
};
/// Explicit complete, invalid, cancelled or unresolved product-domain outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierPairCompletion {
    /// Every required domain, root, correspondence and world construction is certified.
    Complete,
    /// Malformed source or unsupported policy version; typed causes retain offending values.
    Invalid,
    /// Caller cancellation; no successful partial geometry is exposed.
    Cancelled,
    /// Exhausted, unsupported or inconclusive required certificate.
    Incomplete,
}
pub(super) fn operation_completion(cause: &Cause) -> BezierPairCompletion {
    let cause = cause.original_cause();
    if cancelled_cause(cause) {
        return BezierPairCompletion::Cancelled;
    }
    if invalid_cause(cause) {
        return BezierPairCompletion::Invalid;
    }
    BezierPairCompletion::Incomplete
}
fn cancelled_cause(cause: &Cause) -> bool {
    matches!(
        cause,
        Cause::Root(Root::Cancelled)
            | Cause::Source(Source::Algebra(Root::Cancelled))
            | Cause::Sign(RootSignError::Root {
                cause: Root::Cancelled,
                ..
            })
            | Cause::Diagonal(Diagonal::Algebra(Root::Cancelled))
            | Cause::Graph(Graph::Root(Root::Cancelled))
            | Cause::World(World::Cancelled)
    )
}
fn invalid_cause(cause: &Cause) -> bool {
    matches!(
        cause,
        Cause::Source(
            Source::EmptyControls
                | Source::WeightCount { .. }
                | Source::Coordinate(_)
                | Source::Weight { .. }
        ) | Cause::WorldRequest(_)
            | Cause::SignPolicy(Policy::Version(_))
            | Cause::Root(
                Root::OptionsVersion(_)
                    | Root::Interval(_)
                    | Root::Width(_)
                    | Root::Coefficient { .. }
            )
    )
}
