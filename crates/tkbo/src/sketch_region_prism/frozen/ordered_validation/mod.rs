// SPDX-License-Identifier: GPL-2.0-only
//! ADR0039: one source-bound topology → geometry → intersection → tolerance operation.
mod stages;
pub(crate) use stages::{OrderedPrismStage, OrderedPrismStageStatus, OrderedPrismStageTrace};
#[cfg(test)]
mod joined_baseline_tests;
#[cfg(test)]
mod stage_tests;

use super::{joined_orientation, proof_join::JoinedPrismProof, topology};
mod adapters;
mod certificate;
mod checks;
mod classification;
mod errors;
mod execution;
mod options;
mod progress;
mod work;
pub(crate) use certificate::*;
pub(crate) use errors::{
    OrderedPrismCause, OrderedPrismFailureClass, OrderedPrismValidationFailure,
};
pub(crate) use execution::validate_joined_prism;
pub(crate) use options::*;
pub(crate) use progress::*;

#[cfg(test)]
mod cancellation_tests;
#[cfg(test)]
mod operation_tests;
#[cfg(test)]
mod test_support;

mod intersection;
pub(crate) use intersection::JoinedPrismIntersectionCertificate;

#[cfg(test)]
mod isolation_tests;
mod replay;
#[cfg(test)]
mod replay_tests;

#[cfg(test)]
mod classification_tests;
#[cfg(test)]
mod policy_tests;
