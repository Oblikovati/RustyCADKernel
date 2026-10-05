//! Magnitude, unit, interpolation and JSON members of NCollection_Vec2/Vec3/Vec4.
use super::{Vec2, Vec3, Vec4};
use crate::standard::dump::{self, DumpOutput};
use std::io;
use std::ops::{Add, Mul, Sub};

impl<T: super::super::VectorSquareComponent> Vec2<T> {
    /// Evaluates x*x+y*y in upstream order; `Vec2::new(3,4).square_modulus() == 25`.
    pub fn square_modulus(&self) -> T {
        T::vector_component_square([self.x, self.y])
    }
}
impl<T: super::super::VectorSquareComponent> Vec3<T> {
    /// Evaluates x*x+y*y+z*z in upstream order; `Vec3::new(3,4,0).square_modulus() == 25`.
    pub fn square_modulus(&self) -> T {
        T::vector_component_square([self.x, self.y, self.z])
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
// OCCT returns the element type after sqrt, and Normalize uses / followed by assignment.
impl<T: super::super::VectorMagnitudeComponent> Vec2<T> {
    /// Returns native square-root magnitude with integral promotions; `Vec2::new(3_i16, 4).modulus() == 5`.
    pub fn modulus(&self) -> T {
        T::vector_component_modulus([self.x, self.y])
    }
}
impl<T: super::super::VectorMagnitudeComponent> Vec3<T> {
    /// Returns native square-root magnitude; `Vec3::new(3.0_f64, 4.0, 0.0).modulus() == 5.0`.
    pub fn modulus(&self) -> T {
        T::vector_component_modulus([self.x, self.y, self.z])
    }
    /// Divides each component by its magnitude unless zero; `vector.normalize()`.
    pub fn normalize(&mut self) {
        let modulus: T = self.modulus();
        if modulus != T::from(0) {
            self.x = self.x / modulus;
            self.y = self.y / modulus;
            self.z = self.z / modulus;
        }
    }
    /// Returns a normalized copy; `vector.normalized()`.
    pub fn normalized(&self) -> Self {
        let mut copy: Self = *self;
        copy.normalize();
        copy
    }
}
/// Implements componentwise `cwiseAbs` with the native element operation supplied per type.
macro_rules! generic_absolute_members {
    ($vector:ident, $($component:ident),+) => {
        impl<T: super::super::VectorAbsoluteComponent> $vector<T> {
            /// Returns componentwise absolute values; `vector.cwise_abs()`.
            pub fn cwise_abs(&self) -> Self {
                Self::new($(self.$component.vector_component_abs()),+)
            }
        }
    };
}
generic_absolute_members!(Vec2, x, y);
generic_absolute_members!(Vec3, x, y, z);
generic_absolute_members!(Vec4, x, y, z, w);
/// `DumpJson` through `OCCT_DUMP_FIELD_VALUES_NUMERICAL`. The macro reads each argument as
/// a `double`, so float components are promoted exactly; integer instantiations would read
/// an int as a double, which is undefined natively, and have no `dump_json`.
macro_rules! dump_members {
    () => {
        impl<T: super::super::VectorDumpComponent> Vec2<T> {
            /// Writes `"Vec2": [x, y]`; `vector.dump_json(&mut stream)?`.
            pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
                dump::dump_vector(
                    output,
                    "Vec2",
                    &self.data().map(|value| value.promote_vector_dump()),
                )
            }
        }
        impl<T: super::super::VectorDumpComponent> Vec3<T> {
            /// Writes `"Vec3": [x, y, z]`; `vector.dump_json(&mut stream)?`.
            pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
                dump::dump_vector(
                    output,
                    "Vec3",
                    &self.data().map(|value| value.promote_vector_dump()),
                )
            }
        }
        impl<T: super::super::VectorDumpComponent> Vec4<T> {
            /// Writes `"Vec4": [x, y, z, w]`; `vector.dump_json(&mut stream)?`.
            pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
                dump::dump_vector(
                    output,
                    "Vec4",
                    &self.data().map(|value| value.promote_vector_dump()),
                )
            }
        }
    };
}
dump_members!();
