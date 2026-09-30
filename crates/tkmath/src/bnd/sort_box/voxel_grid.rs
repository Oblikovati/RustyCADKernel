// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_BoundSortBox.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bnd_VoxelGrid`: per-axis slices listing the boxes that occupy each voxel layer.

/// Voxel indices of a box in the order `[minX, minY, minZ, maxX, maxY, maxZ]`.
pub(super) type VoxelBox = [i32; 6];

/// Three arrays of `resolution` slices; slice `i` of an axis lists, in insertion order,
/// every box whose voxel range on that axis contains `i`.
///
/// The native per-slice `NCollection_DynamicArray` with an incremental allocator only
/// changes memory layout, so plain vectors keep the same contents and order.
/// Example: `VoxelGrid::new(8).add_box(3, [0, 0, 0, 1, 1, 1])`.
#[derive(Clone, Debug)]
pub(super) struct VoxelGrid {
    slices: [Vec<Vec<i32>>; 3],
}

impl VoxelGrid {
    /// Creates `resolution` empty slices per axis.
    pub(super) fn new(resolution: i32) -> Self {
        let layers = usize::try_from(resolution).unwrap_or(0);
        Self {
            slices: std::array::from_fn(|_| vec![Vec::new(); layers]),
        }
    }
    /// `AddBox`: appends the box index to every slice of its range on each axis; an empty
    /// range (min > max) appends nothing.
    pub(super) fn add_box(&mut self, index: i32, voxels: VoxelBox) {
        for axis in 0..3 {
            for layer in voxels[axis]..=voxels[axis + 3] {
                self.slices[axis][layer as usize].push(index);
            }
        }
    }
    /// `GetSliceX/Y/Z`: the boxes of one slice (empty when none).
    pub(super) fn slice(&self, axis: usize, layer: i32) -> &[i32] {
        &self.slices[axis][layer as usize]
    }
}
