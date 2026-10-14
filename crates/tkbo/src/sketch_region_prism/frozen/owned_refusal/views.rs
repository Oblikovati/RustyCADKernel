// SPDX-License-Identifier: GPL-2.0-only
//! Read-only projected evidence has no access to live operation owners.
use super::*;
impl OwnedPrismJoinRefusal {
    /// Behavior version of the projection, separate from the child failure; `refusal.version()`.
    pub(crate) fn version(&self) -> u32 {
        // No legacy algorithm behavior or class changes with this projection.
        1
    }
    /// Original operation identity; `refusal.operation()`.
    pub(crate) fn operation(&self) -> AuthoredPrismId {
        // Caller dictionary binding remains attached at the operation envelope.
        self.operation
    }
    /// Original join phase; `refusal.phase()`.
    pub(crate) fn phase(&self) -> PrismProofJoinPhase {
        // Preserve the originating stage without inferring it from the nested cause.
        self.phase
    }
    /// The caller's unchanged operation class; `refusal.class()`.
    pub(crate) fn class(&self) -> PrismRefusalClass {
        // Projection completeness must not reinterpret the operation classification.
        self.class
    }
    /// Existing affected authored entity when present; `refusal.entity()`.
    pub(crate) fn entity(&self) -> Option<FrozenPrismKey> {
        // Return the full authored key, including its role and operation.
        self.entity
    }
    /// Borrow exact typed failure payloads; `refusal.cause()`.
    pub(crate) fn cause(&self) -> &OwnedPrismJoinCause {
        // Borrowing the immutable cause cannot mutate its retained evidence.
        &self.cause
    }
}
impl LocalPrismProjectionFailure {
    /// Borrow why projection is incomplete; `failure.issue()` does not replace the original cause.
    pub(crate) fn issue(&self) -> &PrismRefusalProjectionIssue {
        // The original local payload remains available separately.
        &self.issue
    }
    /// Recover the complete original local refusal with unchanged class; `failure.into_original()`.
    pub(crate) fn into_original(self) -> (PrismRefusalClass, PrismProofJoinFailure) {
        let class = self.class;
        (class, self.original)
    }
}
impl PrismRefusalProjectionIssue {
    /// Borrow the typed route to the unprojected payload; `issue.path()`.
    pub(crate) fn path(&self) -> &[PrismRefusalPath] {
        // Preserve nesting so the missing adapter is actionable.
        &self.path
    }
    /// Borrow the precise missing projection obligation; `issue.cause()`.
    pub(crate) fn cause(&self) -> &PrismRefusalProjectionGap {
        // Borrowing the immutable cause cannot mutate its retained evidence.
        &self.cause
    }
}
