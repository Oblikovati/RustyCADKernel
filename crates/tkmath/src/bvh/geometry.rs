// Created on: 2013-12-20
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Geometry.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Geometry<T, N>`: an object set with its own top-level BVH.
use super::binned_builder::BvhBinnedBuilder;
use super::bounding_box::BvhBox;
use super::constants::{LEAF_NODE_SIZE_SINGLE, MAX_TREE_DEPTH, NB_BINS_OPTIMAL};
use super::object_set::{BvhObjectList, BvhObjectSet};
use super::primitive_set::BvhBuilderHandle;
use super::set::{BvhSet, combined_element_boxes};
use super::tree::BvhTree;
use super::tree_owner::BvhTreeOwner;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::cell::Cell;
use std::sync::Arc;

/// Set of geometric objects organized with a top-level BVH.
///
/// The geometry has its own dirty flag, which starts false: until
/// `mark_dirty`, the box is invalid and the tree empty, as upstream.
/// Example: `geometry.objects_mut().append(Some(Box::new(mesh))); geometry.mark_dirty();`.
#[derive(Debug)]
pub struct BvhGeometry<V: BvhVector> {
    object_set: BvhObjectSet<V>,
    is_dirty: bool,
    bvh: BvhTree<V>,
    builder: BvhBuilderHandle<V>,
    cached_box: Cell<BvhBox<V>>,
}
impl<V: BvhVector> Default for BvhGeometry<V> {
    /// Uses a binned SAH builder with 32 bins and one object per leaf.
    fn default() -> Self {
        let binned = BvhBinnedBuilder::<V, NB_BINS_OPTIMAL>::new(
            LEAF_NODE_SIZE_SINGLE,
            MAX_TREE_DEPTH,
            false,
            1,
        );
        Self::new(Some(Arc::new(binned)))
    }
}
impl<V: BvhVector> BvhGeometry<V> {
    /// Creates uninitialized BVH geometry with the given builder;
    /// `BvhGeometry::<BvhVec3d>::new(Some(builder))`.
    pub fn new(builder: BvhBuilderHandle<V>) -> Self {
        Self {
            object_set: BvhObjectSet::default(),
            is_dirty: false,
            bvh: BvhTree::default(),
            builder,
            cached_box: Cell::new(BvhBox::default()),
        }
    }
    /// Returns TRUE if geometry state should be updated; `geometry.is_dirty()`.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    /// Marks geometry as outdated; `geometry.mark_dirty()`.
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
    /// The inherited object set; `geometry.object_set().size()`.
    pub fn object_set(&self) -> &BvhObjectSet<V> {
        &self.object_set
    }
    /// The inherited object set, mutably; `geometry.object_set_mut().clear()`.
    pub fn object_set_mut(&mut self) -> &mut BvhObjectSet<V> {
        &mut self.object_set
    }
    /// Returns the array of geometric objects; `geometry.objects().len()`.
    pub fn objects(&self) -> &BvhObjectList<V> {
        self.object_set.objects()
    }
    /// Returns the mutable array of geometric objects; `geometry.objects_mut()`.
    pub fn objects_mut(&mut self) -> &mut BvhObjectList<V> {
        self.object_set.objects_mut()
    }
    /// Removes all geometric objects (`BVH_ObjectSet::Clear`); `geometry.clear()`.
    pub fn clear(&mut self) {
        self.object_set.clear();
    }
    /// AABB of the whole geometry: recomputed from the object boxes while
    /// dirty, otherwise the cached box; `geometry.cached_box()?`.
    pub fn cached_box(&self) -> Result<BvhBox<V>, Failure> {
        if self.is_dirty {
            self.cached_box
                .set(combined_element_boxes(&self.object_set)?);
        }
        Ok(self.cached_box.get())
    }
    /// Returns the BVH tree, building it first while dirty; `geometry.bvh()?.length()`.
    pub fn bvh(&mut self) -> Result<&BvhTree<V>, Failure> {
        if self.is_dirty {
            self.update()?;
        }
        Ok(&self.bvh)
    }
    /// Returns the method (builder) used to construct BVH; `geometry.builder()`.
    pub fn builder(&self) -> &BvhBuilderHandle<V> {
        &self.builder
    }
    /// Sets the method (builder) used to construct BVH; `geometry.set_builder(Some(b))`.
    pub fn set_builder(&mut self, builder: BvhBuilderHandle<V>) {
        self.builder = builder;
    }
    /// Rebuilds the top-level BVH while dirty, then clears the flag; a null
    /// builder fails as `Standard_NullObject`. Example: `geometry.update()?`.
    pub fn update(&mut self) -> Result<(), Failure> {
        if !self.is_dirty {
            return Ok(());
        }
        let builder = self.builder.clone().ok_or_else(|| {
            Failure::with_type("Standard_NullObject", "BVH_Geometry builder is null", "")
        })?;
        let bounding = self.cached_box()?;
        // The builder needs the set and the tree mutably at once: move the tree out.
        let mut bvh = std::mem::take(&mut self.bvh);
        let built = builder.build(self, &mut bvh, &bounding);
        self.bvh = bvh;
        built?;
        self.is_dirty = false;
        Ok(())
    }
}
impl<V: BvhVector> BvhSet<V> for BvhGeometry<V> {
    fn size(&self) -> i32 {
        self.object_set.size()
    }
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.object_set.element_box(index)
    }
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.object_set.center(index, axis)
    }
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        self.object_set.swap(first, second)
    }
    /// The cached whole-geometry box (`Box()` override).
    fn bounding_box(&self) -> Result<BvhBox<V>, Failure> {
        self.cached_box()
    }
}
impl<V: BvhVector> BvhTreeOwner<V> for BvhGeometry<V> {
    fn take_built_bvh(&mut self) -> Result<BvhTree<V>, Failure> {
        if self.is_dirty {
            self.update()?;
        }
        Ok(std::mem::take(&mut self.bvh))
    }
    fn restore_bvh(&mut self, bvh: BvhTree<V>) {
        self.bvh = bvh;
    }
}
