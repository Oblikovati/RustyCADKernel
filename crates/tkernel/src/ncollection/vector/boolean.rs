//! C++ promotes boolean operands to int, then converts each stored result back to bool.
use super::{Vec2, Vec3, Vec4};
use crate::ncollection::{VectorAbsoluteComponent, VectorSquareComponent};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

fn boolean_division(numerator: bool, denominator: bool) -> bool {
    // Native division by false is undefined; retain Rust's explicit invalid-input failure.
    assert!(
        denominator,
        "boolean vector divisor {denominator}: expected true"
    );
    numerator
}
impl VectorAbsoluteComponent for bool {
    fn vector_component_abs(self) -> Self {
        self
    }
}
impl VectorSquareComponent for bool {
    fn vector_component_square<const N: usize>(components: [Self; N]) -> Self {
        components.into_iter().any(|component| component)
    }
}
macro_rules! boolean_value_assignment {
    ($vector:ident, [$($field:ident),+], $value:ident, $value_fn:ident, $assign:ident, $assign_fn:ident, $returning:ident, $operation:expr) => {
        impl $assign for $vector<bool> {
            fn $assign_fn(&mut self, right: Self) { self.$returning(&right); }
        }
        impl $value for $vector<bool> {
            type Output = Self;
            fn $value_fn(mut self, right: Self) -> Self { self.$returning(&right); self }
        }
        impl $vector<bool> {
            #[doc = concat!("Applies promoted boolean arithmetic in storage order; `vector.", stringify!($returning), "(&right)`.")]
            pub fn $returning(&mut self, right: &Self) -> &mut Self {
                $(self.$field = ($operation)(self.$field, right.$field);)+
                self
            }
        }
    };
}
macro_rules! boolean_vector_arithmetic {
    ($vector:ident, [$($field:ident),+]) => {
        boolean_value_assignment!($vector, [$($field),+], Add, add, AddAssign, add_assign, add_assign_returning, |a: bool, b: bool| a | b);
        boolean_value_assignment!($vector, [$($field),+], Sub, sub, SubAssign, sub_assign, sub_assign_returning, |a: bool, b: bool| a ^ b);
        boolean_value_assignment!($vector, [$($field),+], Mul, mul, MulAssign, mul_assign, mul_assign_returning, |a: bool, b: bool| a & b);
        boolean_value_assignment!($vector, [$($field),+], Div, div, DivAssign, div_assign, div_assign_returning, boolean_division);
        impl $vector<bool> {
            /// Converts the integer sum of boolean products to bool; `left.dot(&right)`.
            pub fn dot(&self, other: &Self) -> bool { false $(| (self.$field & other.$field))+ }
            /// Scales boolean components by promoted integer multiplication; `vector.multiply(false)`.
            pub fn multiply(&mut self, factor: bool) { $(self.$field &= factor;)+ }
            /// Returns a scaled copy; `vector.multiplied(true)` preserves its components.
            pub fn multiplied(&self, factor: bool) -> Self {
                let mut result = *self;
                result.multiply(factor);
                result
            }
            /// Scales in storage order and returns the destination; `vector.multiply_scalar_returning(false)`.
            pub fn multiply_scalar_returning(&mut self, factor: bool) -> &mut Self {
                self.multiply(factor);
                self
            }
            /// Divides components by a nonzero boolean; `vector.divide_scalar_returning(true)`.
            pub fn divide_scalar_returning(&mut self, divisor: bool) -> &mut Self {
                $(self.$field = boolean_division(self.$field, divisor);)+
                self
            }
        }
        impl Mul<bool> for $vector<bool> {
            type Output = Self;
            fn mul(self, factor: bool) -> Self { self.multiplied(factor) }
        }
        impl MulAssign<bool> for $vector<bool> {
            fn mul_assign(&mut self, factor: bool) { self.multiply(factor); }
        }
        impl Div<bool> for $vector<bool> {
            type Output = Self;
            fn div(mut self, divisor: bool) -> Self { self.divide_scalar_returning(divisor); self }
        }
        impl DivAssign<bool> for $vector<bool> {
            fn div_assign(&mut self, divisor: bool) { self.divide_scalar_returning(divisor); }
        }
    };
}
boolean_vector_arithmetic!(Vec2, [x, y]);
boolean_vector_arithmetic!(Vec3, [x, y, z]);
boolean_vector_arithmetic!(Vec4, [x, y, z, w]);
impl Vec2<bool> {
    /// Converts the square root of the integer squared sum back to bool; `Vec2::new(true, false).modulus()`.
    pub fn modulus(&self) -> bool {
        self.square_modulus()
    }
}
impl Vec3<bool> {
    /// Converts the square root of the integer squared sum back to bool; `Vec3::new(true, false, true).modulus()`.
    pub fn modulus(&self) -> bool {
        self.square_modulus()
    }
    /// Native nonzero boolean magnitude is true, so normalization preserves components; `vector.normalize()`.
    pub fn normalize(&mut self) {}
    /// Returns the unchanged boolean vector after native normalization; `vector.normalized()`.
    pub fn normalized(&self) -> Self {
        *self
    }
}
