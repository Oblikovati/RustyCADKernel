// Rust translation of ModelingAlgorithms/TKHLR/HLRAlgo/HLRAlgo_PolyInternalNode.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! HLRAlgo_PolyInternalNode: a node of an internal triangulation.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{SharedState, StandardTransient};
use rustycad_tkmath::gp::{Xy, Xyz};
use std::sync::RwLockWriteGuard;

/// HLRAlgo_PolyInternalNode::NodeIndices: the first segment at the node, its flags and
/// the two edges it lies on.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct NodeIndices {
    pub nd_sg: i32,
    pub flag: i32,
    pub edg1: i32,
    pub edg2: i32,
}

/// HLRAlgo_PolyInternalNode::NodeData: the projected point, the normal, the surface
/// parameters, the parameters on the two edges and the scalar product of the normal
/// with the sight direction.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // The native field names, documented on the type.
pub struct NodeData {
    pub point: Xyz,
    pub normal: Xyz,
    pub uv: Xy,
    pub p_cu1: f64,
    pub p_cu2: f64,
    pub scal: f64,
}

/// HLRAlgo_PolyInternalNode: a transient node (shared by the node arrays of the
/// internal data) holding its indices and its data, both zero at construction.
///
/// The native `Indices()` and `Data()` return references that callers keep while they
/// write through other nodes. The port returns a write guard for each: a caller holds
/// at most one guard of a node at a time, since two nodes of a triangle can be the
/// same node (docs/hlr-design.md section 5).
///
/// ```
/// use rustycad_tkhlr::hlr_algo::PolyInternalNode;
/// let node = PolyInternalNode::new();
/// node.indices().nd_sg = 4;
/// node.data().scal = 0.5;
/// assert_eq!((node.indices().nd_sg, node.data().scal), (4, 0.5));
/// ```
#[derive(Default)]
pub struct PolyInternalNode {
    base: StandardTransient,
    indices: SharedState<NodeIndices>,
    data: SharedState<NodeData>,
}
impl_transient!(
    PolyInternalNode,
    "HLRAlgo_PolyInternalNode",
    StandardTransient,
    base
);

impl PolyInternalNode {
    /// HLRAlgo_PolyInternalNode(): zero indices and data; `PolyInternalNode::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Indices(): the indices, writable through the guard; `node.indices().flag`.
    pub fn indices(&self) -> RwLockWriteGuard<'_, NodeIndices> {
        self.indices.write()
    }

    /// Data(): the data, writable through the guard; `node.data().point`.
    pub fn data(&self) -> RwLockWriteGuard<'_, NodeData> {
        self.data.write()
    }
}
