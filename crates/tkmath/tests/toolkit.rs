//! One executable per toolkit keeps repeated linkage out of Martensite tests.
#![allow(clippy::module_inception, clippy::duplicate_mod)] // Preserve case identity and fixture isolation.
mod exact_roots;
mod sketch_line_incidences;
include!("suites/cases_00.rs");
include!("suites/cases_01.rs");
