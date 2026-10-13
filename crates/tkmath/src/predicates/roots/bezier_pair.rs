// SPDX-License-Identifier: GPL-2.0-only
//! Actual paired source equations; no complete provider claim before lifting and coverage gates.
mod equations;
mod polynomial;
mod projection;
mod source;

mod fiber_row;

mod algebraic;
mod operation;
#[cfg(test)]
mod operation_tests;

mod contact_construction;
mod graph_enclosure;
mod membership;
#[cfg(test)]
mod native_replay;
mod operation_failure;
#[cfg(test)]
mod operation_limits_tests;

mod exceptional;
