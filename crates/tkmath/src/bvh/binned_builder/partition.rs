// Rust translation of FoundationClasses/TKMath/BVH/BVH_BinnedBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::SplitPrimitives` and `BVH::BVH_AxisSelector`.
use crate::bvh::bounding_box::BvhBox;
use crate::bvh::scalar::{BvhScalar, int_floor};
use crate::bvh::set::{BvhSet, BvhSharedSet};
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// Bin of a centroid along `axis` for `bins` slices of `[min, max]`, unclamped.
trait PartitionAccess<V: BvhVector> {
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure>;
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure>;
}

struct MutableSetAccess<'a, V: BvhVector>(&'a mut dyn BvhSet<V>);

impl<V: BvhVector> PartitionAccess<V> for MutableSetAccess<'_, V> {
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.0.center(index, axis)
    }

    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.0.swap(first, second)
    }
}

struct SharedSetAccess<'a, V: BvhVector>(&'a dyn BvhSharedSet<V>);

impl<V: BvhVector> PartitionAccess<V> for SharedSetAccess<'_, V> {
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.0.center(index, axis)
    }

    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.0.swap(first, second)
    }
}

fn bin_of<V: BvhVector, A: PartitionAccess<V>>(
    set: &A,
    index: i32,
    axis: i32,
    min: V::Scalar,
    inverse_step: V::Scalar,
) -> Result<i32, Failure> {
    Ok(int_floor((set.center(index, axis)? - min) * inverse_step))
}

/// Hoare-style partition of `[begin, end]` so primitives in bins `<= bin` come
/// first; returns the first index of the right part.
/// Example: `split_primitives(set, &node_box, 0, 9, 3, 0, 32)?`.
pub fn split_primitives<V: BvhVector>(
    set: &mut dyn BvhSet<V>,
    bounding: &BvhBox<V>,
    begin: i32,
    end: i32,
    bin: i32,
    axis: i32,
    bins: i32,
) -> Result<i32, Failure> {
    split_primitives_with(
        &mut MutableSetAccess(set),
        bounding,
        begin,
        end,
        bin,
        axis,
        bins,
    )
}

pub(super) fn split_primitives_shared<V: BvhVector>(
    set: &dyn BvhSharedSet<V>,
    bounding: &BvhBox<V>,
    begin: i32,
    end: i32,
    bin: i32,
    axis: i32,
    bins: i32,
) -> Result<i32, Failure> {
    split_primitives_with(
        &mut SharedSetAccess(set),
        bounding,
        begin,
        end,
        bin,
        axis,
        bins,
    )
}

fn split_primitives_with<V: BvhVector, A: PartitionAccess<V>>(
    set: &mut A,
    bounding: &BvhBox<V>,
    begin: i32,
    end: i32,
    bin: i32,
    axis: i32,
    bins: i32,
) -> Result<i32, Failure> {
    let (min, inverse_step) = partition_scale(bounding, axis, bins);
    let (mut left, mut right) = (begin, end);
    loop {
        left = scan_left(set, left, end, bin, axis, min, inverse_step)?;
        right = scan_right(set, right, begin, bin, axis, min, inverse_step)?;
        if left <= right {
            swap_partition_items(set, &mut left, &mut right)?;
        }
        if left > right {
            return Ok(left);
        }
    }
}

fn partition_scale<V: BvhVector>(
    bounding: &BvhBox<V>,
    axis: i32,
    bins: i32,
) -> (V::Scalar, V::Scalar) {
    let min = bounding.corner_min().component(axis);
    let max = bounding.corner_max().component(axis);
    (min, V::Scalar::from_int(bins) / (max - min))
}

fn scan_left<V: BvhVector, A: PartitionAccess<V>>(
    set: &A,
    mut left: i32,
    end: i32,
    bin: i32,
    axis: i32,
    min: V::Scalar,
    inverse_step: V::Scalar,
) -> Result<i32, Failure> {
    while bin_of(set, left, axis, min, inverse_step)? <= bin && left < end {
        left += 1;
    }
    Ok(left)
}

fn scan_right<V: BvhVector, A: PartitionAccess<V>>(
    set: &A,
    mut right: i32,
    begin: i32,
    bin: i32,
    axis: i32,
    min: V::Scalar,
    inverse_step: V::Scalar,
) -> Result<i32, Failure> {
    while bin_of(set, right, axis, min, inverse_step)? > bin && right > begin {
        right -= 1;
    }
    Ok(right)
}

fn swap_partition_items<V: BvhVector, A: PartitionAccess<V>>(
    set: &mut A,
    left: &mut i32,
    right: &mut i32,
) -> Result<(), Failure> {
    if *left != *right {
        set.swap(*left, *right)?;
    }
    *left += 1;
    *right -= 1;
    Ok(())
}

/// `BVH_AxisSelector::MainAxis`: the widest axis among X, Y (and Z), with the
/// upstream strict comparisons deciding ties. Example: `main_axis(&size)`.
pub fn main_axis<V: BvhVector>(size: &V) -> i32 {
    if V::DIMENSION == 2 {
        return if size[0] > size[1] { 0 } else { 1 };
    }
    if size[1] > size[0] {
        return if size[1] > size[2] { 1 } else { 2 };
    }
    if size[2] > size[0] { 2 } else { 0 }
}
