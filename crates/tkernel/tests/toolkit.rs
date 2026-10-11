//! One executable per toolkit keeps repeated linkage out of Martensite tests.
#![allow(clippy::module_inception, clippy::duplicate_mod)] // Preserve case identity and fixture isolation.
include!("suites/cases_00.rs");
include!("suites/cases_01.rs");
include!("suites/cases_02.rs");
include!("suites/cases_03.rs");
