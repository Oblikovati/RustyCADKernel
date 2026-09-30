// Rust translation of FoundationClasses/TKMath/BVH/BVH_BinaryTree.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CollapseToQuadTree`: breadth-first conversion keeping every second level.
use super::{BvhBinaryTree, BvhQuadTree, BvhTree};
use crate::bvh::array;
use crate::bvh::vector::{BvhVec4i, BvhVector};
use rustycad_tkernel::standard::Failure;
use std::collections::VecDeque;

impl<V: BvhVector> BvhTree<V, BvhBinaryTree> {
    /// Collapses the tree into a QBVH: each inner node adopts its grandchildren
    /// (or a leaf child directly), and children are stored contiguously.
    ///
    /// An inner QBVH record is `(0, first child, child count - 1, level)`.
    /// Example: `let quad = tree.collapse_to_quad_tree()?;`.
    pub fn collapse_to_quad_tree(&self) -> Result<BvhTree<V, BvhQuadTree>, Failure> {
        let mut quad = BvhTree::<V, BvhQuadTree>::default();
        if self.length() == 0 {
            return Ok(quad);
        }
        let mut queue: VecDeque<(i32, i32)> = VecDeque::from([(0, 0)]);
        let mut node_count = 1;
        while let Some(&(node, level)) = queue.front() {
            array::append(&mut quad.min_point_buffer, *self.min_point(node)?);
            array::append(&mut quad.max_point_buffer, *self.max_point(node)?);
            let record = if self.is_outer(node)? {
                let (begin, end) = (self.beg_primitive(node)?, self.end_primitive(node)?);
                BvhVec4i::new(1, begin, end, level)
            } else {
                let grand_children = self.grand_children(node)?;
                let count = grand_children.len() as i32;
                queue.extend(grand_children.iter().map(|&child| (child, level + 1)));
                quad.depth = quad.depth.max(level + 1);
                let record = BvhVec4i::new(0, node_count, count - 1, level);
                node_count += count;
                record
            };
            array::append(&mut quad.node_info_buffer, record);
            queue.pop_front();
        }
        Ok(quad)
    }

    /// Children of an inner node's children, a leaf child standing for itself.
    fn grand_children(&self, node: i32) -> Result<Vec<i32>, Failure> {
        let mut nodes = Vec::with_capacity(4);
        for child in [self.child::<0>(node)?, self.child::<1>(node)?] {
            if self.is_outer(child)? {
                nodes.push(child);
            } else {
                nodes.push(self.child::<0>(child)?);
                nodes.push(self.child::<1>(child)?);
            }
        }
        Ok(nodes)
    }
}
