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

//! Small vector primitives with OCCT's evaluation order.
use std::ops::{Add, Div, Mul};

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
impl<T: Copy> Vec4<T> {
    /// Copies the first three components; `Vec4::new(1,2,3,4).xyz().z == 3`.
    pub const fn xyz(&self) -> Vec3<T> {
        Vec3::new(self.x, self.y, self.z)
    }
}
impl<T: Copy + Add<Output = T> + Mul<Output = T>> Vec3<T> {
    /// Evaluates x*x+y*y+z*z in upstream order; `Vec3::new(3,4,0).square_modulus() == 25`.
    pub fn square_modulus(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}
macro_rules! float_division {
    ($scalar:ty) => {
        impl Div<$scalar> for Vec3<$scalar> {
            type Output = Self;
            fn div(self, divisor: $scalar) -> Self {
                // OCCT specializes floating scalar division as multiplication by reciprocal.
                let reciprocal = 1.0 / divisor;
                Self::new(
                    self.x * reciprocal,
                    self.y * reciprocal,
                    self.z * reciprocal,
                )
            }
        }
    };
}
float_division!(f32);
float_division!(f64);
