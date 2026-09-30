// Rust translation of ModelingAlgorithms/TKHLR/Contap at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Contour records; the contour algorithms are being ported separately.
mod line;
mod point;
mod types;
pub use line::ContapLine;
pub use point::ContapPoint;
pub use types::{ContapIType, ContapTFunction};

mod cont_ana;
mod cont_ana_cone;
mod geometry;
pub use cont_ana::ContapContAna;

mod cont_ana_cylinder;
mod cont_ana_sphere;
