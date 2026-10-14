// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::{
    owned_request::{OwnedPolynomialPrismRequest, names::SingleBoundaryNamePlan},
    publication::PublishedNamedPrism,
};
impl OwnedProjectionRefusal {
    /// This diagnostic bridge has its own format version; `failure.version()` changes no producer behavior.
    pub(crate) fn version(&self) -> u32 {
        // Producer request versions remain independent of this transport format.
        1
    }
    /// Read exact actual caller operands and policies; `failure.request()` requires no live owner.
    pub(crate) fn request(&self) -> &OwnedPolynomialPrismRequest {
        let invoked = &self.original.invoked;
        invoked.request()
    }
    /// Read the dictionary actually used; `failure.plan()` does not imply that a map projection succeeded.
    pub(crate) fn plan(&self) -> &SingleBoundaryNamePlan {
        let invoked = &self.original.invoked;
        invoked.plan()
    }
    /// Retained coverage is only this accepted child; `failure.accepted_publication_context()` is not outer completion.
    pub(crate) fn accepted_publication_context(&self) -> &PublishedNamedPrism {
        let invoked = &self.original.invoked;
        invoked.result()
    }
    /// Borrow the exact moved payload; `failure.cause()` performs no geometry or identity reconstruction.
    pub(crate) fn cause(&self) -> &CompleteProjectionCause {
        let original = &self.original;
        &original.cause
    }
    /// Preserve the exact outer stop independently; `failure.callback()` does not rewrite child cancellation.
    pub(crate) fn callback(&self) -> Option<&CompletePrismCallbackStop> {
        let callback = &self.callback;
        callback.as_ref()
    }
    /// Callback phase is its actual checkpoint; other phases follow the retained typed child.
    /// `failure.phase()` is absent only if a callback-only cause lacks a retained stop.
    pub(crate) fn phase(&self) -> Option<CompletePrismPhase> {
        if let Some(stop) = &self.callback {
            return Some(stop.checkpoint.phase);
        }
        match self.cause() {
            CompleteProjectionCause::Callback => None,
            CompleteProjectionCause::Names(_) => Some(CompletePrismPhase::Names),
            CompleteProjectionCause::Reservations(_) => Some(CompletePrismPhase::Reservations),
            CompleteProjectionCause::Body(_) => Some(CompletePrismPhase::Body),
            CompleteProjectionCause::Volume(_) => Some(CompletePrismPhase::Volume),
        }
    }
}
