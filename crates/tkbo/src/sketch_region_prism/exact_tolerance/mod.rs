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
