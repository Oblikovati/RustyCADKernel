//! Behavior of the Standard runtime API: type descriptors, transients, handles,
//! failures, the error handler, asserts and TColStd_HPackedMapOfInteger.
//! Tests that change the process-wide stack-trace length live in
//! `standard_runtime_stack_api_behavior.rs`, a separate process.
mod assert;
mod error_handler;
mod failure;
mod handles;
mod hpacked_map;
mod transients;
mod types;
