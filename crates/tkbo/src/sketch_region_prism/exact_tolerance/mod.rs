// SPDX-License-Identifier: GPL-2.0-only
//! ADR0036 exact represented-geometry evidence; no ordered validation promotion.
mod closure_scalar;
#[cfg(test)]
mod closure_scalar_tests;
mod endpoint_uses;
mod failure_class;
#[cfg(test)]
mod frame_tests;
mod frames;
mod joined_endpoints;
mod joined_frames;
mod joined_trims;
pub(crate) use failure_class::JoinedEvidenceClass;
mod joined_progress;
mod joined_types;
#[cfg(test)]
mod quantity_tests;
mod scalar;
pub(crate) use joined_types::*;
#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
mod test_support;

mod conformance;
mod joined_closures;
pub(crate) use conformance::certify_joined_prism_conformance;
#[cfg(test)]
mod joined_tests;

mod stored_metadata;
mod tolerance_types;
mod tolerances;
pub(crate) use super::tolerance_inspection::StoredToleranceCap;
pub(crate) use tolerance_types::*;
pub(crate) use tolerances::certify_joined_prism_tolerances;

#[cfg(test)]
mod joined_cancel_tests;
#[cfg(test)]
mod joined_test_support;

#[cfg(test)]
mod joined_metadata_tests;
#[cfg(test)]
mod joined_operand_tests;

mod replay;
pub(crate) use replay::encode_stored_tolerance_observation;
mod replay_limits;

#[cfg(test)]
mod joined_replay_tests;

#[cfg(test)]
mod joined_failure_tests;
