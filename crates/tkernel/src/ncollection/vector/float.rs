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

//! Magnitude, unit, interpolation and JSON members of NCollection_Vec2/Vec3/Vec4.
use super::{Vec2, Vec3, Vec4};
use crate::standard::dump::{self, DumpOutput};
use std::io;
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
impl<T: Copy + From<u8>> Vec2<T> {
    /// Builds the X unit vector; `Vec2::<f64>::dx().x == 1.0`.
    pub fn dx() -> Self {
        Self::new(T::from(1), T::from(0))
    }
    /// Builds the Y unit vector; `Vec2::<f64>::dy().y == 1.0`.
    pub fn dy() -> Self {
        Self::new(T::from(0), T::from(1))
    }
}
impl<T: Copy + From<u8>> Vec3<T> {
    /// Builds the X unit vector; `Vec3::<f64>::dx().x == 1.0`.
    pub fn dx() -> Self {
        Self::new(T::from(1), T::from(0), T::from(0))
    }
    /// Builds the Y unit vector; `Vec3::<f64>::dy().y == 1.0`.
    pub fn dy() -> Self {
        Self::new(T::from(0), T::from(1), T::from(0))
    }
    /// Builds the Z unit vector; `Vec3::<f64>::dz().z == 1.0`.
    pub fn dz() -> Self {
        Self::new(T::from(0), T::from(0), T::from(1))
    }
}
impl<T> Vec3<T>
where
    T: Copy + From<u8> + Sub<Output = T>,
    Vec3<T>: Mul<T, Output = Vec3<T>> + Add<Output = Vec3<T>>,
{
    /// Interpolates `from*(1-t) + to*t`, as `GetLERP`; `Vec3::lerp(&a, &b, 0.5)`.
    pub fn lerp(from: &Self, to: &Self, parameter: T) -> Self {
        *from * (T::from(1) - parameter) + *to * parameter
    }
}
/// `Modulus`, `cwiseAbs`, `Normalize` and `Normalized` for one element type; `$sqrt` is the
/// native `std::sqrt(x*x + ...)` returned as the element type.
macro_rules! magnitude_members {
    ($scalar:ty, $zero:expr, |$square:ident| $sqrt:expr, |$value:ident| $abs:expr) => {
        impl Vec2<$scalar> {
            /// Returns `std::sqrt(x*x+y*y)` as the element type, as `Modulus`; `Vec2::new(3.0,4.0).modulus()`.
            pub fn modulus(&self) -> $scalar {
                let $square: $scalar = self.square_modulus();
                $sqrt
            }
            /// Takes `std::abs` of each component, as `cwiseAbs`; `Vec2::new(-1.0,2.0).cwise_abs()`.
            pub fn cwise_abs(&self) -> Self {
                Self::new(
                    {
                        let $value = self.x;
                        $abs
                    },
                    {
                        let $value = self.y;
                        $abs
                    },
                )
            }
        }
        impl Vec3<$scalar> {
            /// Returns `std::sqrt(x*x+y*y+z*z)` as the element type, as `Modulus`.
            pub fn modulus(&self) -> $scalar {
                let $square: $scalar = self.square_modulus();
                $sqrt
            }
            /// Takes `std::abs` of each component, as `cwiseAbs`.
            pub fn cwise_abs(&self) -> Self {
                let abs = |$value: $scalar| -> $scalar { $abs };
                Self::new(abs(self.x), abs(self.y), abs(self.z))
            }
            /// Divides each component by the modulus unless it is zero, as `Normalize`.
            pub fn normalize(&mut self) {
                let modulus: $scalar = self.modulus();
                // Upstream divides component by component, not by a reciprocal.
                if modulus != $zero {
                    self.x /= modulus;
                    self.y /= modulus;
                    self.z /= modulus;
                }
            }
            /// Returns a normalized copy, as `Normalized`; `vector.normalized()`.
            pub fn normalized(&self) -> Self {
                let mut copy: Self = *self;
                copy.normalize();
                copy
            }
        }
        impl Vec4<$scalar> {
            /// Takes `std::abs` of each component, as `cwiseAbs`.
            pub fn cwise_abs(&self) -> Self {
                let abs = |$value: $scalar| -> $scalar { $abs };
                Self::new(abs(self.x), abs(self.y), abs(self.z), abs(self.w))
            }
        }
    };
}
magnitude_members!(f32, 0.0, |square| square.sqrt(), |value| value.abs());
magnitude_members!(f64, 0.0, |square| square.sqrt(), |value| value.abs());
// std::sqrt(int) computes in double and the int return value truncates it; std::abs(INT_MIN)
// is undefined natively and wraps here.
magnitude_members!(i32, 0, |square| f64::from(square).sqrt() as i32, |value| {
    value.wrapping_abs()
});
/// `DumpJson` through `OCCT_DUMP_FIELD_VALUES_NUMERICAL`. The macro reads each argument as
/// a `double`, so float components are promoted exactly; integer instantiations would read
/// an int as a double, which is undefined natively, and have no `dump_json`.
macro_rules! dump_members {
    ($scalar:ty) => {
        impl Vec2<$scalar> {
            /// Writes `"Vec2": [x, y]`; `vector.dump_json(&mut stream)?`.
            pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
                dump::dump_vector(output, "Vec2", &self.data().map(f64::from))
            }
        }
        impl Vec3<$scalar> {
            /// Writes `"Vec3": [x, y, z]`; `vector.dump_json(&mut stream)?`.
            pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
                dump::dump_vector(output, "Vec3", &self.data().map(f64::from))
            }
        }
        impl Vec4<$scalar> {
            /// Writes `"Vec4": [x, y, z, w]`; `vector.dump_json(&mut stream)?`.
            pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
                dump::dump_vector(output, "Vec4", &self.data().map(f64::from))
            }
        }
    };
}
dump_members!(f32);
dump_members!(f64);
