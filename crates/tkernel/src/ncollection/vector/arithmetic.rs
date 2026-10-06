//! Per-component NCollection_Vec2/3/4 operators in the native left-to-right order.
use super::{Vec2, Vec3, Vec4, VectorArithmeticComponent};
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

macro_rules! componentwise_assignment {
    ($vector:ident, $assign:ident, $assign_method:ident, $returning:ident, $policy:ident, [$($field:ident),+]) => {
        impl<T: VectorArithmeticComponent + $assign> $assign for $vector<T> {
            fn $assign_method(&mut self, right: Self) {
                self.$returning(&right);
            }
        }
        impl<T: VectorArithmeticComponent + $assign> $vector<T> {
            #[doc = concat!("Applies component assignment in OCCT storage order and returns the destination. ",
                "Usage: `vector.", stringify!($returning), "(&right)` permits chaining.")]
            pub fn $returning(&mut self, right: &Self) -> &mut Self {
                $(self.$field.$policy(right.$field);)+
                self
            }
        }
    };
}

macro_rules! componentwise_value_direct {
    ($vector:ident, $trait:ident, $method:ident, $policy:ident, [$first:ident $(, $rest:ident)*]) => {
        impl<T: VectorArithmeticComponent + $trait<Output = T>> $trait for $vector<T> {
            type Output = Self;
            fn $method(self, right: Self) -> Self {
                Self { $first: self.$first.$policy(right.$first) $(, $rest: self.$rest.$policy(right.$rest))* }
            }
        }
    };
}

macro_rules! componentwise_value_via_assignment {
    ($vector:ident, $trait:ident, $method:ident, $assign:ident, $assign_method:ident) => {
        impl<T: VectorArithmeticComponent + $assign> $trait for $vector<T> {
            type Output = Self;
            fn $method(mut self, right: Self) -> Self {
                self.$assign_method(right);
                self
            }
        }
    };
}

// OCCT Vec2 spells value operations directly; Vec3/4 implement them through +=, -=, *=, /=.
macro_rules! componentwise_arithmetic {
    ($vector:ident, $count:literal, [$first:ident $(, $rest:ident)*]) => {
        componentwise_assignment!($vector, AddAssign, add_assign, add_assign_returning, vector_add_assign, [$first $(, $rest)*]);
        componentwise_assignment!($vector, SubAssign, sub_assign, sub_assign_returning, vector_subtract_assign, [$first $(, $rest)*]);
        componentwise_assignment!($vector, MulAssign, mul_assign, mul_assign_returning, vector_multiply_assign, [$first $(, $rest)*]);
        componentwise_assignment!($vector, DivAssign, div_assign, div_assign_returning, vector_divide_assign, [$first $(, $rest)*]);
        impl<T: VectorArithmeticComponent + Neg<Output = T>> Neg for $vector<T> {
            type Output = Self;
            fn neg(self) -> Self {
                Self { $first: self.$first.vector_negate() $(, $rest: self.$rest.vector_negate())* }
            }
        }
        impl<T> Index<usize> for $vector<T> {
            type Output = T;
            fn index(&self, index: usize) -> &T {
                [&self.$first $(, &self.$rest)*]
                    .into_iter()
                    .nth(index)
                    .unwrap_or_else(|| panic!("component {index}: expected < {}", $count))
            }
        }
        impl<T> IndexMut<usize> for $vector<T> {
            fn index_mut(&mut self, index: usize) -> &mut T {
                [&mut self.$first $(, &mut self.$rest)*]
                    .into_iter()
                    .nth(index)
                    .unwrap_or_else(|| panic!("component {index}: expected < {}", $count))
            }
        }
        impl<T> $vector<T> {
            /// Reports the native component count; `Vec3::<f64>::length() == 3`.
            pub const fn length() -> usize {
                $count
            }
            /// Borrows every component writably in storage order, as `ChangeData` and the
            /// non-const `Element_t*` conversion; `*vector.change_data()[0] = 1.0`.
            pub fn change_data(&mut self) -> [&mut T; $count] {
                [&mut self.$first $(, &mut self.$rest)*]
            }
        }
        impl<T: PartialEq> $vector<T> {
            /// Compares components with `==` in order, as `IsEqual` (NaN is never equal);
            /// `a.is_equal(&b)`.
            pub fn is_equal(&self, other: &Self) -> bool {
                self == other
            }
        }
        impl<T: PartialEq> PartialEq for $vector<T> {
            fn eq(&self, other: &Self) -> bool {
                self.$first == other.$first $( && self.$rest == other.$rest )*
            }
            #[allow(clippy::partialeq_ne_impl)] // Native operator!= negates IsEqual.
            fn ne(&self, other: &Self) -> bool {
                let are_equal = self.eq(other);
                !are_equal
            }
        }
        impl<T: Copy> $vector<T> {
            /// Assigns every component in storage order; `vector.set_values(1, 2, ...)`.
            pub fn set_values(&mut self, $first: T $(, $rest: T)*) {
                self.$first = $first;
                $(self.$rest = $rest;)*
            }
            /// Copies components in storage order; native `GetData` uses `as_components`. `Vec2::new(1,2).data() == [1,2]`.
            pub const fn data(&self) -> [T; $count] {
                [self.$first $(, self.$rest)*]
            }
            /// Builds a vector from components in storage order; `Vec2::from_array([1,2]).y == 2`.
            pub const fn from_array(values: [T; $count]) -> Self {
                let [$first $(, $rest)*] = values;
                Self { $first $(, $rest)* }
            }
        }
        impl<T: Copy + PartialOrd> $vector<T> {
            /// Keeps `self` only when strictly smaller, as `cwiseMin`; `a.cwise_min(&b)`.
            pub fn cwise_min(&self, other: &Self) -> Self {
                Self {
                    $first: if self.$first < other.$first { self.$first } else { other.$first }
                    $(, $rest: if self.$rest < other.$rest { self.$rest } else { other.$rest })*
                }
            }
            /// Keeps `self` only when strictly larger, as `cwiseMax`; `a.cwise_max(&b)`.
            pub fn cwise_max(&self, other: &Self) -> Self {
                Self {
                    $first: if self.$first > other.$first { self.$first } else { other.$first }
                    $(, $rest: if self.$rest > other.$rest { self.$rest } else { other.$rest })*
                }
            }
        }
        impl<T: VectorArithmeticComponent + Add<Output = T> + Mul<Output = T>> $vector<T> {
            /// Sums products left to right, as `Dot`; `Vec2::new(1,2).dot(&Vec2::new(3,4)) == 11`.
            pub fn dot(&self, other: &Self) -> T {
                self.$first.vector_multiply(other.$first) $(.vector_add(self.$rest.vector_multiply(other.$rest)))*
            }
        }
        impl<T: VectorArithmeticComponent + MulAssign> $vector<T> {
            /// Scales each component in place, as `Multiply`; `vector.multiply(2.0)`.
            pub fn multiply(&mut self, factor: T) {
                self.$first.vector_multiply_assign(factor);
                $(self.$rest.vector_multiply_assign(factor);)*
            }
        }
    };
}
macro_rules! direct_values {
    ($vector:ident, [$first:ident $(, $rest:ident)*]) => {
        componentwise_value_direct!($vector, Add, add, vector_add, [$first $(, $rest)*]);
        componentwise_value_direct!($vector, Sub, sub, vector_subtract, [$first $(, $rest)*]);
        componentwise_value_direct!($vector, Mul, mul, vector_multiply, [$first $(, $rest)*]);
        componentwise_value_direct!($vector, Div, div, vector_divide, [$first $(, $rest)*]);
    };
}

macro_rules! values_via_assignment {
    ($vector:ident) => {
        componentwise_value_via_assignment!($vector, Add, add, AddAssign, add_assign);
        componentwise_value_via_assignment!($vector, Sub, sub, SubAssign, sub_assign);
        componentwise_value_via_assignment!($vector, Mul, mul, MulAssign, mul_assign);
        componentwise_value_via_assignment!($vector, Div, div, DivAssign, div_assign);
    };
}

componentwise_arithmetic!(Vec2, 2, [x, y]);
componentwise_arithmetic!(Vec3, 3, [x, y, z]);
componentwise_arithmetic!(Vec4, 4, [x, y, z, w]);
direct_values!(Vec2, [x, y]);
values_via_assignment!(Vec3);
values_via_assignment!(Vec4);

// Scalar operators previously used primitive stamps to avoid a suspected coherence
// overlap. Scalar T and vector VecN<T> RHS types are disjoint; generic impls preserve
// OCCT template support. Vec2 copies use *, while Vec3/4 copies use *=, as upstream.
macro_rules! scalar_multiplication {
    ($vector:ident) => {
        impl<T: VectorArithmeticComponent + MulAssign> MulAssign<T> for $vector<T> {
            fn mul_assign(&mut self, factor: T) {
                self.multiply_scalar_returning(factor);
            }
        }
        impl<T: VectorArithmeticComponent + MulAssign> $vector<T> {
            /// Scales in OCCT component order and returns the destination; `vector.multiply_scalar_returning(2)`.
            pub fn multiply_scalar_returning(&mut self, factor: T) -> &mut Self {
                self.multiply(factor);
                self
            }
        }
    };
}
macro_rules! scalar_copy_multiplication {
    ($vector:ident) => {
        impl<T: VectorArithmeticComponent + MulAssign> $vector<T> {
            /// Returns a scaled copy, as `Multiplied`; `vector.multiplied(2.0)`.
            pub fn multiplied(&self, factor: T) -> Self {
                let mut result = *self;
                result.multiply(factor);
                result
            }
        }
        impl<T: VectorArithmeticComponent + MulAssign> Mul<T> for $vector<T> {
            type Output = Self;
            fn mul(self, factor: T) -> Self {
                self.multiplied(factor)
            }
        }
    };
}
impl<T: VectorArithmeticComponent + Mul<Output = T>> Vec2<T> {
    /// Returns a scaled copy, as `Multiplied`; `Vec2::new(1, 2).multiplied(3)`.
    pub fn multiplied(&self, factor: T) -> Self {
        Self::new(
            self.x.vector_multiply(factor),
            self.y.vector_multiply(factor),
        )
    }
}
impl<T: VectorArithmeticComponent + Mul<Output = T>> Mul<T> for Vec2<T> {
    type Output = Self;
    fn mul(self, factor: T) -> Self {
        self.multiplied(factor)
    }
}
scalar_copy_multiplication!(Vec3);
scalar_copy_multiplication!(Vec4);

scalar_multiplication!(Vec2);
scalar_multiplication!(Vec3);
scalar_multiplication!(Vec4);
impl<T: VectorArithmeticComponent + DivAssign<T>> DivAssign<T> for Vec2<T> {
    fn div_assign(&mut self, divisor: T) {
        self.divide_scalar_returning(divisor);
    }
}
impl<T: VectorArithmeticComponent + DivAssign<T>> Vec2<T> {
    /// Divides each component directly and returns the destination; `vector.divide_scalar_returning(3)`.
    pub fn divide_scalar_returning(&mut self, divisor: T) -> &mut Self {
        self.x.vector_divide_assign(divisor);
        self.y.vector_divide_assign(divisor);
        self
    }
}
impl<T: VectorArithmeticComponent + Div<Output = T>> Div<T> for Vec2<T> {
    type Output = Self;
    fn div(self, divisor: T) -> Self {
        Self::new(self.x.vector_divide(divisor), self.y.vector_divide(divisor))
    }
}

impl<T: Copy + PartialOrd> Vec2<T> {
    /// Returns the larger component, preferring y on ties; `Vec2::new(1,2).max_comp() == 2`.
    pub fn max_comp(&self) -> T {
        if self.x > self.y { self.x } else { self.y }
    }
    /// Returns the smaller component, preferring y on ties; `Vec2::new(1,2).min_comp() == 1`.
    pub fn min_comp(&self) -> T {
        if self.x < self.y { self.x } else { self.y }
    }
}
impl<T: Copy + PartialOrd> Vec3<T> {
    /// Uses the native nested comparison; `Vec3::new(1,3,2).max_comp() == 3`.
    pub fn max_comp(&self) -> T {
        let (x, y, z) = (self.x, self.y, self.z);
        if x > y {
            if x > z { x } else { z }
        } else if y > z {
            y
        } else {
            z
        }
    }
    /// Uses the native nested comparison; `Vec3::new(2,1,3).min_comp() == 1`.
    pub fn min_comp(&self) -> T {
        let (x, y, z) = (self.x, self.y, self.z);
        if x < y {
            if x < z { x } else { z }
        } else if y < z {
            y
        } else {
            z
        }
    }
}
impl<T: Copy + PartialOrd> Vec4<T> {
    /// Compares pairs (x,y) and (z,w) first, as upstream; `Vec4::new(1,4,2,3).max_comp() == 4`.
    pub fn max_comp(&self) -> T {
        let first = if self.x > self.y { self.x } else { self.y };
        let second = if self.z > self.w { self.z } else { self.w };
        if first > second { first } else { second }
    }
    /// Compares pairs (x,y) and (z,w) first, as upstream; `Vec4::new(4,1,2,3).min_comp() == 1`.
    pub fn min_comp(&self) -> T {
        let first = if self.x < self.y { self.x } else { self.y };
        let second = if self.z < self.w { self.z } else { self.w };
        if first < second { first } else { second }
    }
}
