// Rust translation of ModelingAlgorithms/TKBO at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! OCCT TKBO foundations. The implemented IntTools types are exported by [`int_tools`].
//! Boolean options, alerts, progress callbacks and the API base are available.
//! Concrete Boolean builders and the full intersection pipeline remain incomplete.
pub mod bop_algo;
pub mod bop_ds;
pub mod brep_algo_api;
pub mod int_tools;
