// Rust translation of FoundationClasses/TKMath/BVH/BVH_IndexedBoxSet.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_IndexedBoxSet<T, N, DataType>`: a box set permuted through an index array.
use super::{BvhBoxSet, swap_items};
use crate::bvh::array;
use crate::bvh::bounding_box::BvhBox;
use crate::bvh::object::{BvhObject, BvhObjectTransient};
use crate::bvh::primitive_set::{BvhBuilderHandle, BvhPrimitiveSet, BvhPrimitiveSetAccess};
use crate::bvh::set::BvhSet;
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::standard::Failure;

/// Box set whose builders swap only indices, never the elements or boxes.
///
/// Heavy element types then stay in place during BVH construction; selection
/// reads them through one extra indirection.
/// Example: `let mut set = BvhIndexedBoxSet::<BvhVec3d>::default(); set.add(7, bounding);`.
#[derive(Debug)]
pub struct BvhIndexedBoxSet<V: BvhVector, D = i32> {
    base: BvhBoxSet<V, D>,
    indices: LinearVector<i32>,
}
impl<V: BvhVector, D> Default for BvhIndexedBoxSet<V, D> {
    /// Uses the default binned builder of `BVH_PrimitiveSet`.
    fn default() -> Self {
        Self {
            base: BvhBoxSet::default(),
            indices: LinearVector::default(),
        }
    }
}
impl<V: BvhVector, D> BvhIndexedBoxSet<V, D> {
    /// Creates an empty set with a custom builder; `BvhIndexedBoxSet::<V>::new(Some(b))`.
    pub fn new(builder: BvhBuilderHandle<V>) -> Self {
        Self {
            base: BvhBoxSet::new(builder),
            indices: LinearVector::default(),
        }
    }
    /// Reserves room for the expected number of elements; `set.set_size(100)`.
    pub fn set_size(&mut self, size: usize) {
        self.indices.reserve(size);
        self.base.set_size(size);
    }
    /// Appends an element with its box and the next index; `set.add(3, bounding)`.
    pub fn add(&mut self, element: D, bounding: BvhBox<V>) {
        self.indices.append(self.indices.len() as i32);
        self.base.add(element, bounding);
    }
    /// Removes every index, element and box and marks the set dirty; `set.clear()`.
    pub fn clear(&mut self) {
        self.indices.clear(false);
        self.base.clear();
    }
    /// The storage slot of the element at `index`.
    fn slot(&self, index: i32) -> Result<i32, Failure> {
        array::value(&self.indices, index).copied()
    }
}
impl<V: BvhVector, D: Send> BvhIndexedBoxSet<V, D> {
    /// Builds the BVH when dirty (`Update`); `set.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        self.update()
    }
}
impl<V: BvhVector, D: Clone> BvhIndexedBoxSet<V, D> {
    /// Returns a copy of the element at `index`, read through the index array.
    /// Example: `set.element(0)?`.
    pub fn element(&self, index: i32) -> Result<D, Failure> {
        self.base.element(self.slot(index)?)
    }
}
impl<V: BvhVector, D: Send> BvhSet<V> for BvhIndexedBoxSet<V, D> {
    fn size(&self) -> i32 {
        self.base.size()
    }
    /// Returns the box stored at the slot the index array names.
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.base.element_box(self.slot(index)?)
    }
    /// The inherited `Box(index).Center(axis)` through the indexed box.
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        Ok(self.element_box(index)?.center_axis(axis))
    }
    /// Swaps only the two indices.
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        swap_items(&mut self.indices, first, second)
    }
    fn bounding_box(&self) -> Result<BvhBox<V>, Failure> {
        self.cached_box()
    }
}
impl<V: BvhVector, D: Send> BvhPrimitiveSetAccess<V> for BvhIndexedBoxSet<V, D> {
    fn primitive_set(&self) -> &BvhPrimitiveSet<V> {
        &self.base.primitive_set
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<V> {
        &mut self.base.primitive_set
    }
}
impl<V: BvhVector, D: Send> BvhObject<V> for BvhIndexedBoxSet<V, D> {
    fn object_transient(&self) -> &BvhObjectTransient {
        self.base.primitive_set.object_transient()
    }
    fn object_transient_mut(&mut self) -> &mut BvhObjectTransient {
        self.base.primitive_set.object_transient_mut()
    }
    fn object_box(&self) -> Result<BvhBox<V>, Failure> {
        self.cached_box()
    }
}
crate::bvh::primitive_set::primitive_tree_owner!(BvhIndexedBoxSet, D: Send);
