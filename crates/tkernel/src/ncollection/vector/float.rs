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

//! Metric NCollection_Vec2/3/4 members: moduli, unit axes, cross products.
use super::{Vec2, Vec3, Vec4};
use std::ops::{Add, Mul, Sub};

impl<T: Copy + Add<Output = T> + Mul<Output = T>> Vec2<T> {
    /// Evaluates x*x+y*y in upstream order; `Vec2::new(3,4).square_modulus() == 25`.
    pub fn square_modulus(&self) -> T {
        self.x * self.x + self.y * self.y
    }
}
impl<T: Copy + Add<Output = T> + Mul<Output = T>> Vec3<T> {
    /// Evaluates x*x+y*y+z*z in upstream order; `Vec3::new(3,4,0).square_modulus() == 25`.
    pub fn square_modulus(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}
impl<T: Copy + Sub<Output = T> + Mul<Output = T>> Vec3<T> {
    /// Computes the right-handed cross product, as `Cross`; `Vec3::cross(&dx, &dy) == dz`.
    pub fn cross(first: &Self, second: &Self) -> Self {
        Self::new(
            first.y * second.z - first.z * second.y,
            first.z * second.x - first.x * second.z,
            first.x * second.y - first.y * second.x,
        )
    }
}

macro_rules! float_members {
    ($scalar:ty) => {
        impl Vec2<$scalar> {
            /// Returns sqrt(x*x+y*y), as `Modulus`; `Vec2::new(3.0,4.0).modulus() == 5.0`.
            pub fn modulus(&self) -> $scalar {
                self.square_modulus().sqrt()
            }
            /// Takes absolute values, as `cwiseAbs`; `Vec2::new(-1.0,2.0).cwise_abs()`.
            pub fn cwise_abs(&self) -> Self {
                Self::new(self.x.abs(), self.y.abs())
            }
            /// Builds the X unit vector; `Vec2::<f64>::dx().x == 1.0`.
            pub const fn dx() -> Self {
                Self::new(1.0, 0.0)
            }
            /// Builds the Y unit vector; `Vec2::<f64>::dy().y == 1.0`.
            pub const fn dy() -> Self {
                Self::new(0.0, 1.0)
            }
        }
        impl Vec3<$scalar> {
            /// Returns sqrt(x*x+y*y+z*z), as `Modulus`; `Vec3::new(2.0,3.0,6.0).modulus() == 7.0`.
            pub fn modulus(&self) -> $scalar {
                self.square_modulus().sqrt()
            }
            /// Takes absolute values, as `cwiseAbs`; `Vec3::new(-1.0,2.0,-3.0).cwise_abs()`.
            pub fn cwise_abs(&self) -> Self {
                Self::new(self.x.abs(), self.y.abs(), self.z.abs())
            }
            /// Divides each component by the modulus unless it is zero; `vector.normalize()`.
            pub fn normalize(&mut self) {
                let modulus = self.modulus();
                // Upstream divides component by component, not by a reciprocal.
                if modulus != 0.0 {
                    self.x /= modulus;
                    self.y /= modulus;
                    self.z /= modulus;
                }
            }
            /// Returns a normalized copy, as `Normalized`; `vector.normalized()`.
            pub fn normalized(&self) -> Self {
                let mut copy = *self;
                copy.normalize();
                copy
            }
            /// Interpolates `from*(1-t) + to*t`, as `GetLERP`; `Vec3::lerp(&a, &b, 0.5)`.
            pub fn lerp(from: &Self, to: &Self, parameter: $scalar) -> Self {
                *from * (1.0 - parameter) + *to * parameter
            }
            /// Builds the X unit vector; `Vec3::<f64>::dx().x == 1.0`.
            pub const fn dx() -> Self {
                Self::new(1.0, 0.0, 0.0)
            }
            /// Builds the Y unit vector; `Vec3::<f64>::dy().y == 1.0`.
            pub const fn dy() -> Self {
                Self::new(0.0, 1.0, 0.0)
            }
            /// Builds the Z unit vector; `Vec3::<f64>::dz().z == 1.0`.
            pub const fn dz() -> Self {
                Self::new(0.0, 0.0, 1.0)
            }
        }
        impl Vec4<$scalar> {
            /// Takes absolute values, as `cwiseAbs`; `Vec4::new(-1.0,2.0,-3.0,4.0).cwise_abs()`.
            pub fn cwise_abs(&self) -> Self {
                Self::new(self.x.abs(), self.y.abs(), self.z.abs(), self.w.abs())
            }
        }
    };
}
float_members!(f32);
float_members!(f64);
