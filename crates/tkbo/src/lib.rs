// Rust translation of ModelingAlgorithms/TKBO at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! OCCT TKBO foundations. The implemented IntTools types are exported by [`int_tools`].
//! Boolean options, alerts, progress callbacks and the API base are available.
//! Concrete Boolean builders and the full intersection pipeline remain incomplete.
pub mod bop_algo;
pub mod bop_ds;
pub mod bop_tools;
pub mod brep_algo_api;
pub mod int_tools;

#[allow(dead_code)] // ADR0014: private partial stage, not a successful region API.
mod sketch_region_validation;

// ADR0014 keeps this prerequisite internal until the complete owned-region contract passes.
#[allow(dead_code, unused_imports)]
pub(crate) mod sketch_region;

// ADR0014 candidate diagnostics cannot expose an uncertified healthy body.
#[allow(dead_code, unused_imports)]
pub(crate) mod sketch_region_candidates;

#[cfg(test)]
#[path = "../tests/support/changing_child_count.rs"]
mod changing_child_count;

#[cfg(test)]
#[path = "../tests/support/closed_analytic_cubic.rs"]
pub(crate) mod closed_analytic_cubic;
