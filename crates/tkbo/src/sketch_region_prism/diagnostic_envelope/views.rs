// SPDX-License-Identifier: GPL-2.0-only
//! Lock-free reads borrow immutable diagnostic values; unsupported local owners stay separate.
use super::*;
impl OwnedPrismDiagnostic {
    /// Diagnostic behavior version only; `diagnostic.version()` grants no body completion.
    pub(crate) fn version(&self) -> u32 {
        // The originating operation and each requested policy retain separate versions.
        1
    }
    /// Read exact original request provenance; `diagnostic.context().request().operation`.
    pub(crate) fn context(&self) -> &OwnedPrismDiagnosticContext {
        // Context is captured from the actual failure, never reconstructed from internal tokens.
        &self.context
    }
    /// Read the outer preparation boundary; child phase remains in callback or retained local cause; `diagnostic.phase()`.
    pub(crate) fn phase(&self) -> PrismDiagnosticPhase {
        // Diagnostic projection cannot advance an operation phase.
        self.phase
    }
    /// Read unchanged original operation classification; `diagnostic.class()` is separate from coverage.
    pub(crate) fn class(&self) -> PrismDiagnosticClass {
        // Missing projection coverage never rewrites failed/cancelled/declined operation outcomes.
        self.class
    }
    /// Read the exact supported cause or explicit missing payload kind; `diagnostic.cause()`.
    pub(crate) fn cause(&self) -> &OwnedPreparationDiagnosticCause {
        // Unsupported nested owners remain in the local incomplete carrier.
        &self.cause
    }
    /// Read initiating callback phase, ordinal and exact native failure; `diagnostic.callback()`.
    pub(crate) fn callback(&self) -> Option<&OwnedPreparationCallback> {
        // Preserve callback precedence without discarding a distinct child cause.
        self.callback.as_ref()
    }
    /// Read diagnostic projection coverage, never body validity; `diagnostic.coverage()`.
    pub(crate) fn coverage(&self) -> DiagnosticProjectionCoverage {
        if self.gaps.is_empty() {
            return DiagnosticProjectionCoverage::Complete;
        }
        DiagnosticProjectionCoverage::Incomplete
    }
    /// Read all unprojected obligations in deterministic order; `diagnostic.gaps()`.
    pub(crate) fn gaps(&self) -> &[PrismDiagnosticGap] {
        // Cause-owned and outer-owner gaps are independently recorded.
        &self.gaps
    }
}
impl OwnedPrismDiagnosticContext {
    /// Borrow all original input/policy operands; `context.request().sources[0].revision`.
    pub(crate) fn request(&self) -> &OwnedPolynomialPrismRequest {
        // Owned vectors and scalar planes cannot alias a mutable caller geometry object.
        &self.request
    }
    /// Borrow the actual validated plan, absent before admission; `context.plan()`.
    pub(crate) fn plan(&self) -> Option<&SingleBoundaryNamePlan> {
        // Its own version and dictionary bytes are retained without an assumed naming version.
        self.plan.as_ref()
    }
    /// Replay complete original operands and actual dictionary; `context.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        // This is context evidence, not a complete operation or diagnostic-cause serialization.
        &self.replay
    }
}
impl PreparationRefusalProjection {
    /// Borrow the owned evidence from either coverage outcome; `projection.diagnostic()`.
    pub(crate) fn diagnostic(&self) -> &OwnedPrismDiagnostic {
        match self {
            Self::Projected(value) => value,
            Self::Incomplete { diagnostic, .. } => diagnostic,
        }
    }
    /// Recover original unsupported owners for inspection; `projection.into_parts()`.
    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<OwnedPrismDiagnostic>,
        Option<Box<OwnedPrismPreparationFailure>>,
    ) {
        match self {
            Self::Projected(value) => (value, None),
            Self::Incomplete {
                diagnostic,
                original,
            } => (diagnostic, Some(original)),
        }
    }
}
