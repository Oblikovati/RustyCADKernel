// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepMAT2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepMAT2d: the bisecting locus (the medial axis) of a planar face.
//!
//! [`BRepMAT2dExplorer`] reads the contours of a face as 2d curves,
//! [`BRepMAT2dBisectingLocus`] computes their bisecting locus with MAT2d and MAT, and
//! [`BRepMAT2dLinkTopoBilo`] links the basic elements of the locus to the edges and
//! vertices of the face (docs/topalgo-design.md section 14).
mod bisecting_locus;
mod explorer;
mod explorer_wire;
mod link_topo_bilo;

pub use bisecting_locus::BRepMAT2dBisectingLocus;
pub use explorer::{BRepMAT2dContour, BRepMAT2dExplorer};
pub use link_topo_bilo::BRepMAT2dLinkTopoBilo;
