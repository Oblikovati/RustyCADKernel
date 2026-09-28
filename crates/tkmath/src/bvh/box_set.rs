// Created by: Eugeny MALTCHIKOV
// Created on: 2019-04-17
// Copyright (c) 2019 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_BoxSet.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_BoxSet<T, N, DataType>`: elements added together with their boxes.
use super::array;
use super::bounding_box::BvhBox;
use super::object::{BvhObject, BvhObjectTransient};
use super::primitive_set::{BvhBuilderHandle, BvhPrimitiveSet, BvhPrimitiveSetAccess};
use super::set::BvhSet;
use super::vector::BvhVector;
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::standard::Failure;
mod indexed;
pub use indexed::BvhIndexedBoxSet;

/// Primitive set of user elements, each bounded by the box given to `add`.
///
/// Upstream `Center` returns `double`, so only `double` sets compile there; the
/// Rust set is generic and returns the scalar center unchanged.
/// Example: `let mut set = BvhBoxSet::<BvhVec3d>::default(); set.add(7, bounding);`.
#[derive(Debug)]
pub struct BvhBoxSet<V: BvhVector, D = i32> {
    elements: LinearVector<D>,
    boxes: LinearVector<BvhBox<V>>,
    primitive_set: BvhPrimitiveSet<V>,
}
impl<V: BvhVector, D> Default for BvhBoxSet<V, D> {
    /// Uses the default binned builder of `BVH_PrimitiveSet`.
    fn default() -> Self {
        Self::with_state(BvhPrimitiveSet::default())
    }
}
impl<V: BvhVector, D> BvhBoxSet<V, D> {
    /// Creates an empty set with a custom builder; `BvhBoxSet::<V>::new(Some(builder))`.
    pub fn new(builder: BvhBuilderHandle<V>) -> Self {
        Self::with_state(BvhPrimitiveSet::new(builder))
    }
    fn with_state(primitive_set: BvhPrimitiveSet<V>) -> Self {
        Self {
            elements: LinearVector::default(),
            boxes: LinearVector::default(),
            primitive_set,
        }
    }
    /// Reserves room for the expected number of elements; `set.set_size(100)`.
    pub fn set_size(&mut self, size: usize) {
        self.elements.reserve(size);
        self.boxes.reserve(size);
    }
    /// Appends an element with its box and marks the set dirty; `set.add(3, bounding)`.
    pub fn add(&mut self, element: D, bounding: BvhBox<V>) {
        self.elements.append(element);
        self.boxes.append(bounding);
        self.primitive_set.object_transient_mut().is_dirty = true;
    }
    /// Removes every element and box and marks the set dirty; `set.clear()`.
    pub fn clear(&mut self) {
        self.elements.clear(false);
        self.boxes.clear(false);
        self.primitive_set.object_transient_mut().is_dirty = true;
    }
}
impl<V: BvhVector, D: Send> BvhBoxSet<V, D> {
    /// Builds the BVH when dirty (`Update`); `set.build()?`.
    pub fn build(&mut self) -> Result<(), Failure> {
        self.update()
    }
}
impl<V: BvhVector, D: Clone> BvhBoxSet<V, D> {
    /// Returns a copy of the element at `index`; `set.element(0)?`.
    pub fn element(&self, index: i32) -> Result<D, Failure> {
        array::value(&self.elements, index).cloned()
    }
}

/// `std::swap(vector[first], vector[second])` with both indices checked first.
fn swap_items<E>(items: &mut LinearVector<E>, first: i32, second: i32) -> Result<(), Failure> {
    array::value(items, first)?;
    array::value(items, second)?;
    items.as_mut_slice().swap(first as usize, second as usize);
    Ok(())
}

impl<V: BvhVector, D: Send> BvhSet<V> for BvhBoxSet<V, D> {
    /// Returns the number of boxes.
    fn size(&self) -> i32 {
        array::size(&self.boxes)
    }
    /// Returns the box added with the element at `index`.
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        array::value(&self.boxes, index).copied()
    }
    /// Returns the box center along `axis`, `Box(index).Center(axis)`.
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        Ok(self.element_box(index)?.center_axis(axis))
    }
    /// Swaps both the elements and the boxes.
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        swap_items(&mut self.elements, first, second)?;
        swap_items(&mut self.boxes, first, second)
    }
    fn bounding_box(&self) -> Result<BvhBox<V>, Failure> {
        self.cached_box()
    }
}
impl<V: BvhVector, D: Send> BvhPrimitiveSetAccess<V> for BvhBoxSet<V, D> {
    fn primitive_set(&self) -> &BvhPrimitiveSet<V> {
        &self.primitive_set
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<V> {
        &mut self.primitive_set
    }
}
impl<V: BvhVector, D: Send> BvhObject<V> for BvhBoxSet<V, D> {
    fn object_transient(&self) -> &BvhObjectTransient {
        self.primitive_set.object_transient()
    }
    fn object_transient_mut(&mut self) -> &mut BvhObjectTransient {
        self.primitive_set.object_transient_mut()
    }
    fn object_box(&self) -> Result<BvhBox<V>, Failure> {
        self.cached_box()
    }
}
crate::bvh::primitive_set::primitive_tree_owner!(BvhBoxSet, D: Send);
