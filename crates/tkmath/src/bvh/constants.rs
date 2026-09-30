// Rust translation of FoundationClasses/TKMath/BVH/BVH_Constants.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Constants`: tree depth, leaf sizes and bin counts.

/// `BVH_Constants_MaxTreeDepth`: optimal depth, in sync with traversal stacks.
pub const MAX_TREE_DEPTH: i32 = 32;
/// `BVH_Constants_LeafNodeSizeSingle`: for upper-level trees of complex nodes.
pub const LEAF_NODE_SIZE_SINGLE: i32 = 1;
/// `BVH_Constants_LeafNodeSizeAverage`: 4 primitives per leaf.
pub const LEAF_NODE_SIZE_AVERAGE: i32 = 4;
/// `BVH_Constants_LeafNodeSizeDefault`: 5 primitives per leaf.
pub const LEAF_NODE_SIZE_DEFAULT: i32 = 5;
/// `BVH_Constants_LeafNodeSizeSmall`: 8 primitives per leaf (e.g. triangles).
pub const LEAF_NODE_SIZE_SMALL: i32 = 8;
/// `BVH_Constants_NbBinsOptimal`: bin count of the binned builder.
pub const NB_BINS_OPTIMAL: usize = 32;
/// `BVH_Constants_NbBinsBest`: maximum useful bin count of the binned builder.
pub const NB_BINS_BEST: usize = 48;
/// `BVH::THE_NODE_MIN_SIZE`: minimum node size to split (a `double`).
pub const THE_NODE_MIN_SIZE: f64 = 1e-5;
