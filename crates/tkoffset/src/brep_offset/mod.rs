//! BRep offset geometry helpers.
mod simple_offset_edge;
mod simple_offset_vertex;
mod simple_offset_vertex_map;

pub use simple_offset_edge::{OffsetEdgeGeometry, OffsetFaceSurface, build_simple_offset_edge};
pub use simple_offset_vertex::{
    OffsetVertexGeometry, OffsetVertexIncidentEdge, build_simple_offset_vertex,
};
pub use simple_offset_vertex_map::{OffsetVertexData, build_simple_offset_vertices};
