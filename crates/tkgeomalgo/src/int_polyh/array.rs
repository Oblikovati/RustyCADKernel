// Created on: 2012-01-26
// Created by: Peter KURNEV
// Copyright (c) 2012-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_Array.hxx and the
// IntPolyh_ArrayOf*.hxx typedefs at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_Array`: the dynamic arrays of the triangulation (points, edges, triangles,
//! section lines, tangent zones, points with normals), 0-based, with an allocated size
//! and a number of stored items (docs/geomalgo-design.md section 12).
use super::edge::IntPolyhEdge;
use super::point::IntPolyhPoint;
use super::section_line::IntPolyhSectionLine;
use super::start_point::IntPolyhStartPoint;
use super::triangle::IntPolyhTriangle;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::fmt;
use std::ops::{Index, IndexMut};

/// `IntPolyh_Array<Type>`: the storage of an `NCollection_DynamicArray`, whose `Init(N)`
/// sets the item `N` (default-constructing the ones before it), the number of allocated
/// items (`GetN`) and the number of stored ones (`NbItems`).
///
/// ```
/// use rustycad_tkgeomalgo::int_polyh::IntPolyhArray;
/// let mut a: IntPolyhArray<i32> = IntPolyhArray::with_size(3, 256);
/// a[0] = 7;
/// a.increment_nb_items();
/// assert_eq!((a.nb_items(), a.get_n(), *a.value(0).unwrap()), (1, 3, 7));
/// ```
#[derive(Clone, Debug)]
pub struct IntPolyhArray<T> {
    nb_allocated: i32,
    nb_items: i32,
    increment: i32,
    vector: Vec<T>,
}

impl<T: Default + Clone> Default for IntPolyhArray<T> {
    fn default() -> Self {
        Self::new(256)
    }
}

impl<T: Default + Clone> IntPolyhArray<T> {
    /// `IntPolyh_Array(aIncrement = 256)`: no item allocated. `IntPolyhArray::<i32>::new(256)`.
    pub fn new(increment: i32) -> Self {
        Self {
            nb_allocated: 0,
            nb_items: 0,
            increment,
            vector: Vec::new(),
        }
    }

    /// `IntPolyh_Array(aN, aIncrement = 256)`: `aN` items allocated.
    /// `IntPolyhArray::<i32>::with_size(10, 256).get_n() == 10`.
    pub fn with_size(n: i32, increment: i32) -> Self {
        let mut array: Self = Self::new(increment);
        array.init(n);
        array
    }

    /// `Copy(aOther)` (also `operator=`): the items and counts of `other`; the increment
    /// stays this array's own, as natively. `a.copy_from(&b)`.
    pub fn copy_from(&mut self, other: &Self) -> &mut Self {
        self.vector.clear();
        self.init(other.nb_allocated);
        self.vector.clone_from(&other.vector);
        self.nb_items = other.nb_items;
        self
    }

    /// `Init(aN)`: allocates `aN` items. The storage sets its item `aN` to a default one
    /// (growing to `aN + 1` items), so an existing item `aN` is reset. `a.init(10)`.
    pub fn init(&mut self, n: i32) {
        // A negative index is converted to a huge size_t natively (undefined): nothing is
        // stored here.
        if let Ok(index) = usize::try_from(n) {
            if index < self.vector.len() {
                self.vector[index] = T::default();
            } else {
                self.vector.resize(index + 1, T::default());
            }
        }
        self.nb_allocated = n;
    }

    /// `IncrementNbItems()`: one more stored item, growing the allocation by the
    /// increment when it is full. `a.increment_nb_items()`.
    pub fn increment_nb_items(&mut self) {
        self.nb_items += 1;
        if self.nb_items >= self.nb_allocated {
            self.init(self.nb_allocated + self.increment);
        }
    }
}

impl<T> IntPolyhArray<T> {
    /// `GetN()`: the number of allocated items. `a.get_n()`.
    pub fn get_n(&self) -> i32 {
        self.nb_allocated
    }

    /// `NbItems()`: the number of stored items. `a.nb_items()`.
    pub fn nb_items(&self) -> i32 {
        self.nb_items
    }

    /// `SetNbItems(aNb)`. `a.set_nb_items(3)`.
    pub fn set_nb_items(&mut self, nb: i32) {
        self.nb_items = nb;
    }

    /// `Value(aIndex)` (also `operator[] const`): the item of the storage at `index`.
    /// Natively the index is not checked (undefined past the storage); here that is a
    /// Standard_OutOfRange. `a.value(0)?`.
    pub fn value(&self, index: i32) -> Result<&T, Failure> {
        let len: usize = self.vector.len();
        usize::try_from(index)
            .ok()
            .and_then(|i| self.vector.get(i))
            .ok_or_else(|| storage_error(index, len))
    }

    /// `ChangeValue(aIndex)` (also `operator[]`); Standard_OutOfRange past the storage.
    /// `*a.change_value(0)? = 7`.
    pub fn change_value(&mut self, index: i32) -> Result<&mut T, Failure> {
        let len: usize = self.vector.len();
        usize::try_from(index)
            .ok()
            .and_then(|i| self.vector.get_mut(i))
            .ok_or_else(|| storage_error(index, len))
    }
}

impl<T: IntPolyhDumpItem> IntPolyhArray<T> {
    /// `Dump()`: every stored item, numbered from 0, under the native title
    /// ("ArrayOfSectionLines" whatever the item type). `a.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        write!(out, "\n ArrayOfSectionLines 0-> {}", self.nb_items - 1)?;
        for i in 0..self.nb_items {
            self[i].dump_item(out)?;
        }
        writeln!(out)
    }
}

/// The `Dump()` call of `IntPolyh_Array::Dump` on an item: the item classes that have a
/// `Dump()` without argument (points, start points, section lines); the native template
/// does not compile for the others.
pub trait IntPolyhDumpItem {
    /// Writes the item as its `Dump()` does.
    fn dump_item(&self, out: &mut dyn fmt::Write) -> fmt::Result;
}

fn storage_error(index: i32, len: usize) -> Failure {
    Failure::out_of_range(&format!(
        "IntPolyh_Array::Value: index {index} is outside the storage 0..{len}"
    ))
}

impl<T> Index<i32> for IntPolyhArray<T> {
    type Output = T;
    /// `operator[]`; panics past the storage (undefined behaviour natively).
    fn index(&self, index: i32) -> &T {
        match usize::try_from(index).ok().and_then(|i| self.vector.get(i)) {
            Some(item) => item,
            None => panic!("{}", storage_error(index, self.vector.len()).message()),
        }
    }
}

impl<T> IndexMut<i32> for IntPolyhArray<T> {
    fn index_mut(&mut self, index: i32) -> &mut T {
        let len: usize = self.vector.len();
        match usize::try_from(index)
            .ok()
            .and_then(|i| self.vector.get_mut(i))
        {
            Some(item) => item,
            None => panic!("{}", storage_error(index, len).message()),
        }
    }
}

/// `IntPolyh_PointNormal`: a point of the surface and its unit normal.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IntPolyhPointNormal {
    /// `Point`.
    pub point: Point3,
    /// `Normal`.
    pub normal: Vector3,
}

/// `IntPolyh_ArrayOfPoints`.
pub type IntPolyhArrayOfPoints = IntPolyhArray<IntPolyhPoint>;
/// `IntPolyh_ArrayOfEdges`.
pub type IntPolyhArrayOfEdges = IntPolyhArray<IntPolyhEdge>;
/// `IntPolyh_ArrayOfTriangles`.
pub type IntPolyhArrayOfTriangles = IntPolyhArray<IntPolyhTriangle>;
/// `IntPolyh_ArrayOfSectionLines`.
pub type IntPolyhArrayOfSectionLines = IntPolyhArray<IntPolyhSectionLine>;
/// `IntPolyh_ArrayOfTangentZones`.
pub type IntPolyhArrayOfTangentZones = IntPolyhArray<IntPolyhStartPoint>;
/// `IntPolyh_ArrayOfPointNormal`.
pub type IntPolyhArrayOfPointNormal = IntPolyhArray<IntPolyhPointNormal>;
