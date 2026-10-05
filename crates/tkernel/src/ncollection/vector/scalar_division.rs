//! OCCT specializes Vec3/4 floating division as one reciprocal followed by multiplication.
use super::{Vec3, Vec4};
use std::ops::{Div, DivAssign};

/// Selects OCCT's scalar division path for Vec3/4 elements.
/// Custom `Copy + DivAssign` elements opt into the default with
/// `impl VectorScalarDivision for MyElement {}`; floating types use OCCT's reciprocal path.
pub trait VectorScalarDivision: Copy + DivAssign {
    /// Divides components in storage order; `3_i64.divide_components([&mut value])`.
    fn divide_components<const N: usize>(self, components: [&mut Self; N]) {
        for component in components {
            *component /= self;
        }
    }
}

macro_rules! integral_division {
    ($($element:ty),+ $(,)?) => { $(impl VectorScalarDivision for $element {})+ };
}
integral_division!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

macro_rules! floating_division {
    ($element:ty) => {
        impl VectorScalarDivision for $element {
            fn divide_components<const N: usize>(self, components: [&mut Self; N]) {
                let reciprocal: Self = 1.0 / self;
                for component in components {
                    *component *= reciprocal;
                }
            }
        }
    };
}
floating_division!(f32);
floating_division!(f64);

macro_rules! scalar_vector_operators {
    ($vector:ident) => {
        impl<T: VectorScalarDivision> DivAssign<T> for $vector<T> {
            fn div_assign(&mut self, divisor: T) {
                self.divide_scalar_returning(divisor);
            }
        }
        impl<T: VectorScalarDivision> $vector<T> {
            /// Returns the destination after native scalar division; `vector.divide_scalar_returning(3.0)`.
            /// OCCT 3d097a03 specializes f32/f64 to multiply by one reciprocal.
            pub fn divide_scalar_returning(&mut self, divisor: T) -> &mut Self {
                divisor.divide_components(self.change_data());
                self
            }
        }
        impl<T: VectorScalarDivision> Div<T> for $vector<T> {
            type Output = Self;
            fn div(mut self, divisor: T) -> Self {
                self /= divisor;
                self
            }
        }
    };
}
scalar_vector_operators!(Vec3);
scalar_vector_operators!(Vec4);
