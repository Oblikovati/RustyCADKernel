//! BRep offset geometry helpers.
mod collapse_singularities;
mod simple_offset;
mod simple_offset_angle;
mod simple_offset_edge;
mod simple_offset_face;
mod simple_offset_vertex;
mod simple_offset_vertex_map;
mod simple_offset_wall;
mod surface;
mod surface_analytic;

pub use collapse_singularities::collapse_singularities;
pub use simple_offset::BRepOffsetSimpleOffset;
pub use simple_offset_angle::{
    compute_simple_offset_max_angle, compute_simple_offset_safe_distance,
};
pub use simple_offset_edge::{OffsetEdgeGeometry, OffsetFaceSurface, build_simple_offset_edge};
pub use simple_offset_face::{SimpleOffsetFaceData, build_simple_offset_face_data};
pub use simple_offset_vertex::{
    OffsetVertexGeometry, OffsetVertexIncidentEdge, build_simple_offset_vertex,
};
pub use simple_offset_vertex_map::{OffsetVertexData, build_simple_offset_vertices};
pub use surface::{BRepOffsetStatus, surface};

pub use simple_offset_wall::build_simple_offset_wall;
