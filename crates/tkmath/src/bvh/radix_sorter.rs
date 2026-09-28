// Created on: 2016-04-13
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2016 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_RadixSorter.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_RadixSorter<T, N>`: Morton-code radix sort on a 1024^3 grid.
use super::bounding_box::{BvhBox, cwise_std_max};
use super::constants::THE_NODE_MIN_SIZE;
use super::scalar::{BvhScalar, int_floor};
use super::set::BvhSet;
use super::sorter::BvhSorter;
use super::vector::BvhVector;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
mod links;
mod morton;
pub use links::{BitComparator, BitPredicate, BvhEncodedLink, sort_links};
pub use morton::{THE_MORTON_LUT, encode_morton_code};

/// Voxels per axis of the Morton grid (10 bits).
const GRID_DIMENSION: i32 = 1024;

/// Radix sort of a primitive set by 10-bit Morton codes of box centers, scaled
/// to the given scene box. N = 4 ignores W; N = 2 uses Z = 0.
/// Example: `BvhRadixSorter::new(set.bounding_box()?).perform(&mut set)?`.
#[derive(Debug)]
pub struct BvhRadixSorter<V: BvhVector> {
    bounding: BvhBox<V>,
    encoded_links: Option<Array1<'static, BvhEncodedLink>>,
    is_parallel: bool,
}
impl<V: BvhVector> BvhRadixSorter<V> {
    /// Creates new BVH radix sorter for the given AABB; `BvhRadixSorter::new(scene_box)`.
    pub fn new(bounding: BvhBox<V>) -> Self {
        Self {
            bounding,
            encoded_links: None,
            is_parallel: false,
        }
    }
    /// Morton codes assigned by the last `Perform`, indexed `[start, last]`;
    /// `None` before the first sort (a null handle upstream).
    /// Example: `sorter.encoded_links().unwrap().value(0)?`.
    pub fn encoded_links(&self) -> Option<&Array1<'static, BvhEncodedLink>> {
        self.encoded_links.as_ref()
    }

    /// Step 1: one Morton code per primitive center.
    fn encode(
        &self,
        set: &dyn BvhSet<V>,
        start: i32,
        last: i32,
    ) -> Result<Array1<'static, BvhEncodedLink>, Failure> {
        let scene_min = *self.bounding.corner_min();
        let mut node_min_size = V::splat(V::Scalar::from_double(THE_NODE_MIN_SIZE));
        cwise_std_max(
            &mut node_min_size,
            &(*self.bounding.corner_max() - scene_min),
        );
        let reverse_size = V::splat(V::Scalar::from_int(GRID_DIMENSION)) / node_min_size;
        let mut links = Array1::with_bounds(start, last).map_err(|_| {
            Failure::with_type(
                "Standard_RangeError",
                &format!("BVH_RadixSorter range [{start}, {last}]: expected last >= start - 1"),
                "",
            )
        })?;
        for primitive in start..=last {
            let center = set.element_box(primitive)?.center();
            let voxel = (center - scene_min) * reverse_size;
            let code = morton_code(&voxel);
            links[primitive] = (code, primitive);
        }
        Ok(links)
    }
}

/// Clamped voxel coordinates encoded as a Morton code.
fn morton_code<V: BvhVector>(voxel: &V) -> u32 {
    let coordinate = |axis: i32| int_floor(voxel.component(axis)).clamp(0, GRID_DIMENSION - 1);
    // The 4th component is ignored; 2D points use the zero Z voxel.
    let z = if V::DIMENSION > 2 { coordinate(2) } else { 0 };
    encode_morton_code(coordinate(0) as u32, coordinate(1) as u32, z as u32)
}

impl<V: BvhVector> BvhSorter<V> for BvhRadixSorter<V> {
    fn perform_range(
        &mut self,
        set: &mut dyn BvhSet<V>,
        start: i32,
        last: i32,
    ) -> Result<(), Failure> {
        let mut links = self.encode(set, start, last)?;
        // Step 2: sort primitives by their Morton codes.
        sort_links(links.as_mut_slice(), 29, self.is_parallel);
        let mut link_map: Array1<'static, i32> = Array1::with_bounds(start, last)
            .map_err(|_| Failure::out_of_range("BVH_RadixSorter link map bounds"))?;
        for link in start..=last {
            link_map[links[link].1] = link;
        }
        self.encoded_links = Some(links);
        rearrange(set, &mut link_map, start, last)
    }
    fn is_parallel(&self) -> bool {
        self.is_parallel
    }
    fn set_parallel(&mut self, is_parallel: bool) {
        self.is_parallel = is_parallel;
    }
}

/// Step 3: in-place rearrangement following the link map's cycles.
fn rearrange<V: BvhVector>(
    set: &mut dyn BvhSet<V>,
    link_map: &mut Array1<'static, i32>,
    start: i32,
    last: i32,
) -> Result<(), Failure> {
    let mut primitive = start;
    while primitive <= last {
        let sorted = link_map[primitive];
        if primitive == sorted {
            primitive += 1;
            continue;
        }
        set.swap(primitive, sorted)?;
        let target = link_map[sorted];
        link_map[sorted] = link_map[primitive];
        link_map[primitive] = target;
    }
    Ok(())
}
