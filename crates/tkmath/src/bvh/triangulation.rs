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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Triangulation.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Triangulation<T, N>`: triangles as an example of a BVH primitive set.
use super::array;
use super::bounding_box::{BvhBox, cwise_std_max, cwise_std_min};
use super::object::{BvhObject, BvhObjectTransient};
use super::primitive_set::{BvhBuilderHandle, BvhPrimitiveSet, BvhPrimitiveSetAccess};
use super::scalar::BvhScalar;
use super::set::BvhSet;
use super::vector::{BvhArray4i, BvhVector};
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::standard::Failure;

/// Vertex coordinates plus one `(i0, i1, i2, unused)` record per triangle.
/// Example: `array::append(&mut triangulation.elements, Vec4::new(0, 1, 2, 0))`.
#[derive(Debug, Default)]
pub struct BvhTriangulation<V: BvhVector> {
    /// Array of vertex coordinates.
    pub vertices: LinearVector<V>,
    /// Array of indices of triangle vertices.
    pub elements: BvhArray4i,
    primitive_set: BvhPrimitiveSet<V>,
}
impl<V: BvhVector> BvhTriangulation<V> {
    /// Creates empty triangulation with the given builder; `BvhTriangulation::new(Some(b))`.
    pub fn new(builder: BvhBuilderHandle<V>) -> Self {
        Self {
            vertices: LinearVector::default(),
            elements: LinearVector::default(),
            primitive_set: BvhPrimitiveSet::new(builder),
        }
    }
    /// The three vertices of a triangle, checked like `BVH::Array::Value`.
    fn triangle(&self, index: i32) -> Result<[V; 3], Failure> {
        let record = array::value(&self.elements, index)?;
        Ok([
            *array::value(&self.vertices, record.x)?,
            *array::value(&self.vertices, record.y)?,
            *array::value(&self.vertices, record.z)?,
        ])
    }
}
impl<V: BvhVector> BvhSet<V> for BvhTriangulation<V> {
    /// Returns total number of triangles.
    fn size(&self) -> i32 {
        array::size(&self.elements)
    }
    /// Returns AABB of the given triangle (`std::min`/`std::max` per component).
    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        let [first, second, third] = self.triangle(index)?;
        let (mut min, mut max) = (first, first);
        cwise_std_min(&mut min, &second);
        cwise_std_min(&mut min, &third);
        cwise_std_max(&mut max, &second);
        cwise_std_max(&mut max, &third);
        Ok(BvhBox::from_corners(min, max))
    }
    /// Returns centroid position along the given axis: the component sum times
    /// `static_cast<T>(1.0 / 3.0)`.
    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        let [first, second, third] = self.triangle(index)?;
        let sum = first.component(axis) + second.component(axis) + third.component(axis);
        Ok(sum * V::Scalar::from_double(1.0 / 3.0))
    }
    /// Performs transposing the two given triangles in the set.
    fn swap(&mut self, first: i32, second: i32) -> Result<(), Failure> {
        let first_record = *array::change_value(&mut self.elements, first)?;
        let second_record = std::mem::replace(
            array::change_value(&mut self.elements, second)?,
            first_record,
        );
        *array::change_value(&mut self.elements, first)? = second_record;
        Ok(())
    }
    /// Returns the cached AABB of the whole triangulation.
    fn bounding_box(&self) -> Result<BvhBox<V>, Failure> {
        self.cached_box()
    }
}
impl<V: BvhVector> BvhPrimitiveSetAccess<V> for BvhTriangulation<V> {
    fn primitive_set(&self) -> &BvhPrimitiveSet<V> {
        &self.primitive_set
    }
    fn primitive_set_mut(&mut self) -> &mut BvhPrimitiveSet<V> {
        &mut self.primitive_set
    }
}
impl<V: BvhVector> BvhObject<V> for BvhTriangulation<V> {
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
