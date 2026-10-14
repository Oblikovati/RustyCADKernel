// SPDX-License-Identifier: GPL-2.0-only
//! Exact local originals stay available only while their real producing invocation lives.
use super::super::{
    OwnedPolynomialPrismRequest, OwnedPrismPreparationFailure, PreparedPolynomialPrism,
    resolver::{InvokedPrismPublicationFailure, ResolutionCause},
};
use super::{context::*, projection::ProjectionRefusal};
pub(crate) struct ScopedPrismFailure<'a> {
    pub(super) callback: Option<CompletePrismCallbackStop>,
    pub(super) cause: ScopedPrismCause<'a>,
}
pub(crate) enum ScopedPrismCause<'a> {
    Preparation(Box<OwnedPrismPreparationFailure>),
    Resolution {
        prepared: &'a PreparedPolynomialPrism,
        cause: Box<ResolutionCause>,
    },
    Publication(Box<InvokedPrismPublicationFailure<'a>>),
    Projection(Box<ProjectionRefusal>),
}
impl<'a> ScopedPrismFailure<'a> {
    /// Read exact originating input; `failure.request()` never accepts detached context.
    pub(crate) fn request(&self) -> &OwnedPolynomialPrismRequest {
        match &self.cause {
            ScopedPrismCause::Preparation(value) => &value.retained.request,
            ScopedPrismCause::Resolution { prepared, .. } => &prepared.retained.request,
            ScopedPrismCause::Publication(value) => value.request(),
            ScopedPrismCause::Projection(value) => value.invoked.request(),
        }
    }
    /// Keep outer callback provenance distinct from the child class; `failure.callback()`.
    pub(crate) fn callback(&self) -> Option<&CompletePrismCallbackStop> {
        self.callback.as_ref()
    }
    /// Borrow the exact original stage owner; `failure.cause()` grants no owned-public-error claim.
    pub(crate) fn cause(&self) -> &ScopedPrismCause<'a> {
        &self.cause
    }
}

impl<'a> ScopedPrismFailure<'a> {
    /// Transfer actual local originals only within the scope; `failure.into_parts()`.
    pub(crate) fn into_parts(self) -> (Option<CompletePrismCallbackStop>, ScopedPrismCause<'a>) {
        (self.callback, self.cause)
    }
}
