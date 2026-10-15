// SPDX-License-Identifier: GPL-2.0-only
//! Owned annular publication values: the actual typed child failures, never a count or a string.
use super::*;

/// One moved child failure with its exact explicit-tag bytes; `child.failure().cause`.
pub(in crate::sketch_region_prism) struct OwnedAnnularChildRefusal<T> {
    pub(super) failure: Box<T>,
    pub(super) encoded: Vec<u8>,
}
impl<T> OwnedAnnularChildRefusal<T> {
    /// Borrow the actual moved child failure; `child.failure()` keeps every typed operand.
    pub(in crate::sketch_region_prism) fn failure(&self) -> &T {
        &self.failure
    }
    /// Borrow the child's explicit-tag bytes; `child.encoded()` has no Debug fallback.
    pub(in crate::sketch_region_prism) fn encoded(&self) -> &[u8] {
        &self.encoded
    }
}
pub(in crate::sketch_region_prism) enum OwnedAnnularPublicationCause {
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    Join(Box<OwnedAnnularJoinRefusal>),
    Validation(Box<OwnedAnnularChildRefusal<AnnularValidationFailure>>),
    History(Box<OwnedPublicationHistory>),
    HistoryCancelled {
        operation: AuthoredPrismId,
        phase: PrismHistoryPhase,
    },
    Measures(Box<OwnedAnnularChildRefusal<AnnularMeasureFailure>>),
    Binding {
        proof: &'static str,
    },
    /// The interrupting callback keeps the child it interrupted, as in the producer.
    Callback {
        stop: OwnedPublicationCallback,
        downstream: Option<Box<Self>>,
    },
    Unprojected(PublicationPendingPayload),
}
/// Immutable refusal: original operation, policies, phase and class, plus explicit gaps.
pub(in crate::sketch_region_prism) struct OwnedAnnularPublicationDiagnostic {
    pub(super) operation: AuthoredPrismId,
    pub(super) requested: AnnularPublicationOptions,
    pub(super) phase: PrismPublicationPhase,
    pub(super) class: PrismPublicationClass,
    pub(super) cause: OwnedAnnularPublicationCause,
    pub(super) gaps: Vec<PublicationDiagnosticGap>,
}
/// Only the diagnostic is publishable; `original` is present exactly when a gap remains.
pub(in crate::sketch_region_prism) struct AnnularPublicationRefusalProjection<'a> {
    pub(super) diagnostic: Box<OwnedAnnularPublicationDiagnostic>,
    pub(super) original: Option<Box<AnnularPublicationFailure<'a>>>,
}
impl<'a> AnnularPublicationRefusalProjection<'a> {
    /// Read the owned diagnostic; `projection.diagnostic().gaps().is_empty()`.
    pub(in crate::sketch_region_prism) fn diagnostic(&self) -> &OwnedAnnularPublicationDiagnostic {
        &self.diagnostic
    }
    /// Read the retained original when coverage is incomplete; `projection.original()`.
    pub(in crate::sketch_region_prism) fn original(
        &self,
    ) -> Option<&AnnularPublicationFailure<'a>> {
        self.original.as_deref()
    }
    /// Take the owned diagnostic, dropping any operation-local original.
    pub(in crate::sketch_region_prism) fn into_diagnostic(
        self,
    ) -> Box<OwnedAnnularPublicationDiagnostic> {
        self.diagnostic
    }
}
impl OwnedAnnularPublicationDiagnostic {
    /// Read the internal operation token; `diagnostic.operation()`.
    pub(in crate::sketch_region_prism) fn operation(&self) -> AuthoredPrismId {
        self.operation
    }
    /// Read every original child policy; `diagnostic.requested().history`.
    pub(in crate::sketch_region_prism) fn requested(&self) -> AnnularPublicationOptions {
        self.requested
    }
    /// Read the unchanged outer phase; `diagnostic.phase()`.
    pub(in crate::sketch_region_prism) fn phase(&self) -> PrismPublicationPhase {
        self.phase
    }
    /// Read the unchanged outer class; `diagnostic.class()` is separate from coverage.
    pub(in crate::sketch_region_prism) fn class(&self) -> PrismPublicationClass {
        self.class
    }
    /// Read the owned typed cause tree; `diagnostic.cause()`.
    pub(in crate::sketch_region_prism) fn cause(&self) -> &OwnedAnnularPublicationCause {
        &self.cause
    }
    /// Read ordered explicit gaps; empty means the actual refusal is owned losslessly.
    pub(in crate::sketch_region_prism) fn gaps(&self) -> &[PublicationDiagnosticGap] {
        &self.gaps
    }
}
