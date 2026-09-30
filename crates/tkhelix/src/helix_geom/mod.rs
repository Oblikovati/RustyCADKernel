// Rust translation of ModelingAlgorithms/TKHelix/HelixGeom at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package HelixGeom: the analytic helix and its B-spline approximation.
//!
//! [`HelixGeomHelixCurve`] evaluates the helix as an `Adaptor3d_Curve`,
//! [`HelixGeomTools`] approximates it (or any adapted curve) with AdvApprox, and the
//! builders [`HelixGeomBuilderHelixCoil`] (one coil) and [`HelixGeomBuilderHelix`]
//! (one coil per turn, placed in a frame) implement the abstract builder traits
//! [`HelixGeomBuilderApproxCurve`] and [`HelixGeomBuilderHelixGen`]
//! (docs/helix-design.md sections 1 and 2).
mod builder_approx_curve;
mod builder_helix;
mod builder_helix_coil;
mod builder_helix_gen;
pub(crate) mod errors;
mod helix_curve;
mod tools;

pub use builder_approx_curve::{HelixGeomBuilderApproxCurve, HelixGeomBuilderApproxCurveBase};
pub use builder_helix::HelixGeomBuilderHelix;
pub use builder_helix_coil::HelixGeomBuilderHelixCoil;
pub use builder_helix_gen::{HelixGeomBuilderHelixGen, HelixGeomBuilderHelixGenBase};
pub use helix_curve::{HelixGeomHelixCurve, HelixParameters};
pub use tools::HelixGeomTools;
