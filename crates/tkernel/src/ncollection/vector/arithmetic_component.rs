//! Native integer promotion/narrowing with opt-in operator dispatch for custom scalars.
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

macro_rules! value_operation {
    ($method:ident, $operator:tt, $bound:ident) => {
        #[doc = concat!("Evaluates the native scalar operation; `left.", stringify!($method), "(right)`.")]
        fn $method(self, right: Self) -> Self where Self: $bound<Output = Self> {
            self $operator right
        }
    };
}
macro_rules! assignment_operation {
    ($method:ident, $operator:tt, $bound:ident) => {
        #[doc = concat!("Uses the scalar assignment overload; `left.", stringify!($method), "(right)`.")]
        fn $method(&mut self, right: Self) where Self: $bound {
            *self $operator right;
        }
    };
}

/// Native vector arithmetic policy. Custom operator types retain their dispatch with
/// `impl VectorArithmeticComponent for MyElement {}`; primitive integers override
/// the defaults so native narrowing never becomes Rust debug-overflow panics.
pub trait VectorArithmeticComponent: Copy {
    value_operation!(vector_add, +, Add);
    value_operation!(vector_subtract, -, Sub);
    value_operation!(vector_multiply, *, Mul);
    value_operation!(vector_divide, /, Div);
    assignment_operation!(vector_add_assign, +=, AddAssign);
    assignment_operation!(vector_subtract_assign, -=, SubAssign);
    assignment_operation!(vector_multiply_assign, *=, MulAssign);
    assignment_operation!(vector_divide_assign, /=, DivAssign);
    /// Uses the scalar unary operator; `value.vector_negate()`.
    fn vector_negate(self) -> Self
    where
        Self: Neg<Output = Self>,
    {
        -self
    }
}
impl VectorArithmeticComponent for f32 {}
impl VectorArithmeticComponent for f64 {}

macro_rules! promoted_value {
    ($scalar:ty, $promoted:ty, $method:ident, $wrapping:ident) => {
        fn $method(self, right: Self) -> Self {
            (self as $promoted).$wrapping(right as $promoted) as $scalar
        }
    };
}
macro_rules! promoted_assignment {
    ($method:ident, $value_method:ident) => {
        fn $method(&mut self, right: Self) {
            *self = self.$value_method(right);
        }
    };
}
macro_rules! primitive_negation {
    (signed; $scalar:ty, $promoted:ty) => {
        fn vector_negate(self) -> Self {
            (self as $promoted).wrapping_neg() as $scalar
        }
    };
    (; $scalar:ty, $promoted:ty) => {};
}
macro_rules! primitive_arithmetic {
    ($scalar:ty, $promoted:ty $(, $signed:ident)?) => {
        impl VectorArithmeticComponent for $scalar {
            promoted_value!($scalar, $promoted, vector_add, wrapping_add);
            promoted_value!($scalar, $promoted, vector_subtract, wrapping_sub);
            promoted_value!($scalar, $promoted, vector_multiply, wrapping_mul);
            promoted_value!($scalar, $promoted, vector_divide, wrapping_div);
            promoted_assignment!(vector_add_assign, vector_add);
            promoted_assignment!(vector_subtract_assign, vector_subtract);
            promoted_assignment!(vector_multiply_assign, vector_multiply);
            promoted_assignment!(vector_divide_assign, vector_divide);
            primitive_negation!($($signed)?; $scalar, $promoted);
        }
    };
}
primitive_arithmetic!(i8, i32, signed);
primitive_arithmetic!(u8, i32);
primitive_arithmetic!(i16, i32, signed);
primitive_arithmetic!(u16, i32);
primitive_arithmetic!(i32, i32, signed);
primitive_arithmetic!(u32, u32);
primitive_arithmetic!(i64, i64, signed);
primitive_arithmetic!(u64, u64);
primitive_arithmetic!(isize, isize, signed);
primitive_arithmetic!(usize, usize);
primitive_arithmetic!(i128, i128, signed);
primitive_arithmetic!(u128, u128);
