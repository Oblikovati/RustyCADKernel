// Rust translation of FoundationClasses/TKMath/BVH/BVH_BinnedBuilder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::SplitPrimitives` and `BVH::BVH_AxisSelector`.
use crate::bvh::bounding_box::BvhBox;
use crate::bvh::scalar::{BvhScalar, int_floor};
use crate::bvh::set::BvhSet;
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::Failure;

/// Bin of a centroid along `axis` for `bins` slices of `[min, max]`, unclamped.
fn bin_of<V: BvhVector>(
    set: &dyn BvhSet<V>,
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
    let min = bounding.corner_min().component(axis);
    let max = bounding.corner_max().component(axis);
    let inverse_step = V::Scalar::from_int(bins) / (max - min);
    let (mut left, mut right) = (begin, end);
    loop {
        while bin_of(set, left, axis, min, inverse_step)? <= bin && left < end {
            left += 1;
        }
        while bin_of(set, right, axis, min, inverse_step)? > bin && right > begin {
            right -= 1;
        }
        if left <= right {
            if left != right {
                set.swap(left, right)?;
            }
            left += 1;
            right -= 1;
        }
        if left > right {
            return Ok(left);
        }
    }
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
