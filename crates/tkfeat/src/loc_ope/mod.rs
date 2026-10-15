//! `LocOpe`: the tools of the local topological operations: intersections of curves with
//! shapes, splitting of faces and shapes by wires, gluing, and the swept shapes (prism,
//! drafted prism, revolution, pipe, linear and revolution forms) with their history.
mod build_shape;
mod build_wires;
mod cs_intersector;
mod curve_shape_intersector;
mod dprism;
mod find_edges;
mod find_edges_in_face;
mod generator;
mod glued_shape;
mod gluer;
mod localize;
mod package;
mod pipe;
mod pnt_face;
mod split_shape;
mod spliter;
mod sweeps;
mod swept;
mod wires_on_shape;
pub use build_shape::BuildShape;
pub use build_wires::BuildWires;
pub use cs_intersector::CSIntersector;
pub use curve_shape_intersector::CurveShapeIntersector;
pub use dprism::DPrism;
pub use find_edges::FindEdges;
pub use find_edges_in_face::FindEdgesInFace;
pub use generator::Generator;
pub use glued_shape::{GeneratedShape, GluedShape};
pub use gluer::Gluer;
pub use package::{closed_edge, closed_wire, sample_edges, tgt_faces};
pub use pipe::Pipe;
pub use pnt_face::{Operation, PntFace};
pub use split_shape::SplitShape;
pub use spliter::Spliter;
pub use sweeps::{LinearForm, Prism, Revol, RevolutionForm};
pub use wires_on_shape::WiresOnShape;
