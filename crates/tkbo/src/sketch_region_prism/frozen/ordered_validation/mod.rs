// SPDX-License-Identifier: GPL-2.0-only
//! ADR0039 ordered stage bookkeeping; a trace alone is never a validation certificate.
mod stages;
pub(crate) use stages::{OrderedPrismStage, OrderedPrismStageStatus, OrderedPrismStageTrace};
#[cfg(test)]
mod joined_baseline_tests;
#[cfg(test)]
mod stage_tests;
