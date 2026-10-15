// SPDX-License-Identifier: GPL-2.0-only
use super::super::MetricCurve;

/// Explicit occurrence traversal; `reversed` negates the exact full-range integral.
/// Example: `OrientedMetricCurve { curve, reversed: true }` traverses native 1 to 0.
/// This carries no source identity, boundary coverage or orientation validity proof.
#[derive(Clone, Copy)]
pub struct OrientedMetricCurve<'a> {
    /// Stored polynomial Bézier or affine line, always evaluated on all of `[0,1]`.
    pub curve: MetricCurve<'a, 2>,
    /// Reverse traversal independently of stored poles and coordinate values.
    pub reversed: bool,
}

/// Admission and rounding policy; excess limits are refused without clamping.
/// Example: `PolynomialIntegralOptions { absolute_error_limit: 1e-10, ..Default::default() }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolynomialIntegralOptions {
    /// Supported behavior version is 1; no existing integration behavior is replaced.
    pub behavior_version: u32,
    /// Maximum stored degree per curve; allowed range is 0 through 16.
    pub max_degree: u16,
    /// Maximum ordered occurrences; allowed range is 1 through 64.
    pub max_curves: u16,
    /// Finite nonnegative absolute error cap in the requested integral's units.
    pub absolute_error_limit: f64,
}
impl Default for PolynomialIntegralOptions {
    fn default() -> Self {
        Self {
            behavior_version: 1,
            max_degree: 16,
            max_curves: 64,
            absolute_error_limit: 1e-12,
        }
    }
}

/// Scalar quantity only; neither variant asserts an accepted region or valid body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolynomialIntegralOperation {
    /// One half of the ordered sum of full-range integrals of x dy - y dx.
    SignedBoundaryIntegral,
    /// Signed boundary integral times the supplied affine sweep's exact determinant.
    SignedSweepIntegral,
    /// ADR 0066 part C: outer loop area minus hole areas, under caller-certified Jordan nesting.
    FaceArea,
}

/// Certification describes arithmetic evidence, not boundary or body validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolynomialIntegralEvidence {
    /// Exact rational integration followed only by outward binary64 rounding.
    CertifiedBound,
}

/// Immutable completed scalar integration with a certified absolute rounding bound.
/// An open curve still has a complete integral. Interpreting it as area requires proven
/// oriented boundary coverage; interpreting the sweep integral as volume additionally
/// requires proven support and injectivity. This report supplies none of those proofs.
/// No measured geometric approximation error, topology or history is implied.
#[derive(Clone, Debug, PartialEq)]
pub struct CertifiedPolynomialMeasure {
    pub(super) enclosure: [f64; 2],
    pub(super) absolute_error_bound: f64,
    pub(super) options: PolynomialIntegralOptions,
    pub(super) operation: PolynomialIntegralOperation,
}
impl CertifiedPolynomialMeasure {
    /// Read the adjacent-binary64 enclosure of the exact scalar, including underflow.
    /// Example: `let [lower, upper] = report.enclosure();`.
    pub fn enclosure(&self) -> [f64; 2] {
        self.enclosure
    }
    /// Read the deterministic lower-enclosure endpoint used as the representative value.
    /// Example: `assert_eq!(report.representative(), report.enclosure()[0]);`.
    pub fn representative(&self) -> f64 {
        self.enclosure[0]
    }
    /// Read the upward-rounded absolute error bound for the representative value.
    /// Example: `assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);`.
    pub fn absolute_error_bound(&self) -> f64 {
        self.absolute_error_bound
    }
    /// Read the original policy; no tolerances or work bounds are widened.
    /// Example: `assert_eq!(report.requested_options().behavior_version, 1);`.
    pub fn requested_options(&self) -> PolynomialIntegralOptions {
        self.options
    }
    /// Read the effective policy, identical to the requested policy because no clamps occur.
    /// Example: `assert_eq!(report.effective_options(), report.requested_options());`.
    pub fn effective_options(&self) -> PolynomialIntegralOptions {
        self.options
    }
    /// Identify the conditional scalar quantity actually integrated.
    /// Example: `let quantity = report.operation();`.
    pub fn operation(&self) -> PolynomialIntegralOperation {
        self.operation
    }
    /// Identify the error classification; no quadrature estimate is used.
    /// Example: `let evidence = report.evidence();`.
    pub fn evidence(&self) -> PolynomialIntegralEvidence {
        PolynomialIntegralEvidence::CertifiedBound
    }
}
