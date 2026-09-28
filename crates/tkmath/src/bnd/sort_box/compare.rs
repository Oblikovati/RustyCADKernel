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

use super::super::BoundingBox3;
use super::{BoundSortBox, range_failure, uninitialized};
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::Failure;

/// Bit set in the per-box mask once the box shares an X slice with the query.
const OCCUPIED_X: u8 = 0b01;
/// Bit set once the box also shares a Y slice.
const OCCUPIED_Y: u8 = 0b10;
/// Both bits: the box becomes a candidate when it also shares a Z slice.
const OCCUPIED_XY: u8 = 0b11;

impl BoundSortBox {
    /// `Compare(Bnd_Box)`: the indices of the boxes the query touches or contains.
    ///
    /// Large boxes come first in insertion order, then grid boxes in the order of the Z
    /// slices of the query (each box once, at its first shared Z slice). A void query or one
    /// outside the enclosing box gives an empty list. Example: `sorter.compare_box(&query)?`.
    pub fn compare_box(&mut self, query: &BoundingBox3) -> Result<&List<i32>, Failure> {
        self.last_result.clear();
        if query.is_void() || query.is_out_box(&self.enclosing) {
            return Ok(&self.last_result);
        }
        let boxes = self.boxes.as_ref().ok_or_else(uninitialized)?;
        for index in &self.large_boxes {
            if !stored(boxes, *index)?.is_out_box(query) {
                self.last_result.append(*index);
            }
        }
        let voxels = self.bounding_voxels(query)?;
        let grid = self.voxel_grid.as_ref().ok_or_else(uninitialized)?;
        let lower = boxes.lower();
        let mut masks = vec![0u8; boxes.len()];
        let mut mark = |axis: usize, bit: u8| {
            for layer in voxels[axis]..=voxels[axis + 3] {
                for index in grid.slice(axis, layer) {
                    masks[(index - lower) as usize] |= bit;
                }
            }
        };
        mark(0, OCCUPIED_X);
        mark(1, OCCUPIED_Y);
        for layer in voxels[2]..=voxels[5] {
            for index in grid.slice(2, layer) {
                let mask = &mut masks[(index - lower) as usize];
                if *mask != OCCUPIED_XY {
                    continue;
                }
                // Clearing the mask reports each box once.
                *mask = 0;
                if !stored(boxes, *index)?.is_out_box(query) {
                    self.last_result.append(*index);
                }
            }
        }
        Ok(&self.last_result)
    }
}

fn stored<'b>(
    boxes: &'b Array1<'static, BoundingBox3>,
    index: i32,
) -> Result<&'b BoundingBox3, Failure> {
    boxes
        .value(index)
        .map_err(|error| range_failure("Compare", error))
}
