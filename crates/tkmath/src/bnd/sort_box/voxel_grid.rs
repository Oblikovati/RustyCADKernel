// Created on: 1992-11-24
// Created by: Didier PIFFAULT
// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
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
