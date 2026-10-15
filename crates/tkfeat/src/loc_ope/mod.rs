//! `LocOpe`: the tools of the local topological operations: intersections of curves with
//! shapes, splitting of faces and shapes by wires, gluing, and the swept shapes (prism,
//! drafted prism, revolution, pipe, linear and revolution forms) with their history.
mod build_shape;
mod build_wires;
mod cs_intersector;
mod curve_shape_intersector;
mod localize;
mod package;
mod pnt_face;
mod wires_on_shape;
pub use build_shape::BuildShape;
pub use build_wires::BuildWires;
pub use cs_intersector::CSIntersector;
pub use curve_shape_intersector::CurveShapeIntersector;
pub use package::{closed_edge, closed_wire, sample_edges, tgt_faces};
pub use pnt_face::{Operation, PntFace};
pub use wires_on_shape::WiresOnShape;
