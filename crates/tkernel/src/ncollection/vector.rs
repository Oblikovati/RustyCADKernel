// Copyright (c) 2025 OPEN CASCADE SAS
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

//! NCollection_Vec2/Vec3/Vec4: small vector primitives with OCCT's evaluation order.
//!
//! The native classes store an array; here the components are public fields, so the
//! non-const `x()` reference overloads are field access and `ChangeData` returns an array
//! of component references.
mod arithmetic;
mod conversion;
mod float;
mod swizzle;
pub use conversion::ElementCast;

/// Two scalar components; `Vec2::new(1.0_f64,2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2<T> {
    /// First component; `vector.x = 1.0`.
    pub x: T,
    /// Second component; `vector.y = 2.0`.
    pub y: T,
}
/// Three scalar components; `Vec3::new(1.0_f64,2.0,3.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3<T> {
    /// First component; `vector.x = 1.0`.
    pub x: T,
    /// Second component; `vector.y = 2.0`.
    pub y: T,
    /// Third component; `vector.z = 3.0`.
    pub z: T,
}
/// Four scalar components; `Vec4::new(1.0_f64,2.0,3.0,1.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4<T> {
    /// First component; `vector.x = 1.0`.
    pub x: T,
    /// Second component; `vector.y = 2.0`.
    pub y: T,
    /// Third component; `vector.z = 3.0`.
    pub z: T,
    /// Homogeneous component; `vector.w = 1.0`.
    pub w: T,
}
impl<T> Vec2<T> {
    /// Initializes two values; `Vec2::new(1,2).y == 2`.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}
impl<T> Vec3<T> {
    /// Initializes three values; `Vec3::new(1,2,3).z == 3`.
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}
impl<T> Vec4<T> {
    /// Initializes four values; `Vec4::new(1,2,3,4).w == 4`.
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }
}
impl<T: Copy> Vec2<T> {
    /// Fills both components, as the explicit scalar constructor does; `Vec2::from_value(0).x == 0`.
    pub const fn from_value(value: T) -> Self {
        Self::new(value, value)
    }
}
impl<T: Copy> Vec3<T> {
    /// Fills all components, as the explicit scalar constructor does; `Vec3::from_value(0).z == 0`.
    pub const fn from_value(value: T) -> Self {
        Self::new(value, value, value)
    }
    /// Extends a planar vector with Z (the native default is 0); `Vec3::from_vec2(Vec2::new(1,2), 3).z == 3`.
    pub const fn from_vec2(planar: Vec2<T>, z: T) -> Self {
        Self::new(planar.x, planar.y, z)
    }
    /// Replaces the components from a planar vector and Z, as `SetValues(Vec2, z)`;
    /// `vector.set_values_vec2(Vec2::new(1,2), 3)`.
    pub fn set_values_vec2(&mut self, planar: Vec2<T>, z: T) {
        *self = Self::from_vec2(planar, z);
    }
}
impl<T: Copy> Vec4<T> {
    /// Fills all components, as the explicit scalar constructor does; `Vec4::from_value(0).w == 0`.
    pub const fn from_value(value: T) -> Self {
        Self::new(value, value, value, value)
    }
    /// Extends a spatial vector with W (the native default is 0); `Vec4::from_vec3(Vec3::new(1,2,3), 1).w == 1`.
    pub const fn from_vec3(spatial: Vec3<T>, w: T) -> Self {
        Self::new(spatial.x, spatial.y, spatial.z, w)
    }
    /// Replaces the components from a spatial vector and W, as `SetValues(Vec3, w)`;
    /// `vector.set_values_vec3(Vec3::new(1,2,3), 4)`.
    pub fn set_values_vec3(&mut self, spatial: Vec3<T>, w: T) {
        *self = Self::from_vec3(spatial, w);
    }
}
impl<T: Copy + Default> Vec4<T> {
    /// Extends a planar vector with zero Z and W, as `NCollection_Vec4(Vec2)`;
    /// `Vec4::from_vec2(Vec2::new(1,2)).z == 0`.
    pub fn from_vec2(planar: Vec2<T>) -> Self {
        Self::new(planar.x, planar.y, T::default(), T::default())
    }
}
