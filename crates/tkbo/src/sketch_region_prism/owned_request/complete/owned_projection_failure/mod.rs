// SPDX-License-Identifier: GPL-2.0-only
//! ADR0056: move one actual post-publication refusal; other scoped families remain untouched.
mod classification;
mod replay;
mod views;
pub(crate) use super::projection::CompleteProjectionCause;
use super::{projection::ProjectionRefusal, *};
pub(crate) use classification::*;

/// Retains the exact failing child and accepted publication as context, never outer success.
pub(crate) struct OwnedProjectionRefusal {
    original: Box<ProjectionRefusal>,
    callback: Option<CompletePrismCallbackStop>,
}

/// Move the actual projection branch; `own_projection_refusal(failure)` returns other branches intact.
pub(crate) fn own_projection_refusal(
    failure: Box<ScopedPrismFailure<'_>>,
) -> Result<OwnedProjectionRefusal, Box<ScopedPrismFailure<'_>>> {
    if !matches!(&failure.cause, ScopedPrismCause::Projection(_)) {
        return Err(failure);
    }
    let ScopedPrismFailure { callback, cause } = *failure;
    match cause {
        ScopedPrismCause::Projection(original) => Ok(OwnedProjectionRefusal { original, callback }),
        _ => unreachable!("projection branch was checked before consuming its owner"),
    }
}

#[cfg(test)]
mod payload_tests;
#[cfg(test)]
mod tests;
