// Rust translation of FoundationClasses/TKMath/BVH/BVH_LinearBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_LinearBuilder<T, N>`: LBVH construction from radix-sorted Morton codes.
use super::bounding_box::BvhBox;
use super::builder::{BvhBuilder, BvhBuilderTransient, update_depth};
use super::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH};
use super::radix_sorter::{BvhEncodedLink, BvhRadixSorter};
use super::set::BvhSet;
use super::sorter::BvhSorter;
use super::tree::BvhTree;
use super::vector::{BvhVec4i, BvhVector};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use std::marker::PhantomData;
mod bounds;
pub use bounds::{BvhUpdateBoundTask, update_bounds};

/// Fast LBVH builder (Lauterbach et al., "Fast BVH construction on GPUs", 2009).
///
/// The Morton-code radix sort reduces the build to a sort; the trees are worse
/// than SAH trees but built an order of magnitude faster. The depth limit is
/// stored but not applied, as upstream; node levels stay 0.
/// Example: `BvhLinearBuilder::<BvhVec3d>::new(1, 32).build(&mut set, &mut tree, &bounding)?`.
#[derive(Debug)]
pub struct BvhLinearBuilder<V: BvhVector> {
    transient: BvhBuilderTransient,
    vector: PhantomData<V>,
}
impl<V: BvhVector> Default for BvhLinearBuilder<V> {
    fn default() -> Self {
        Self::new(LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH)
    }
}
impl<V: BvhVector> BvhLinearBuilder<V> {
    /// Creates LBVH builder; `BvhLinearBuilder::<BvhVec3d>::new(5, 32)`.
    pub fn new(leaf_node_size: i32, max_tree_depth: i32) -> Self {
        Self {
            transient: BvhBuilderTransient::new(leaf_node_size, max_tree_depth),
            vector: PhantomData,
        }
    }

    /// `lowerBound`: first link in `[start, final)` whose code has bit `digit`
    /// set, by binary search over links already partitioned on that bit.
    /// Example: `builder.lower_bound(&links, 0, 8, 29)?`.
    pub fn lower_bound(
        &self,
        links: &Array1<'static, BvhEncodedLink>,
        mut start: i32,
        final_index: i32,
        digit: i32,
    ) -> Result<i32, Failure> {
        let mut count = final_index - start;
        let bit = 1_u32 << digit;
        while count > 0 {
            let step = count / 2;
            if link_code(links, start + step)? & bit != 0 {
                count = step;
            } else {
                start += step + 1;
                count -= step + 1;
            }
        }
        Ok(start)
    }

    /// `emitHierachy`: emits the subtree of links `[start, final)` and returns
    /// its root. Leaves cover primitives `[shift, shift + final - start - 1]`;
    /// inner nodes split at the first link with bit `digit` set, or at the
    /// middle once the digits run out. Point buffers are not extended.
    /// Example: `builder.emit_hierarchy(&mut tree, &links, 29, 0, 0, size)?`.
    pub fn emit_hierarchy(
        &self,
        bvh: &mut BvhTree<V>,
        links: &Array1<'static, BvhEncodedLink>,
        digit: i32,
        shift: i32,
        start: i32,
        final_index: i32,
    ) -> Result<i32, Failure> {
        if final_index - start <= self.transient.leaf_node_size() {
            return Ok(bvh.add_leaf_node_uninit(shift, shift + final_index - start - 1));
        }
        let position = self.split_position(links, digit, start, final_index)?;
        if position == start || position == final_index {
            return self.emit_hierarchy(bvh, links, digit - 1, shift, start, final_index);
        }
        let node = bvh.add_inner_node_uninit(0, 0);
        let right_shift = shift + position - start;
        let left = self.emit_hierarchy(bvh, links, digit - 1, shift, start, position)?;
        let right =
            self.emit_hierarchy(bvh, links, digit - 1, right_shift, position, final_index)?;
        let record: &mut BvhVec4i = &mut bvh.node_info_buffer_mut()[node as usize];
        record.y = left;
        record.z = right;
        Ok(node)
    }

    /// The split link of a range; a middle split that cannot divide the range
    /// (leaf size below 1) recurses forever upstream and fails here instead.
    fn split_position(
        &self,
        links: &Array1<'static, BvhEncodedLink>,
        digit: i32,
        start: i32,
        final_index: i32,
    ) -> Result<i32, Failure> {
        if digit >= 0 {
            return self.lower_bound(links, start, final_index, digit);
        }
        let position = (start + final_index) / 2;
        if position == start || position == final_index {
            return Err(Failure::with_type(
                "Standard_StackOverflow",
                &format!(
                    "BVH_LinearBuilder::emitHierachy: range [{start}, {final_index}) cannot be split \
                     with leaf node size {}, expected >= 1",
                    self.transient.leaf_node_size()
                ),
                "",
            ));
        }
        Ok(position)
    }
}

/// The Morton code of a link, checked like `NCollection_Array1::Value`.
fn link_code(links: &Array1<'static, BvhEncodedLink>, index: i32) -> Result<u32, Failure> {
    links.value(index).map(|link| link.0).map_err(|_| {
        Failure::out_of_range(&format!(
            "NCollection_Array1::Value: index {index}, expected {}..={}",
            links.lower(),
            links.upper()
        ))
    })
}

impl<V: BvhVector> BvhBuilder<V> for BvhLinearBuilder<V> {
    fn builder_transient(&self) -> &BvhBuilderTransient {
        &self.transient
    }
    /// Radix-sorts the set by Morton code inside `bounding`, emits the hierarchy,
    /// then computes the node boxes bottom-up. An empty set leaves the tree as it
    /// was: upstream returns before `Clear()`.
    fn build(
        &self,
        set: &mut dyn BvhSet<V>,
        bvh: &mut BvhTree<V>,
        bounding: &BvhBox<V>,
    ) -> Result<(), Failure> {
        if set.size() == 0 {
            return Ok(());
        }
        bvh.clear();
        let mut sorter = BvhRadixSorter::new(*bounding);
        sorter.set_parallel(self.transient.is_parallel());
        sorter.perform(set)?;
        let links = sorter.encoded_links().ok_or_else(|| {
            Failure::new("BVH_LinearBuilder: radix sorter produced no links, expected sorted links")
        })?;
        self.emit_hierarchy(bvh, links, 29, 0, 0, set.size())?;
        let node_count = bvh.node_info_buffer().len();
        bvh.min_point_buffer_mut().resize(node_count);
        bvh.max_point_buffer_mut().resize(node_count);
        let height = BvhUpdateBoundTask::new(self.transient.is_parallel()).run(set, bvh, 0, 0)?;
        update_depth(bvh, height);
        Ok(())
    }
}
