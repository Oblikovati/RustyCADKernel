// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepLib at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepLib: the makers of vertices, edges, wires, faces, shells and
//! solids, and the package functions that repair and complete the geometry of
//! shapes (3d curves, same parameter, tolerances, regularity).
//!
//! The makers derive from [`BRepLibMakeShape`] and [`BRepLibCommand`] through
//! `Deref`, and the process-wide [`precision`] and [`plane`] are the native statics
//! (docs/topalgo-design.md section 7).
mod bounding_vertex;
mod build_curve3d;
mod check_curve_on_surface;
mod command;
mod continuity;
mod deflection;
mod degenerated_curve;
mod encode_regularity;
mod errors;
mod extend_face;
mod find_surface;
mod fuse_edges;
mod globals;
mod inner_tolerances;
mod make_edge;
mod make_edge2d;
mod make_face;
mod make_polygon;
mod make_shape;
mod make_shell;
mod make_solid;
mod make_vertex;
mod make_wire;
mod normals;
mod orient_closed_solid;
mod pcurve_on_plane;
mod point_cloud_shape;
mod same_parameter;
mod same_range;
mod sort_faces;
mod tool_triangulated_shape;
mod update_edge_tol;
mod update_tolerances;
mod valid_range;
mod validate_edge;

pub use bounding_vertex::bounding_vertex;
pub use build_curve3d::{
    BUILD_CURVE3D_DEFAULTS, build_curve3d, build_curves3d, build_curves3d_default,
};
pub use check_curve_on_surface::BRepLibCheckCurveOnSurface;
pub use command::BRepLibCommand;
pub use continuity::continuity_of_faces;
pub use deflection::update_deflection;
pub use encode_regularity::{
    ENCODE_REGULARITY_TOL_ANG, encode_regularity, encode_regularity_of_edge,
    encode_regularity_of_edges,
};
pub use errors::{
    BRepLibEdgeError, BRepLibFaceError, BRepLibShapeModification, BRepLibShellError,
    BRepLibWireError,
};
pub use extend_face::{ExtendDirections, extend_face};
pub use find_surface::BRepLibFindSurface;
pub use fuse_edges::{BRepLibFuseEdges, FuseEdgeLists, FuseFaceMap};
pub use globals::{plane, precision, set_plane, set_precision};
pub use inner_tolerances::update_inner_tolerances;
pub use make_edge::BRepLibMakeEdge;
pub use make_edge2d::BRepLibMakeEdge2d;
pub use make_face::BRepLibMakeFace;
pub use make_polygon::BRepLibMakePolygon;
pub use make_shape::BRepLibMakeShape;
pub use make_shell::BRepLibMakeShell;
pub use make_solid::BRepLibMakeSolid;
pub use make_vertex::BRepLibMakeVertex;
pub use make_wire::BRepLibMakeWire;
pub use normals::{ENSURE_NORMAL_CONSISTENCY_DEFAULTS, ensure_normal_consistency};
pub use orient_closed_solid::orient_closed_solid;
pub use pcurve_on_plane::{
    build_pcurve_for_edge_on_plane, build_pcurve_for_edges_on_plane, pcurve_for_edge_on_plane,
};
pub use point_cloud_shape::{BRepLibPointCloudShape, BRepLibPointCloudShapeBase};
pub use same_parameter::{
    same_parameter_edge, same_parameter_edge_with_tolerance, same_parameter_shape,
    same_parameter_shape_with_reshape,
};
pub use same_range::{check_same_range, same_range};
pub use sort_faces::{reverse_sort_faces, sort_faces};
pub use tool_triangulated_shape::BRepLibToolTriangulatedShape;
pub use update_edge_tol::{update_edge_tol, update_edge_tolerance};
pub use update_tolerances::{update_tolerances, update_tolerances_with_reshape};
pub use valid_range::{RangeEnd, ValidRange, find_valid_range, find_valid_range_of_edge};
pub use validate_edge::BRepLibValidateEdge;
