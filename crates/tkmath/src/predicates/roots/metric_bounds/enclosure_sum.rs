// SPDX-License-Identifier: GPL-2.0-only
//! ADR0031: exact endpoint addition, conditional on the supplied component enclosures.
mod arithmetic;
mod error;
mod types;
use super::super::certificate_arithmetic::{ExactRatio, rounding::outward_binary64};
pub use error::*;
pub use types::*;

/// Sum nonnegative enclosures without certifying their originating computations.
/// Empty input completes with zero; each finite endpoint denotes its exact binary64 value.
/// Example: `sum_nonnegative_enclosures(&[[1., 2.]], Default::default(), || false)?`.
pub fn sum_nonnegative_enclosures(
    components: &[[f64; 2]],
    options: EnclosureSumOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CertifiedEnclosureSum, EnclosureSumError> {
    let result = arithmetic::sum(components, options, &mut cancelled);
    result.map_err(|cause| EnclosureSumError {
        operation: EnclosureSumOperation::NonnegativeBounds,
        requested_options: options,
        cause,
    })
}
fn checkpoint(cancelled: &mut impl FnMut() -> bool) -> Result<(), EnclosureSumCause> {
    if cancelled() {
        return Err(EnclosureSumCause::Cancelled);
    }
    Ok(())
}
#[cfg(test)]
mod fixture_tests;
#[cfg(test)]
mod tests;
