// SPDX-License-Identifier: GPL-2.0-only
use super::PolynomialSideFailure;

/// Finite admission and integration policy; excess limits are refused, never clamped.
/// Example: `PolynomialSideOptions { absolute_error_limit: 1e-5, ..Default::default() }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolynomialSideOptions {
    /// The supported additive scalar behavior version is 1.
    pub behavior_version: u32,
    /// Maximum stored polynomial degree, between zero and 16 inclusive.
    pub max_degree: u16,
    /// Maximum completed dyadic subdivisions, between zero and 65536 inclusive.
    pub max_subdivisions: u32,
    /// Finite nonnegative absolute error cap in squared world-coordinate units.
    pub absolute_error_limit: f64,
}
impl Default for PolynomialSideOptions {
    fn default() -> Self {
        Self {
            behavior_version: 1,
            max_degree: 16,
            max_subdivisions: 4096,
            absolute_error_limit: 1e-5,
        }
    }
}

/// Completed work, retained on failure and cancellation as well as success.
/// Example: `assert_eq!(PolynomialSideWork::default().subdivisions, 0);`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PolynomialSideWork {
    /// Complete parent restrictions with both child bounds computed.
    pub subdivisions: u32,
    /// Fully computed interval bounds, including a child before a later cancellation.
    pub interval_bounds: u32,
}

/// Arithmetic classification of the conditional scalar enclosure.
/// Example: `assert_eq!(report.evidence(), PolynomialSideEvidence::CertifiedBound);`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolynomialSideEvidence {
    /// Exact polynomial restriction, convex norm inequalities and outward endpoint rounding.
    CertifiedBound,
}

/// Immutable certified scalar integral, conditional on the caller's supplied polynomial.
/// This integrates parameter multiplicity, not the area of a validated face or body.
/// Example: `assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);`.
#[derive(Clone, Debug, PartialEq)]
pub struct CertifiedPolynomialSideIntegral {
    pub(super) enclosure: [f64; 2],
    pub(super) error: f64,
    pub(super) options: PolynomialSideOptions,
    pub(super) work: PolynomialSideWork,
}
impl CertifiedPolynomialSideIntegral {
    /// Read the outward certified interval; `let [lower, upper] = report.enclosure();`.
    pub fn enclosure(&self) -> [f64; 2] {
        // Endpoints already include final outward rounding.
        self.enclosure
    }
    /// Read the deterministic lower endpoint; `let integral = report.representative();`.
    pub fn representative(&self) -> f64 {
        // The representative is pinned to the lower endpoint for replay.
        self.enclosure[0]
    }
    /// Read the upward-rounded error from the representative, including all rounding.
    /// Example: `let certified_area_error = report.absolute_error_bound();`.
    pub fn absolute_error_bound(&self) -> f64 {
        // The bound includes both integration and endpoint rounding.
        self.error
    }
    /// Read original limits; `let requested = report.requested_options();`.
    pub fn requested_options(&self) -> PolynomialSideOptions {
        // Admission never clamps or substitutes the requested policy.
        self.options
    }
    /// Read applied limits, equal to requested limits; `let applied = report.effective_options();`.
    pub fn effective_options(&self) -> PolynomialSideOptions {
        // Admission never clamps or substitutes the requested policy.
        self.options
    }
    /// Identify the proved enclosure method; `let proof_kind = report.evidence();`.
    pub fn evidence(&self) -> PolynomialSideEvidence {
        // This API never promotes a quadrature estimate into a certified report.
        PolynomialSideEvidence::CertifiedBound
    }
    /// Read completed work; `let splits = report.work().subdivisions;`.
    pub fn work(&self) -> PolynomialSideWork {
        // Completed work belongs to this operation, independent of later reuse.
        self.work
    }
}

/// No completed scalar was published; the optional enclosure is diagnostic only.
/// Caller-owned source identities must be attached before any face-level interpretation.
/// Example: `let spent = error.work.subdivisions;`.
#[derive(Debug)]
pub struct PolynomialSideError {
    /// Requested version and limits, even if admission failed.
    pub requested_options: PolynomialSideOptions,
    /// Applied limits, absent until policy admission succeeds.
    pub effective_options: Option<PolynomialSideOptions>,
    /// Completed work before the terminal cause.
    pub work: PolynomialSideWork,
    /// Last finite certified enclosure, without completed-operation status.
    pub diagnostic_enclosure: Option<[f64; 2]>,
    /// Typed cancellation, input, work or representability failure.
    pub cause: Box<PolynomialSideFailure>,
}
