//! BRep offset geometry helpers.
mod classification;
mod collapse_singularities;
pub use classification::{BRepOffsetError, BRepOffsetMode};
mod interval;
pub use interval::BRepOffsetInterval;
mod make_loops;
pub use make_loops::BRepOffsetMakeLoops;
mod inter2d;
pub use inter2d::{BRepOffsetInter2d, CoincidentVertices, EdgeIntersections};
mod inter3d;
pub use inter3d::{BRepOffsetInter3d, OffsetFaces, ShapeShapeMap};
mod offset;
mod simple_offset;
mod simple_offset_angle;
mod simple_offset_edge;
mod simple_offset_face;
mod simple_offset_maker;
mod simple_offset_vertex;
mod simple_offset_vertex_map;
mod simple_offset_wall;
mod surface;
mod surface_analytic;
pub use offset::{BRepOffsetOffset, CreatedShapes, PipeOptions};
mod tool;
pub use tool::{BRepOffsetTool, EdgeNeighbours, EnlargeFaceOptions, ShapeImages, VertexEdges};

pub use crate::brep_offset_api::BRepOffsetAPIMakeOffsetShape;
pub use collapse_singularities::collapse_singularities;
mod build_offset_faces;
mod maker;
pub use maker::{BRepOffsetMakeOffset, MakeOffsetOptions};
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

pub use simple_offset_wall::build_simple_offset_wall;
mod analyse;
pub use analyse::BRepOffsetAnalyse;
