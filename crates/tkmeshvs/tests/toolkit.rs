//! One executable per toolkit keeps repeated linkage out of Martensite tests.
#![allow(clippy::module_inception, clippy::duplicate_mod)] // Preserve case identity and fixture isolation.
mod native_replay;
mod tool_native;
include!("suites/cases_00.rs");
