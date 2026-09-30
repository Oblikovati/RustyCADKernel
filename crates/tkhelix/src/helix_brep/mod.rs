// Rust translation of ModelingAlgorithms/TKHelix/HelixBRep at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package HelixBRep: composite helix wires.
//!
//! [`HelixBRepBuilderHelix`] builds a wire of helix parts along an axis, each part from
//! the coils of `HelixGeom_BuilderHelix`, with the tangents averaged at the joints of
//! the parts (docs/helix-design.md sections 3 and 4).
mod builder_helix;
mod builder_helix_perform;
mod builder_helix_smoothing;

pub use builder_helix::HelixBRepBuilderHelix;
