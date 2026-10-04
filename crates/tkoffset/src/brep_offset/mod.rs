//! BRep offset geometry helpers.
mod classification;
mod collapse_singularities;
mod correct_singular_face;
pub use classification::{BRepOffsetError, BRepOffsetMode};
mod interval;
pub use interval::BRepOffsetInterval;
mod make_offset;
mod make_offset_build;
mod make_offset_check;
mod make_offset_history;
mod offset_face;
mod offset_rails;
mod offset_rails_geometry;
mod offset_tube;
mod simple_offset;
mod simple_offset_angle;
mod simple_offset_edge;
mod simple_offset_face;
mod simple_offset_maker;
mod simple_offset_vertex;
mod simple_offset_vertex_map;
mod simple_offset_wall;
mod sphere_join;
mod surface;
mod surface_analytic;

pub use crate::brep_offset_api::BRepOffsetAPIMakeOffsetShape;
pub use collapse_singularities::collapse_singularities;
pub use make_offset::{BRepOffsetMakeOffset, BRepOffsetMakeOffsetConfiguration};
pub use simple_offset::BRepOffsetSimpleOffset;
pub use simple_offset_angle::{
    compute_simple_offset_max_angle, compute_simple_offset_safe_distance,
};
pub use simple_offset_edge::{OffsetEdgeGeometry, OffsetFaceSurface, build_simple_offset_edge};
pub use simple_offset_face::{SimpleOffsetFaceData, build_simple_offset_face_data};
pub use simple_offset_maker::{BRepOffsetMakeSimpleOffset, BRepOffsetSimpleStatus};
pub use simple_offset_vertex::{
    OffsetVertexGeometry, OffsetVertexIncidentEdge, build_simple_offset_vertex,
};
pub use simple_offset_vertex_map::{OffsetVertexData, build_simple_offset_vertices};
pub use surface::{BRepOffsetStatus, surface};

pub use offset_face::BRepOffsetOffsetFace;
pub use offset_rails::BRepOffsetOffset;
pub use simple_offset_wall::build_simple_offset_wall;
pub use sphere_join::build_vertex_arc_join;
mod analyse;
pub use analyse::BRepOffsetAnalyse;
