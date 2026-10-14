// SPDX-License-Identifier: GPL-2.0-only
//! Owned evidence is read without locks; retained local originals do not cross publication.
use super::*;
impl OwnedPrismPublicationDiagnostic {
    /// Read projection version, separate from publication policy; `diagnostic.version()`.
    pub(crate) fn version(&self) -> u32 {
        // This version describes the bounded owned projection, not body acceptance.
        1
    }
    /// Read actual invocation context; `diagnostic.context().request().operation`.
    pub(crate) fn context(&self) -> &OwnedPrismDiagnosticContext {
        // Context was captured by the actual invocation issuer.
        &self.context
    }
    /// Read observed internal operation token; `diagnostic.operation()` requires its context and coverage.
    pub(crate) fn operation(&self) -> AuthoredPrismId {
        // Reusable authored tokens are not caller persistence identities by themselves.
        self.operation
    }
    /// Read unchanged outer publication phase; `diagnostic.phase()` is not an inferred leaf phase.
    pub(crate) fn phase(&self) -> PrismPublicationPhase {
        // Retained child phases remain separately accessible in the typed cause.
        self.phase
    }
    /// Read original operation classification; `diagnostic.class()` is independent of projection coverage.
    pub(crate) fn class(&self) -> PrismPublicationClass {
        // Missing projection evidence never rewrites operation classification.
        self.class
    }
    /// Borrow exact supported cause evidence; `diagnostic.cause()` reports unsupported families explicitly.
    pub(crate) fn cause(&self) -> &OwnedPublicationDiagnosticCause {
        // Only private construction can pair a cause with this context.
        &self.cause
    }
    /// Read all known projection gaps; `diagnostic.gaps()` never describes body validity.
    pub(crate) fn gaps(&self) -> &[PublicationDiagnosticGap] {
        // Order follows the retained cause tree, not hash iteration.
        &self.gaps
    }
    /// Read diagnostic coverage; `diagnostic.coverage()` grants no operation completion.
    pub(crate) fn coverage(&self) -> DiagnosticProjectionCoverage {
        if self.gaps.is_empty() {
            return DiagnosticProjectionCoverage::Complete;
        }
        DiagnosticProjectionCoverage::Incomplete
    }
}
impl<'a> InvokedPublicationRefusalProjection<'a> {
    /// Borrow immutable evidence from either outcome; `projection.diagnostic()`.
    pub(crate) fn diagnostic(&self) -> &OwnedPrismPublicationDiagnostic {
        // Local originals remain private to the consuming inspection path.
        &self.diagnostic
    }
    /// Recover the same original Box when projection is incomplete; `projection.into_parts()`.
    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<OwnedPrismPublicationDiagnostic>,
        Option<Box<PrismPublicationFailure<'a>>>,
    ) {
        let Self {
            diagnostic,
            original,
        } = self;
        (diagnostic, original)
    }
}
