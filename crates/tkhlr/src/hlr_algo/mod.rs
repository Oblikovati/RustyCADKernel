// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package HLRAlgo: the data structures of the hidden line algorithms.
//!
//! - the 16-direction boxes and their integer packing (the package functions,
//!   [`MinMaxIndices`]), the wire and edge blocks of a face ([`WiresBlock`],
//!   [`EdgesBlock`]);
//! - the visible parts of an edge and their walk ([`EdgeStatus`], [`EdgeIterator`]),
//!   the records of an edge-face interference ([`Intersection`], [`Coincidence`],
//!   [`Interference`]) and the projection ([`Projector`]);
//! - the polyhedral algorithm: segments ([`BiPoint`]), faces ([`PolyData`]), shells
//!   ([`PolyShellData`]), the internal triangulations ([`PolyInternalData`] with its
//!   nodes, segments and triangles) and the algorithm itself ([`PolyAlgo`]).
//!
//! docs/hlr-design.md section 5 fixes how the transients, their reference accessors
//! and the native array pointers map to Rust.
mod bi_point;
mod edge_iterator;
mod edge_status;
mod edges_block;
mod failures;
mod interference;
mod min_max;
mod poly_algo;
mod poly_algo_update;
mod poly_data;
mod poly_data_hide;
mod poly_internal_data;
mod poly_internal_links;
mod poly_internal_node;
mod poly_internal_split;
mod poly_records;
mod poly_shell_data;
mod projector;
mod projector_project;
mod wires_block;

pub use bi_point::{BiPoint, BiPointFace, BiPointIndices, BiPointPoints, BiPointRegularity};
pub use edge_iterator::EdgeIterator;
pub use edge_status::EdgeStatus;
pub use edges_block::EdgesBlock;
pub use interference::{Coincidence, Interference, Intersection};
pub use min_max::{
    MinMaxIndices, add_min_max, copy_min_max, decode_min_max, encode_min_max, enlarge_min_max,
    init_min_max, size_box, update_min_max,
};
pub use poly_algo::{PolyAlgo, PolySegment};
pub use poly_data::{FaceIndices, PolyData, PolyTriangle};
pub use poly_internal_data::{NodeArray, PolyInternalData, SegmentArray, TriangleArray};
pub use poly_internal_node::{NodeData, NodeIndices, PolyInternalNode};
pub use poly_records::{
    HidingPlane, HidingTriangleIndices, PolyHidingData, PolyInternalSegment, PolyMask, TriangleData,
};
pub use poly_shell_data::{PolyShellData, ShellIndices};
pub use projector::Projector;
pub use wires_block::WiresBlock;

pub(crate) use failures::c_int_truncation;
