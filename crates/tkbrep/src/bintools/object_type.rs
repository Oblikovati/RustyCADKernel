// Copyright (c) 2021 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BinTools/BinTools_ObjectType.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools_ObjectType`: the object identifiers of the shape-reference format.

/// One object identifier byte. The reader stores whatever byte it finds (a malformed
/// file or the end of the stream gives values outside the enumeration, as the native
/// enum variable holds them), so the type is a byte with named constants rather than
/// a closed Rust enum. Bytes from 200 on encode a shape type and orientation
/// (`200 + (type << 2) + orientation`).
///
/// ```
/// use rustycad_tkbrep::bintools::ObjectType;
/// assert_eq!(ObjectType::END_SHAPE.0, 199);
/// assert!(ObjectType::REFERENCE16.is_reference());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ObjectType(pub u8);

impl ObjectType {
    /// `BinTools_ObjectType_Unknown`.
    pub const UNKNOWN: Self = Self(0);
    /// `BinTools_ObjectType_Reference8`: an 8-bit back reference.
    pub const REFERENCE8: Self = Self(1);
    /// `BinTools_ObjectType_Reference16`: a 16-bit back reference.
    pub const REFERENCE16: Self = Self(2);
    /// `BinTools_ObjectType_Reference32`: a 32-bit back reference.
    pub const REFERENCE32: Self = Self(3);
    /// `BinTools_ObjectType_Reference64`: a 64-bit back reference.
    pub const REFERENCE64: Self = Self(4);
    /// `BinTools_ObjectType_Location`: a location made of powered datums.
    pub const LOCATION: Self = Self(5);
    /// `BinTools_ObjectType_SimpleLocation`: one datum at power 1.
    pub const SIMPLE_LOCATION: Self = Self(6);
    /// `BinTools_ObjectType_EmptyLocation`: the identity.
    pub const EMPTY_LOCATION: Self = Self(7);
    /// `BinTools_ObjectType_LocationEnd`: the end of a datum list.
    pub const LOCATION_END: Self = Self(8);
    /// `BinTools_ObjectType_Curve`.
    pub const CURVE: Self = Self(9);
    /// `BinTools_ObjectType_EmptyCurve`.
    pub const EMPTY_CURVE: Self = Self(10);
    /// `BinTools_ObjectType_Curve2d`.
    pub const CURVE2D: Self = Self(11);
    /// `BinTools_ObjectType_EmptyCurve2d`.
    pub const EMPTY_CURVE2D: Self = Self(12);
    /// `BinTools_ObjectType_Surface`.
    pub const SURFACE: Self = Self(13);
    /// `BinTools_ObjectType_EmptySurface`.
    pub const EMPTY_SURFACE: Self = Self(14);
    /// `BinTools_ObjectType_Polygon3d`.
    pub const POLYGON3D: Self = Self(15);
    /// `BinTools_ObjectType_EmptyPolygon3d`.
    pub const EMPTY_POLYGON3D: Self = Self(16);
    /// `BinTools_ObjectType_PolygonOnTriangulation`.
    pub const POLYGON_ON_TRIANGULATION: Self = Self(17);
    /// `BinTools_ObjectType_EmptyPolygonOnTriangulation`.
    pub const EMPTY_POLYGON_ON_TRIANGULATION: Self = Self(18);
    /// `BinTools_ObjectType_Triangulation`.
    pub const TRIANGULATION: Self = Self(19);
    /// `BinTools_ObjectType_EmptyTriangulation`.
    pub const EMPTY_TRIANGULATION: Self = Self(20);
    /// `BinTools_ObjectType_EmptyShape`: the null shape.
    pub const EMPTY_SHAPE: Self = Self(198);
    /// `BinTools_ObjectType_EndShape`: the end of a shape record.
    pub const END_SHAPE: Self = Self(199);

    /// `BinTools_IStream::IsReference` on this identifier: one of the four reference
    /// widths; `ObjectType::REFERENCE8.is_reference()`.
    pub fn is_reference(self) -> bool {
        (Self::REFERENCE8.0..=Self::REFERENCE64.0).contains(&self.0)
    }
}
