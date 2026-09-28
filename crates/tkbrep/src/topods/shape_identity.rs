// Created on: 1990-12-11
// Created by: Remi Lequette
// Copyright (c) 1990-1999 Matra Datavision
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
// Rust translation of IsPartner, IsSame, IsEqual and `std::hash<TopoDS_Shape>` of
// ModelingData/TKBRep/TopoDS/TopoDS_Shape.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Shape identity and the native hash (docs/topology-design.md section 7).
use super::Shape;
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkernel::standard::hashing;

impl Shape {
    /// `IsPartner(theOther)`: the same TShape; `shape.is_partner(&other)`.
    pub fn is_partner(&self, other: &Self) -> bool {
        self.tshape() == other.tshape()
    }

    /// `IsSame(theOther)`: the same TShape and location; `shape.is_same(&other)`.
    pub fn is_same(&self, other: &Self) -> bool {
        self.is_partner(other) && self.location() == other.location()
    }

    /// `IsEqual(theOther)` (`operator==`): the same TShape, location and orientation;
    /// `shape.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.is_same(other) && self.orientation() == other.orientation()
    }

    /// `IsNotEqual(theOther)` (`operator!=`); `shape.is_not_equal(&other)`.
    pub fn is_not_equal(&self, other: &Self) -> bool {
        !self.is_equal(other)
    }

    /// The address of the TShape object, `TShape().get()`: 0 for a null shape.
    pub fn tshape_address(&self) -> usize {
        self.tshape().object_address() as usize
    }

    /// `std::hash<TopoDS_Shape>`: the hash of the TShape address and of the location.
    /// Addresses are specific to one process, natively as here; `shape.hash_code()`.
    pub fn hash_code(&self) -> usize {
        Self::hash_from_parts(self.tshape_address(), self.location().hash_code())
    }

    /// The `std::hash<TopoDS_Shape>` formula on explicit parts: Murmur of the 8 address
    /// bytes, seeded by the location hash, or by the default seed for the identity
    /// (location hash 0).
    ///
    /// ```
    /// use rustycad_tkbrep::topods::Shape;
    /// let identity = Shape::hash_from_parts(0x1000, 0);
    /// assert_ne!(identity, Shape::hash_from_parts(0x1000, 7));
    /// ```
    pub fn hash_from_parts(tshape_address: usize, location_hash: usize) -> usize {
        if location_hash == 0 {
            return hashing::hash::<usize, usize>(&tshape_address);
        }
        hashing::hash_combine(&[tshape_address], size_of::<usize>(), location_hash)
    }
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl Eq for Shape {}

impl HashKey for Shape {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}

impl std::hash::Hash for Shape {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}

impl std::fmt::Debug for Shape {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = self.tshape().get().map(|tshape| tshape.shape_type());
        formatter
            .debug_struct("Shape")
            .field("tshape", &(self.tshape_address() as *const ()))
            .field("type", &kind)
            .field("location", self.location())
            .field("orientation", &self.orientation())
            .finish()
    }
}
