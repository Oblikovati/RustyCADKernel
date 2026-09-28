//! OCCT Poly: triangulations, polygons, mesh connectivity and mesh tools.
//!
//! Transient classes (`Triangulation`, the polygons) are shared through TKernel `Handle`s
//! and keep their fields behind a lock, so every native member takes `&self` and a change
//! made through one handle is seen through all of them. The static `Poly` functions are
//! free functions of this module.
mod coherent;
mod connect;
mod errors;
mod harray;
mod io;
pub mod make_loops;
mod merge_nodes;
pub mod mesh_purpose;
mod nodes;
mod owned;
mod package;
mod parameters;
mod points;
mod polygon2d;
mod polygon3d;
mod polygon_on_triangulation;
mod shared_state;
mod triangle;
mod triangulation;
pub use coherent::{
    CoherentLink, CoherentNode, CoherentTriPtr, CoherentTriPtrIterator, CoherentTriangle,
    CoherentTriangulation, TwoIntegers,
};
pub use connect::Connect;
pub use harray::HArray1OfReal;
pub use io::{
    dump_polygon2d, dump_polygon3d, dump_triangulation, read_polygon2d, read_polygon3d,
    read_triangulation, write_polygon2d, write_polygon3d, write_triangulation,
};
pub use merge_nodes::MergeNodesTool;
pub use mesh_purpose::MeshPurpose;
pub use nodes::{ArrayOfNodes, ArrayOfUvNodes, NodeArray};
pub use package::{
    ListOfTriangulation, catenate, compute_normals, intersect, intersect_tri_line,
    point_on_triangle, polygon_properties,
};
pub use parameters::TriangulationParameters;
pub use polygon_on_triangulation::PolygonOnTriangulation;
pub use polygon2d::Polygon2D;
pub use polygon3d::Polygon3D;
pub use triangle::{Array1OfTriangle, Triangle};
pub use triangulation::{DeferredTriangulation, Triangulation};
