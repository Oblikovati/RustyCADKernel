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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_PrimitiveSet.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_PrimitiveSet<T, N>`: a set of primitives of one object with its own BVH.
use super::binned_builder::BvhBinnedBuilder;
use super::bounding_box::BvhBox;
use super::builder::BvhBuilder;
use super::constants::{LEAF_NODE_SIZE_DEFAULT, MAX_TREE_DEPTH, NB_BINS_BEST};
use super::object::BvhObjectTransient;
use super::set::{BvhSet, combined_element_boxes};
use super::tree::BvhTree;
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::cell::Cell;
use std::sync::Arc;

/// Shared handle of a BVH builder; `None` is a null handle.
pub type BvhBuilderHandle<V> = Option<Arc<dyn BvhBuilder<V>>>;

/// `BVH_PrimitiveSet3d`: state of double-precision 3D primitive sets.
pub type BvhPrimitiveSet3d = BvhPrimitiveSet<crate::bvh::BvhVec3d>;

/// State of `BVH_PrimitiveSet`: object flags, the bottom-level tree, its
/// builder and the cached set box. Concrete sets embed it and implement
/// [`BvhPrimitiveSetAccess`]. Example: `BvhPrimitiveSet::<Vec3<f64>>::default()`.
#[derive(Debug)]
pub struct BvhPrimitiveSet<V: BvhVector> {
    object: BvhObjectTransient,
    bvh: BvhTree<V>,
    builder: BvhBuilderHandle<V>,
    cached_box: Cell<BvhBox<V>>,
}
impl<V: BvhVector> Default for BvhPrimitiveSet<V> {
    /// Uses the default builder: binned SAH split with 48 bins, 5 primitives
    /// per leaf and depth 32.
    fn default() -> Self {
        let binned = BvhBinnedBuilder::<V, NB_BINS_BEST>::new(
            LEAF_NODE_SIZE_DEFAULT,
            MAX_TREE_DEPTH,
            false,
            1,
        );
        Self::new(Some(Arc::new(binned)))
    }
}
impl<V: BvhVector> BvhPrimitiveSet<V> {
    /// `BVH_PrimitiveSet::MaxTreeDepth`.
    pub const MAX_TREE_DEPTH: i32 = MAX_TREE_DEPTH;
    /// Creates set state with the given builder; `BvhPrimitiveSet::new(Some(builder))`.
    pub fn new(builder: BvhBuilderHandle<V>) -> Self {
        Self {
            object: BvhObjectTransient::default(),
            bvh: BvhTree::default(),
            builder,
            cached_box: Cell::new(BvhBox::default()),
        }
    }
    /// Borrows the `BVH_ObjectTransient` data; `state.object_transient().is_dirty`.
    pub fn object_transient(&self) -> &BvhObjectTransient {
        &self.object
    }
    /// Mutably borrows the `BVH_ObjectTransient` data.
    pub fn object_transient_mut(&mut self) -> &mut BvhObjectTransient {
        &mut self.object
    }
}

/// The non-virtual behavior of `BVH_PrimitiveSet` for a concrete set.
///
/// Implementors must also forward [`BvhSet::bounding_box`] and
/// `BvhObject::object_box` to [`BvhPrimitiveSetAccess::cached_box`]: upstream
/// one override serves both base classes. Example: `triangulation.bvh()?`.
pub trait BvhPrimitiveSetAccess<V: BvhVector>: BvhSet<V> {
    /// Borrows the embedded primitive-set state.
    fn primitive_set(&self) -> &BvhPrimitiveSet<V>;
    /// Mutably borrows the embedded primitive-set state.
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<V>;

    /// AABB of the set: recomputed from the elements while dirty, otherwise the
    /// cached box (invalid until the first dirty evaluation); `set.cached_box()?`.
    fn cached_box(&self) -> Result<BvhBox<V>, Failure> {
        let state = self.primitive_set();
        if state.object.is_dirty {
            state.cached_box.set(combined_element_boxes(self)?);
        }
        Ok(state.cached_box.get())
    }
    /// Returns the BVH tree, building it first while dirty; `set.bvh()?.length()`.
    fn bvh(&mut self) -> Result<&BvhTree<V>, Failure>
    where
        Self: Sized,
    {
        if self.primitive_set().object.is_dirty {
            self.update()?;
        }
        Ok(&self.primitive_set().bvh)
    }
    /// Returns the method (builder) used to construct BVH; `set.builder()`.
    fn builder(&self) -> &BvhBuilderHandle<V> {
        &self.primitive_set().builder
    }
    /// Sets the method (builder) used to construct BVH; `set.set_builder(Some(b))`.
    fn set_builder(&mut self, builder: BvhBuilderHandle<V>) {
        self.primitive_set_mut().builder = builder;
    }
    /// Rebuilds the BVH while dirty, then clears the dirty flag; a null builder
    /// fails as `Standard_NullObject`. Example: `set.update()?`.
    fn update(&mut self) -> Result<(), Failure>
    where
        Self: Sized,
    {
        if !self.primitive_set().object.is_dirty {
            return Ok(());
        }
        let builder = self.primitive_set().builder.clone().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BVH_PrimitiveSet builder is null",
                "",
            )
        })?;
        let bounding = self.cached_box()?;
        // The builder needs the set and the tree mutably at once: move the tree out.
        let mut bvh = std::mem::take(&mut self.primitive_set_mut().bvh);
        let built = builder.build(self, &mut bvh, &bounding);
        self.primitive_set_mut().bvh = bvh;
        built?;
        self.primitive_set_mut().object.is_dirty = false;
        Ok(())
    }
    /// `BVH()` for traversals: updates the tree while dirty, then moves it out;
    /// the body of every primitive set's `BvhTreeOwner::take_built_bvh`.
    /// Example: `let tree = set.take_primitive_bvh()?;`.
    fn take_primitive_bvh(&mut self) -> Result<BvhTree<V>, Failure>
    where
        Self: Sized,
    {
        self.update()?;
        Ok(std::mem::take(&mut self.primitive_set_mut().bvh))
    }
    /// Moves a tree taken by `take_primitive_bvh` back; `set.restore_primitive_bvh(tree)`.
    fn restore_primitive_bvh(&mut self, bvh: BvhTree<V>) {
        self.primitive_set_mut().bvh = bvh;
    }
}

/// Implements `BvhTreeOwner` of a primitive set through its embedded state.
macro_rules! primitive_tree_owner {
    ($set:ident $(, $parameter:ident: $bound:path)*) => {
        impl<V: crate::bvh::BvhVector $(, $parameter: $bound)*>
            crate::bvh::tree_owner::BvhTreeOwner<V> for $set<V $(, $parameter)*>
        {
            fn take_built_bvh(
                &mut self,
            ) -> Result<crate::bvh::BvhTree<V>, rustycad_tkernel::standard::Failure> {
                crate::bvh::BvhPrimitiveSetAccess::take_primitive_bvh(self)
            }
            fn restore_bvh(&mut self, bvh: crate::bvh::BvhTree<V>) {
                crate::bvh::BvhPrimitiveSetAccess::restore_primitive_bvh(self, bvh);
            }
        }
    };
}
pub(crate) use primitive_tree_owner;
