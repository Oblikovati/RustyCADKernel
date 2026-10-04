// Rust translation of FoundationClasses/TKMath/BVH/BVH_Tree.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_TreeBase<T, N>` and the arity-tagged `BVH_Tree<T, N, Arity>`.
//!
//! Each node is a record `(outer flag, begin/first child, end/second child, level)`
//! plus a minimum and a maximum point in parallel buffers.
use super::array;
use super::vector::{BvhArray4i, BvhVec4i, BvhVector};
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::standard::Failure;
use std::marker::PhantomData;
mod binary;
mod collapse;
mod json;
mod quad;

/// `BVH_BinaryTree`: tag of the binary BVH specialization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BvhBinaryTree;

/// `BVH_QuadTree`: tag of the quad BVH (QBVH) specialization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BvhQuadTree;

/// Immutable node inputs passed to a shared-set worker split.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhNodeSnapshot<V: BvhVector> {
    /// Node index in the source tree.
    pub index: i32,
    /// First primitive in the inclusive node range.
    pub begin: i32,
    /// Last primitive in the inclusive node range.
    pub end: i32,
    /// Depth of this node.
    pub level: i32,
    /// Node minimum bound.
    pub minimum: V,
    /// Node maximum bound.
    pub maximum: V,
}

/// Bounding volume hierarchy of arity 2 (default) or 4.
///
/// The buffers are public data upstream (builders write them directly), so
/// mutable access is exposed. Node indices are `int`; out-of-range indices
/// raise `Standard_OutOfRange` upstream and return `Err` here.
/// Example: `let mut tree = BvhTree::<Vec3<f64>>::default();`.
#[derive(Clone, Debug)]
pub struct BvhTree<V: BvhVector, A = BvhBinaryTree> {
    node_info_buffer: BvhArray4i,
    min_point_buffer: LinearVector<V>,
    max_point_buffer: LinearVector<V>,
    depth: i32,
    arity: PhantomData<A>,
}
impl<V: BvhVector, A> Default for BvhTree<V, A> {
    fn default() -> Self {
        Self {
            node_info_buffer: LinearVector::default(),
            min_point_buffer: LinearVector::default(),
            max_point_buffer: LinearVector::default(),
            depth: 0,
            arity: PhantomData,
        }
    }
}
impl<V: BvhVector, A> BvhTree<V, A> {
    /// Copies stable split inputs while a parallel builder holds the tree lock.
    pub(crate) fn node_snapshot(&self, node: i32) -> Result<BvhNodeSnapshot<V>, Failure> {
        Ok(BvhNodeSnapshot {
            index: node,
            begin: self.beg_primitive(node)?,
            end: self.end_primitive(node)?,
            level: self.level(node)?,
            minimum: *self.min_point(node)?,
            maximum: *self.max_point(node)?,
        })
    }

    /// Returns depth (height) of BVH tree; `tree.depth()`.
    pub fn depth(&self) -> i32 {
        self.depth
    }
    /// Mutable depth, the public `myDepth` that builders update; `*tree.depth_mut() = 3`.
    pub fn depth_mut(&mut self) -> &mut i32 {
        &mut self.depth
    }
    /// Returns total number of BVH tree nodes (node records); `tree.length()`.
    pub fn length(&self) -> i32 {
        array::size(&self.node_info_buffer)
    }
    /// Returns minimum point of the given node; `tree.min_point(0)?.x`.
    pub fn min_point(&self, node: i32) -> Result<&V, Failure> {
        array::value(&self.min_point_buffer, node)
    }
    /// Returns maximum point of the given node; `tree.max_point(0)?.x`.
    pub fn max_point(&self, node: i32) -> Result<&V, Failure> {
        array::value(&self.max_point_buffer, node)
    }
    /// Mutable minimum point of the given node; `*tree.min_point_mut(0)? = point`.
    pub fn min_point_mut(&mut self, node: i32) -> Result<&mut V, Failure> {
        array::change_value(&mut self.min_point_buffer, node)
    }
    /// Mutable maximum point of the given node; `*tree.max_point_mut(0)? = point`.
    pub fn max_point_mut(&mut self, node: i32) -> Result<&mut V, Failure> {
        array::change_value(&mut self.max_point_buffer, node)
    }
    /// Returns index of first primitive of the given leaf node; `tree.beg_primitive(0)?`.
    pub fn beg_primitive(&self, node: i32) -> Result<i32, Failure> {
        Ok(self.node_info(node)?.y)
    }
    /// Returns index of last primitive of the given leaf node; `tree.end_primitive(0)?`.
    pub fn end_primitive(&self, node: i32) -> Result<i32, Failure> {
        Ok(self.node_info(node)?.z)
    }
    /// Mutable first primitive of a leaf; `*tree.beg_primitive_mut(0)? = 10`.
    pub fn beg_primitive_mut(&mut self, node: i32) -> Result<&mut i32, Failure> {
        Ok(&mut self.node_info_mut(node)?.y)
    }
    /// Mutable last primitive of a leaf; `*tree.end_primitive_mut(0)? = 20`.
    pub fn end_primitive_mut(&mut self, node: i32) -> Result<&mut i32, Failure> {
        Ok(&mut self.node_info_mut(node)?.z)
    }
    /// Returns `End - Beg + 1` for the given leaf node; `tree.nb_primitives(0)?`.
    pub fn nb_primitives(&self, node: i32) -> Result<i32, Failure> {
        Ok(self.end_primitive(node)? - self.beg_primitive(node)? + 1)
    }
    /// Returns level (depth) of the given node; `tree.level(0)?`.
    pub fn level(&self, node: i32) -> Result<i32, Failure> {
        Ok(self.node_info(node)?.w)
    }
    /// Mutable level of the given node; `*tree.level_mut(0)? = 5`.
    pub fn level_mut(&mut self, node: i32) -> Result<&mut i32, Failure> {
        Ok(&mut self.node_info_mut(node)?.w)
    }
    /// Checks whether the given node is outer (a leaf); `tree.is_outer(0)?`.
    pub fn is_outer(&self, node: i32) -> Result<bool, Failure> {
        Ok(self.node_info(node)?.x != 0)
    }
    /// Returns array of node data records; `array::size(tree.node_info_buffer())`.
    pub fn node_info_buffer(&self) -> &BvhArray4i {
        &self.node_info_buffer
    }
    /// Mutable array of node data records; `tree.node_info_buffer_mut()`.
    pub fn node_info_buffer_mut(&mut self) -> &mut BvhArray4i {
        &mut self.node_info_buffer
    }
    /// Returns array of node minimum points; `tree.min_point_buffer()`.
    pub fn min_point_buffer(&self) -> &LinearVector<V> {
        &self.min_point_buffer
    }
    /// Mutable array of node minimum points; `tree.min_point_buffer_mut()`.
    pub fn min_point_buffer_mut(&mut self) -> &mut LinearVector<V> {
        &mut self.min_point_buffer
    }
    /// Returns array of node maximum points; `tree.max_point_buffer()`.
    pub fn max_point_buffer(&self) -> &LinearVector<V> {
        &self.max_point_buffer
    }
    /// Mutable array of node maximum points; `tree.max_point_buffer_mut()`.
    pub fn max_point_buffer_mut(&mut self) -> &mut LinearVector<V> {
        &mut self.max_point_buffer
    }
    fn node_info(&self, node: i32) -> Result<&BvhVec4i, Failure> {
        array::value(&self.node_info_buffer, node)
    }
    fn node_info_mut(&mut self, node: i32) -> Result<&mut BvhVec4i, Failure> {
        array::change_value(&mut self.node_info_buffer, node)
    }
}
