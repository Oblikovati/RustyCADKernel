// Rust translation of FoundationClasses/TKMath/BVH/BVH_QuickSorter.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_QuickSorter<T, N>`: centroid sort along one axis with `std::sort`.
use super::set::{BvhSet, BvhSharedSet};
use super::sorter::BvhSorter;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::stl;
use std::marker::PhantomData;

/// Centroid-based sort of an abstract set along X (0), Y (1) or Z (2).
///
/// The order of equal centroids follows libstdc++'s unstable introsort, as in
/// the GCC oracle. Example: `BvhQuickSorter::<BvhVec3d>::new(1).perform(&mut set)?`.
#[derive(Clone, Copy, Debug)]
pub struct BvhQuickSorter<V: BvhVector> {
    axis: i32,
    is_parallel: bool,
    vector: PhantomData<V>,
}
impl<V: BvhVector> Default for BvhQuickSorter<V> {
    fn default() -> Self {
        Self::new(0)
    }
}
impl<V: BvhVector> BvhQuickSorter<V> {
    /// Creates new BVH quick sorter for the given axis (0 by default upstream);
    /// `BvhQuickSorter::<BvhVec3d>::new(0)`.
    pub const fn new(axis: i32) -> Self {
        Self {
            axis,
            is_parallel: false,
            vector: PhantomData,
        }
    }
    /// Axis used to arrange the primitives; `sorter.axis()`.
    pub const fn axis(&self) -> i32 {
        self.axis
    }
}
impl<V: BvhVector> BvhSorter<V> for BvhQuickSorter<V> {
    /// Sorts an index permutation by centroid, then applies it with cycle swaps.
    ///
    /// Centroids are read once before sorting; `Center` is a const query, so
    /// the comparisons see the same values the native comparator recomputes.
    fn perform_range(
        &mut self,
        set: &mut dyn BvhSet<V>,
        start: i32,
        last: i32,
    ) -> Result<(), Failure> {
        sort_range(&mut MutableSortAccess(set), self.axis, start, last)
    }
    fn is_parallel(&self) -> bool {
        self.is_parallel
    }
    fn set_parallel(&mut self, is_parallel: bool) {
        self.is_parallel = is_parallel;
    }
}

/// Sorts a shared set by centroid through its synchronized read/swap methods.
pub(crate) fn sort_range_shared<V: BvhVector>(
    set: &dyn BvhSharedSet<V>,
    axis: i32,
    start: i32,
    last: i32,
) -> Result<(), Failure> {
    sort_range(&mut SharedSortAccess(set), axis, start, last)
}

trait SortAccess<V: BvhVector> {
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure>;
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure>;
}

struct MutableSortAccess<'a, V: BvhVector>(&'a mut dyn BvhSet<V>);

impl<V: BvhVector> SortAccess<V> for MutableSortAccess<'_, V> {
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.0.center(index, axis)
    }

    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.0.swap(first, second)
    }
}

struct SharedSortAccess<'a, V: BvhVector>(&'a dyn BvhSharedSet<V>);

impl<V: BvhVector> SortAccess<V> for SharedSortAccess<'_, V> {
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.0.center(index, axis)
    }

    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.0.swap(first, second)
    }
}

fn sort_range<V: BvhVector>(
    set: &mut dyn SortAccess<V>,
    axis: i32,
    start: i32,
    last: i32,
) -> Result<(), Failure> {
    let size = last - start + 1;
    if size <= 1 {
        return Ok(());
    }
    let centers = (0..size)
        .map(|offset| set.center(start + offset, axis))
        .collect::<Result<Vec<V::Scalar>, Failure>>()?;
    let mut indices: Vec<usize> = (0..size as usize).collect();
    stl::sort_by(&mut indices, |first, second| {
        centers[*first] < centers[*second]
    });
    apply_permutation(set, start, &indices)
}

/// Moves each sorted element to its destination in the native cycle order.
fn apply_permutation<V: BvhVector>(
    set: &mut dyn SortAccess<V>,
    start: i32,
    indices: &[usize],
) -> Result<(), Failure> {
    let mut inverse = vec![0; indices.len()];
    for (position, index) in indices.iter().enumerate() {
        inverse[*index] = position;
    }
    for position in 0..inverse.len() {
        while inverse[position] != position {
            let target = inverse[position];
            set.swap(start + position as i32, start + target as i32)?;
            inverse.swap(position, target);
        }
    }
    Ok(())
}
