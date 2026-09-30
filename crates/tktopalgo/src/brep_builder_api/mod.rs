// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepBuilderAPI at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepBuilderAPI: the application interface of the BRep construction
//! algorithms. The makers of vertices, edges, wires, faces, shells and solids wrap the
//! BRepLib makers; the modifications (transformations, copy, NURBS conversion) run a
//! `BRepTools_Modifier`; [`BRepBuilderAPIMakeShape`] is the virtual root that the
//! algorithms of the higher toolkits implement (docs/topalgo-design.md section 10).
//!
//! The sewing classes (`BRepBuilderAPI_Sewing`, `FastSewing`, `MakeShapeOnMesh`,
//! `BndBoxTreeSelector`, `VertexInspector`) are ported separately.
mod collect;
mod command;
mod copy;
mod errors;
mod find_plane;
mod gtransform;
mod make_edge;
mod make_edge2d;
mod make_face;
mod make_polygon;
mod make_shape;
mod make_shell;
mod make_solid;
mod make_vertex;
mod make_wire;
mod modify_shape;
mod nurbs_convert;
mod package;
mod transform;
mod wrap;

pub use collect::{BRepBuilderAPICollect, ShapeHistoryMap};
pub use command::BRepBuilderAPICommand;
pub use copy::BRepBuilderAPICopy;
pub use errors::{
    BRepBuilderAPIEdgeError, BRepBuilderAPIFaceError, BRepBuilderAPIPipeError,
    BRepBuilderAPIShellError, BRepBuilderAPITransitionMode, BRepBuilderAPIWireError,
};
pub use find_plane::BRepBuilderAPIFindPlane;
pub use gtransform::BRepBuilderAPIGTransform;
pub use make_edge::BRepBuilderAPIMakeEdge;
pub use make_edge2d::BRepBuilderAPIMakeEdge2d;
pub use make_face::BRepBuilderAPIMakeFace;
pub use make_polygon::BRepBuilderAPIMakePolygon;
pub use make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
pub use make_shell::BRepBuilderAPIMakeShell;
pub use make_solid::BRepBuilderAPIMakeSolid;
pub use make_vertex::BRepBuilderAPIMakeVertex;
pub use make_wire::BRepBuilderAPIMakeWire;
pub use modify_shape::BRepBuilderAPIModifyShape;
pub use nurbs_convert::BRepBuilderAPINurbsConvert;
pub use package::{plane, precision, set_plane, set_precision};
pub use transform::BRepBuilderAPITransform;
