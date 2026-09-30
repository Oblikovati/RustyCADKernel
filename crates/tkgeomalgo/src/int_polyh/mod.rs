// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntPolyh: the intersection of two surfaces through the intersection of
//! their triangulations, refined where they interfere (docs/geomalgo-design.md section
//! 12). `IntPolyhIntersection` is the API; `IntPolyhMaillageAffinage` the triangulations
//! and their comparison.
mod array;
mod dump;
mod edge;
mod intersection;
mod maillage;
mod point;
mod section_line;
mod start_point;
mod tools;
mod triangle;
mod triangle_refine;
mod triangle_split;

pub use array::{
    IntPolyhArray, IntPolyhArrayOfEdges, IntPolyhArrayOfPointNormal, IntPolyhArrayOfPoints,
    IntPolyhArrayOfSectionLines, IntPolyhArrayOfTangentZones, IntPolyhArrayOfTriangles,
    IntPolyhDumpItem, IntPolyhPointNormal,
};
pub use edge::{IntPolyhCouple, IntPolyhEdge};
pub use intersection::IntPolyhIntersection;
pub use maillage::{IntPolyhMaillageAffinage, IntPolyhTriangleEdge};
pub use point::IntPolyhPoint;
pub use section_line::IntPolyhSectionLine;
pub use start_point::IntPolyhStartPoint;
pub use tools::IntPolyhTools;
pub use triangle::IntPolyhTriangle;
