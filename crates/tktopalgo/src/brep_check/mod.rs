// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepCheck at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepCheck: the validity checks of a BRep shape.
//!
//! Each checker derives from the abstract [`BRepCheckResult`] (docs/topalgo-design.md
//! section 6): [`BRepCheckVertex`], [`BRepCheckEdge`], [`BRepCheckWire`],
//! [`BRepCheckFace`] and [`BRepCheckShell`] record the defects ([`BRepCheckStatus`]) of
//! their shape itself and of the shape in each context it was checked in.
//! `BRepCheck_Solid` and `BRepCheck_Analyzer`, which runs the checkers over a whole
//! shape, wait for `BRepClass3d_SolidClassifier`.
mod edge;
mod edge_context;
mod edge_tolerance;
mod errors;
mod explore;
mod face;
mod face_classify;
mod face_wires;
mod package;
mod result;
mod shell;
mod shell_closed;
mod shell_orientation;
mod status;
mod vertex;
mod wire;
mod wire_choix_uv;
mod wire_closed;
mod wire_orientation;
mod wire_self_intersect;
mod wire_self_pairs;

pub use edge::BRepCheckEdge;
pub use face::BRepCheckFace;
pub use package::BRepCheck;
pub use result::{BRepCheckResult, ResultBase};
pub use shell::BRepCheckShell;
pub use status::{BRepCheckStatus, ListOfStatus};
pub use vertex::BRepCheckVertex;
pub use wire::BRepCheckWire;
