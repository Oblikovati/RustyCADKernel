// SPDX-License-Identifier: GPL-2.0-only
use super::super::PolynomialIntegralOperation;

/// Version 1 Taylor order of every subinterval approximant; fixed so replay is identical.
pub const RATIONAL_TAYLOR_ORDER: usize = 16;

/// Admission, accuracy and work policy; excess limits are refused without clamping.
/// Example: `RationalIntegralOptions { absolute_error_limit: 1e-10, ..Default::default() }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RationalIntegralOptions {
    /// Supported behavior version is 1; the polynomial integral entries are unchanged.
    pub behavior_version: u32,
    /// Maximum stored degree per curve; allowed range is 0 through 16.
    pub max_degree: u16,
    /// Maximum ordered occurrences; allowed range is 1 through 64.
    pub max_curves: u16,
    /// Finite nonnegative absolute error cap in the requested integral's units.
    pub absolute_error_limit: f64,
    /// Maximum examined parameter subintervals per curve; allowed range 1 through 65536.
    pub max_subintervals: u32,
}
impl Default for RationalIntegralOptions {
    fn default() -> Self {
        Self {
            behavior_version: 1,
            max_degree: 16,
            max_curves: 64,
            absolute_error_limit: 1e-12,
            max_subintervals: 4096,
        }
    }
}

/// How the published enclosure was obtained; neither variant validates a region or body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RationalIntegralEvidence {
    /// Every curve had a constant denominator: exact rational value, outward rounding only.
    ExactRational,
    /// At least one curve used the certified Taylor-model enclosure of ADR 0066.
    ErrorBoundedIntegration,
}

/// Immutable completed scalar integration with a certified absolute error bound.
/// Interpreting it as area or volume requires the same separate coverage, orientation
/// and injectivity proofs as the polynomial report; this report supplies none of them.
#[derive(Clone, Debug, PartialEq)]
pub struct CertifiedRationalMeasure {
    pub(super) enclosure: [f64; 2],
    pub(super) representative: f64,
    pub(super) absolute_error_bound: f64,
    pub(super) options: RationalIntegralOptions,
    pub(super) operation: PolynomialIntegralOperation,
    pub(super) evidence: RationalIntegralEvidence,
    pub(super) subintervals: u64,
}
impl CertifiedRationalMeasure {
    /// Read the binary64 enclosure that contains the exact (possibly irrational) scalar.
    /// Example: `let [lower, upper] = report.enclosure();`.
    pub fn enclosure(&self) -> [f64; 2] {
        self.enclosure
    }
    /// Read the deterministic representative: the lower rounding of the enclosure midpoint.
    /// Example: `assert!(report.enclosure()[0] <= report.representative());`.
    pub fn representative(&self) -> f64 {
        self.representative
    }
    /// Read the upward-rounded bound on `abs(exact - representative)`.
    /// Example: `assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);`.
    pub fn absolute_error_bound(&self) -> f64 {
        self.absolute_error_bound
    }
    /// Read the original policy; no tolerance or work bound is widened.
    /// Example: `assert_eq!(report.requested_options().behavior_version, 1);`.
    pub fn requested_options(&self) -> RationalIntegralOptions {
        self.options
    }
    /// Read the effective policy, identical to the requested one because nothing is clamped.
    /// Example: `assert_eq!(report.effective_options(), report.requested_options());`.
    pub fn effective_options(&self) -> RationalIntegralOptions {
        self.options
    }
    /// Identify the conditional scalar quantity actually integrated.
    /// Example: `let quantity = report.operation();`.
    pub fn operation(&self) -> PolynomialIntegralOperation {
        self.operation
    }
    /// Identify whether the value is exact or bounded by certified integration.
    /// Example: `let evidence = report.evidence();`.
    pub fn evidence(&self) -> RationalIntegralEvidence {
        self.evidence
    }
    /// Read the measured work: accepted plus refined subintervals over all curves.
    /// Example: `assert!(report.integration_subintervals() >= 1);`.
    pub fn integration_subintervals(&self) -> u64 {
        self.subintervals
    }
}
