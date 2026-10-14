// SPDX-License-Identifier: GPL-2.0-only
use super::PolynomialIntegralOptions;

/// Exact rational squared measure followed by a certified outward square-root enclosure.
/// This is algebraic evidence, distinct from rational integral rounding evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolynomialPlaneEvidence {
    /// Exact native integral squared times exact placed Gram determinant, then directed sqrt.
    CertifiedAlgebraicBound,
}
/// Immutable conditional unsigned world-plane scalar; no topology or body validation.
/// Unlike `CertifiedPolynomialMeasure`, the exact result may be irrational. The error
/// bounds the lower endpoint by the full outward enclosure width, without an estimated sqrt.
#[derive(Clone, Debug, PartialEq)]
pub struct CertifiedPolynomialPlaneMeasure {
    pub(super) enclosure: [f64; 2],
    pub(super) absolute_error_bound: f64,
    pub(super) options: PolynomialIntegralOptions,
}
impl CertifiedPolynomialPlaneMeasure {
    /// Read the adjacent enclosing world-area endpoints, including underflow.
    /// Example: `let [lower, upper] = report.enclosure();`.
    pub fn enclosure(&self) -> [f64; 2] {
        // Value copies prevent readers from changing certified endpoints.
        self.enclosure
    }
    /// Read the deterministic lower endpoint; this never overstates the exact scalar.
    /// Example: `assert_eq!(report.representative(), report.enclosure()[0]);`.
    pub fn representative(&self) -> f64 {
        // Choosing the lower endpoint makes scalar replay deterministic.
        self.enclosure[0]
    }
    /// Read the upward-rounded enclosure width in world coordinates squared.
    /// Example: `assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);`.
    pub fn absolute_error_bound(&self) -> f64 {
        // The retained width bounds error even when the exact square root is irrational.
        self.absolute_error_bound
    }
    /// Read the original admission and world-area error policy.
    /// Example: `assert_eq!(report.requested_options().behavior_version, 1);`.
    pub fn requested_options(&self) -> PolynomialIntegralOptions {
        // The policy snapshot belongs to this completed arithmetic request.
        self.options
    }
    /// Read the effective policy; bounds and tolerance are never clamped or widened.
    /// Example: `assert_eq!(report.effective_options(), report.requested_options());`.
    pub fn effective_options(&self) -> PolynomialIntegralOptions {
        // Admission refuses excess limits, so no hidden clamp needs reconstruction.
        self.options
    }
    /// Identify algebraic enclosure evidence without implying geometric validity.
    /// Example: `let evidence = report.evidence();`.
    pub fn evidence(&self) -> PolynomialPlaneEvidence {
        // The algebraic tag keeps this report distinct from rational-only integration.
        PolynomialPlaneEvidence::CertifiedAlgebraicBound
    }
}
