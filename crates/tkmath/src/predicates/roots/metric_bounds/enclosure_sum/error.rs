// SPDX-License-Identifier: GPL-2.0-only
use super::{EnclosureSumOperation, EnclosureSumOptions};
/// Typed refusal retaining requested policy; no partial successful sum is returned.
/// Example: `let operation = error.operation;`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnclosureSumError {
    /// Arithmetic operation that could not complete.
    pub operation: EnclosureSumOperation,
    /// Original policy, including invalid values.
    pub requested_options: EnclosureSumOptions,
    /// Exact stage and offending supplied value or bound.
    pub cause: EnclosureSumCause,
}
/// Reasons a finite conditional enclosure sum is incomplete.
/// Example: `matches!(error.cause, EnclosureSumCause::Cancelled)`.
#[derive(Clone, Debug, PartialEq)]
pub enum EnclosureSumCause {
    /// Version must be one, count cap at most 64, and error cap finite/nonnegative.
    Policy(EnclosureSumOptions),
    /// Input count exceeds the unchanged requested maximum.
    ComponentCount {
        /// Supplied number of components.
        actual: usize,
        /// Unchanged requested upper count.
        maximum: u16,
    },
    /// Component must have finite ordered endpoints with lower at least zero.
    Component {
        /// Position in the caller's ordered input.
        index: usize,
        /// Offending endpoint pair, retained without repair.
        supplied: [f64; 2],
    },
    /// Exact nonnegative endpoint or width cannot be enclosed by finite binary64.
    Unrepresentable {
        /// Endpoint or width that must be finite.
        quantity: &'static str,
        /// Exact rational value that exceeded binary64 range.
        exact_value: String,
    },
    /// Complete outward width exceeds the original finite cap.
    ErrorLimit {
        /// Original finite nonnegative full-width cap.
        requested: f64,
        /// Upward-rounded width including endpoint rounding.
        achieved: f64,
    },
    /// Caller requested cancellation; subsequent reuse starts from empty local state.
    Cancelled,
}

impl std::fmt::Display for EnclosureSumError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:?}: {:?}; expected finite ordered nonnegative components and width <= {:?}",
            self.operation, self.cause, self.requested_options.absolute_error_limit
        )
    }
}
impl std::error::Error for EnclosureSumError {}
