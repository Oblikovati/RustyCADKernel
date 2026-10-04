// Rust translation of FoundationClasses/TKMath/BVH/BVH_Set.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Set<T, N>`: the minimal geometry interface needed to construct a BVH.
use super::bounding_box::BvhBox;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// Set of abstract entities bounded by BVH boxes.
///
/// Builders and sorters permute the set through [`BvhSet::swap`]. Existing
/// mutable-only queue builds serialize node callbacks safely.
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

/// Thread-safe set operations required by the opt-in parallel builder.
/// Implementations must make each read/swap linearizable; disjoint node ranges
/// may be processed at the same time. `BvhMutexSet` adapts an existing set.
pub trait BvhSharedSet<V: BvhVector>: Send + Sync {
    /// Returns total number of objects.
    fn size(&self) -> i32;
    /// Returns AABB of the given object.
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure>;
    /// Returns centroid position along the given axis.
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure>;
    /// Swaps entries through synchronized shared access.
    fn swap(&self, first: i32, second: i32) -> Result<(), Failure>;
}

pub(crate) trait BvhSetRead<V: BvhVector> {
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure>;
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure>;
}

pub(crate) struct MutableSetRead<'a, V: BvhVector>(&'a dyn BvhSet<V>);

impl<V: BvhVector> MutableSetRead<'_, V> {
    pub(crate) fn new(set: &dyn BvhSet<V>) -> MutableSetRead<'_, V> {
        MutableSetRead(set)
    }
}

impl<V: BvhVector> BvhSetRead<V> for MutableSetRead<'_, V> {
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.0.element_box(index)
    }

    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.0.center(index, axis)
    }
}

pub(crate) struct SharedSetRead<'a, V: BvhVector>(&'a dyn BvhSharedSet<V>);

impl<V: BvhVector> SharedSetRead<'_, V> {
    pub(crate) fn new(set: &dyn BvhSharedSet<V>) -> SharedSetRead<'_, V> {
        SharedSetRead(set)
    }
}

impl<V: BvhVector> BvhSetRead<V> for SharedSetRead<'_, V> {
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.0.element_box(index)
    }

    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.0.center(index, axis)
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
