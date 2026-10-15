//! Rust CAD kernel: a behavioral port of the pinned OCCT source tree.
//!
//! The first TKernel APIs are available through [`foundation`]. Full toolkit
//! acceptance requires all upstream tests and API mappings to pass.

pub use rustycad_tkernel as foundation;

pub mod properties;
/// Ordered validation of any owned body (ADR 0072): [`validation::validate_body`].
pub use rustycad_tkbo::body_check as validation;

#[cfg(test)]
mod invalid_shapes;
#[cfg(test)]
mod test_shapes;
#[cfg(test)]
mod validation_tests;
