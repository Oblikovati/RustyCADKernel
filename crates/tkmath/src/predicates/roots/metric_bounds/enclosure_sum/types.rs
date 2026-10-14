// SPDX-License-Identifier: GPL-2.0-only
/// Finite endpoint-sum policy; no component budget is allocated or changed.
/// Example: `EnclosureSumOptions { absolute_error_limit: 1e-4, ..Default::default() }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnclosureSumOptions {
    /// Supported additive behavior version, currently one.
    pub behavior_version: u32,
    /// Maximum input component count, at most 64; zero admits only an empty sum.
    pub max_components: u16,
    /// Finite nonnegative full-width error cap, in the components' common units.
    pub absolute_error_limit: f64,
}
impl Default for EnclosureSumOptions {
    fn default() -> Self {
        Self {
            behavior_version: 1,
            max_components: 64,
            absolute_error_limit: 1e-4,
        }
    }
}
/// Typed identity of the conditional arithmetic operation.
/// Example: `let operation = EnclosureSumOperation::NonnegativeBounds;`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnclosureSumOperation {
    /// Exact lower/upper addition and final outward rounding; input claims are external.
    NonnegativeBounds,
}
/// Arithmetic evidence scope; component geometric claims remain external.
/// Example: `let evidence = EnclosureSumEvidence::CertifiedConditionalEndpointArithmetic;`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnclosureSumEvidence {
    /// Exact endpoint sums with outward rounding, conditional on supplied enclosures.
    CertifiedConditionalEndpointArithmetic,
}
/// Immutable owned sum; component intervals retain their supplied bits and order.
/// This certifies endpoint arithmetic only, not that components enclose physical quantities.
/// Example: `assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);`.
#[derive(Clone, Debug, PartialEq)]
pub struct CertifiedEnclosureSum {
    pub(super) components: Vec<[f64; 2]>,
    pub(super) enclosure: [f64; 2],
    pub(super) error: f64,
    pub(super) options: EnclosureSumOptions,
}
impl CertifiedEnclosureSum {
    /// Identify the proved conditional arithmetic; `let evidence = report.evidence();`.
    pub fn evidence(&self) -> EnclosureSumEvidence {
        // This tag never certifies the inputs' physical or geometric interpretation.
        EnclosureSumEvidence::CertifiedConditionalEndpointArithmetic
    }

    /// Read unchanged component endpoints; `let supplied = report.components();`.
    pub fn components(&self) -> &[[f64; 2]] {
        // Borrowing this owned slice cannot mutate the report or original input.
        &self.components
    }
    /// Read the certified sum enclosure; `let [lower, upper] = report.enclosure();`.
    pub fn enclosure(&self) -> [f64; 2] {
        // Both endpoint roundings are included in the published width.
        self.enclosure
    }
    /// Read the deterministic lower representative; `let value = report.representative();`.
    pub fn representative(&self) -> f64 {
        // A fixed endpoint avoids an additional unreported midpoint rounding.
        self.enclosure[0]
    }
    /// Read the upward-rounded full width; `let error = report.absolute_error_bound();`.
    pub fn absolute_error_bound(&self) -> f64 {
        // The width includes child uncertainty and final sum rounding.
        self.error
    }
    /// Read original limits; `let requested = report.requested_options();`.
    pub fn requested_options(&self) -> EnclosureSumOptions {
        // Policy admission never substitutes a child or total budget.
        self.options
    }
    /// Read unchanged applied limits; `let effective = report.effective_options();`.
    pub fn effective_options(&self) -> EnclosureSumOptions {
        // There are no implicit clamps or adaptive retries in this operation.
        self.options
    }
    /// Identify conditional endpoint arithmetic; `let kind = report.operation();`.
    pub fn operation(&self) -> EnclosureSumOperation {
        // Component provenance remains the caller's separate proof obligation.
        EnclosureSumOperation::NonnegativeBounds
    }
}
