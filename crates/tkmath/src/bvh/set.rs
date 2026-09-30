// Rust translation of FoundationClasses/TKMath/BVH/BVH_Set.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Set<T, N>`: the minimal geometry interface needed to construct a BVH.
use super::bounding_box::BvhBox;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// Set of abstract entities bounded by BVH boxes.
///
/// Builders and sorters permute the set through [`BvhSet::swap`]. `Send` lets a
/// parallel builder hand the set to its worker threads.
/// Example: `sorter.perform(&mut triangulation)?`.
pub trait BvhSet<V: BvhVector>: Send {
    /// Returns total number of objects; `set.size()`.
    fn size(&self) -> i32;
    /// Returns AABB of the given object, as `Box(theIndex)`; `set.element_box(0)?`.
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure>;
    /// Returns centroid position along the given axis; `set.center(0, 1)?`.
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure>;
    /// Performs transposing the two given objects in the set; `set.swap(0, 1)?`.
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure>;
    /// Returns AABB of the entire set, as `Box()`; primitive sets override this
    /// with their cached box. Example: `set.bounding_box()?`.
    fn bounding_box(&self) -> Result<BvhBox<V>, Failure> {
        combined_element_boxes(self)
    }
}

/// `BVH_Set::Box()`: combines every element box in index order.
/// Example: `combined_element_boxes(&set)?`.
pub fn combined_element_boxes<V: BvhVector, S: BvhSet<V> + ?Sized>(
    set: &S,
) -> Result<BvhBox<V>, Failure> {
    let mut bounding = BvhBox::default();
    for index in 0..set.size() {
        bounding.combine(&set.element_box(index)?);
    }
    Ok(bounding)
}
