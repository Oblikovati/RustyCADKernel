// SPDX-License-Identifier: GPL-2.0-only
//! ADR0053 private scoped composition; borrowed refusal projection remains a public-interface gap.
mod admission;
mod context;
mod operation;
mod projection;
mod replay;
mod result;
mod scoped_failure;
pub(crate) use context::*;
pub(crate) use operation::with_owned_prism_operation;
pub(crate) use result::OwnedCompletePrism;
pub(crate) use scoped_failure::{ScopedPrismCause, ScopedPrismFailure};

#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod cancellation_tests;
#[cfg(test)]
mod degenerate_tests;
#[cfg(test)]
mod isolation_tests;
#[cfg(test)]
mod operation_tests;
#[cfg(test)]
mod policy_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod runtime_tests;
