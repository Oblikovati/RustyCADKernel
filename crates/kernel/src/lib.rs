//! Rust CAD kernel: a behavioral port of the pinned OCCT source tree.
//!
//! The first TKernel APIs are available through [`foundation`]. Full toolkit
//! acceptance requires all upstream tests and API mappings to pass.
//!
//! Read-only body queries for applications (ADR 0076): stable topology indices in
//! [`topology`], display tessellation in [`presentation`] and exact ray picking in
//! [`pick`]. None of them changes the body or its history.

pub use rustycad_tkernel as foundation;

pub mod pick;
pub mod presentation;
pub mod properties;
pub mod query_error;
pub mod topology;
pub mod validation;

#[cfg(test)]
mod invalid_shapes;
#[cfg(test)]
mod test_shapes;
#[cfg(test)]
mod validation_tests;
