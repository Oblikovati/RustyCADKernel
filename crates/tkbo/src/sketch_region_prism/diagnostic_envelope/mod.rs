// SPDX-License-Identifier: GPL-2.0-only
//! ADR0048: owned diagnostic coverage is independent of the original operation class.
mod context;
mod preparation;
mod types;
mod views;
use super::owned_request::{names::*, *};
pub(crate) use preparation::project_owned_preparation_refusal;
use rustycad_tkernel::standard::Failure;
pub(crate) use types::*;
