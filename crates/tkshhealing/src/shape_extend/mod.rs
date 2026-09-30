// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeExtend: the status flags, the wire data, the complex curve, the
//! composite surface, the explorer and the message registrators shared by the other
//! Shape Healing packages.
//!
//! Ported so far: the status flags ([`ShapeExtendStatus`], [`ShapeExtendParametrisation`])
//! and their bit codes ([`ShapeExtend`]); the other classes land here when the package is
//! ported (docs/tkshhealing-progress.md lists the ported packages).
mod package;
mod status;

pub use package::ShapeExtend;
pub use status::{ShapeExtendParametrisation, ShapeExtendStatus};
