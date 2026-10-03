//! BRep offset geometry helpers.
mod simple_offset_edge;
mod surface;
mod surface_analytic;

pub use simple_offset_edge::{OffsetEdgeGeometry, OffsetFaceSurface, build_simple_offset_edge};
pub use surface::{BRepOffsetStatus, surface};
