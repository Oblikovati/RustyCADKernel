// SPDX-License-Identifier: GPL-2.0-only
//! ADR0037 bounded history composition; observation alone never publishes a valid body.
mod bounds;
mod context_names;
mod contexts;
mod copied_contexts;
mod types;
use types::*;
mod limits;
use super::*;
use limits::*;
#[cfg(test)]
mod bounds_tests;

#[cfg(test)]
mod context_tests;

#[cfg(test)]
mod copied_context_tests;
