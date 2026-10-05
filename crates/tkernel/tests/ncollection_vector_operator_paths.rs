//! Regressions for NCollection vector element operator dispatch.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Tagged(i32);

macro_rules! tagged_binary {
    ($trait:ident, $method:ident, $tag:literal) => {
        impl $trait for Tagged {
            type Output = Self;
            fn $method(self, right: Self) -> Self {
                Self($tag + self.0 * 10 + right.0)
            }
        }
    };
}

macro_rules! tagged_assign {
    ($trait:ident, $method:ident, $tag:literal) => {
        impl $trait for Tagged {
            fn $method(&mut self, right: Self) {
                self.0 = $tag + self.0 * 10 + right.0;
            }
        }
    };
}

tagged_binary!(Add, add, 1000);
tagged_assign!(AddAssign, add_assign, 2000);
tagged_binary!(Sub, sub, 3000);
tagged_assign!(SubAssign, sub_assign, 4000);
tagged_binary!(Mul, mul, 5000);
tagged_assign!(MulAssign, mul_assign, 6000);
tagged_binary!(Div, div, 7000);
tagged_assign!(DivAssign, div_assign, 8000);

#[test]
fn vec2_value_operators_use_element_value_operators() {
    let left = Vec2::new(Tagged(2), Tagged(4));
    let right = Vec2::new(Tagged(3), Tagged(2));
    assert_eq!((left + right).data(), [Tagged(1023), Tagged(1042)]);
    assert_eq!((left - right).data(), [Tagged(3023), Tagged(3042)]);
    assert_eq!((left * right).data(), [Tagged(5023), Tagged(5042)]);
    assert_eq!((left / right).data(), [Tagged(7023), Tagged(7042)]);
}

#[test]
fn vec3_value_operators_follow_assignment_path() {
    let left = Vec3::new(Tagged(2), Tagged(4), Tagged(1));
    let right = Vec3::new(Tagged(3), Tagged(2), Tagged(5));
    assert_eq!(
        (left + right).data(),
        [Tagged(2023), Tagged(2042), Tagged(2015)]
    );
    assert_eq!(
        (left - right).data(),
        [Tagged(4023), Tagged(4042), Tagged(4015)]
    );
    assert_eq!(
        (left * right).data(),
        [Tagged(6023), Tagged(6042), Tagged(6015)]
    );
    assert_eq!(
        (left / right).data(),
        [Tagged(8023), Tagged(8042), Tagged(8015)]
    );
}

#[test]
fn vec4_value_operators_follow_assignment_path() {
    let left = Vec4::new(Tagged(2), Tagged(4), Tagged(1), Tagged(6));
    let right = Vec4::new(Tagged(3), Tagged(2), Tagged(5), Tagged(7));
    assert_eq!(
        (left + right).data(),
        [Tagged(2023), Tagged(2042), Tagged(2015), Tagged(2067)]
    );
    assert_eq!(
        (left - right).data(),
        [Tagged(4023), Tagged(4042), Tagged(4015), Tagged(4067)]
    );
    assert_eq!(
        (left * right).data(),
        [Tagged(6023), Tagged(6042), Tagged(6015), Tagged(6067)]
    );
    assert_eq!(
        (left / right).data(),
        [Tagged(8023), Tagged(8042), Tagged(8015), Tagged(8067)]
    );
}

#[test]
fn vec2_compound_operators_use_element_assignment_operators() {
    let mut left = Vec2::new(Tagged(2), Tagged(4));
    let right = Vec2::new(Tagged(3), Tagged(2));
    left += Vec2::new(Tagged(3), Tagged(2));
    assert_eq!(left.data(), [Tagged(2023), Tagged(2042)]);
    left = Vec2::new(Tagged(2), Tagged(4));
    left -= right;
    assert_eq!(left.data(), [Tagged(4023), Tagged(4042)]);
    left = Vec2::new(Tagged(2), Tagged(4));
    left *= right;
    assert_eq!(left.data(), [Tagged(6023), Tagged(6042)]);
    left = Vec2::new(Tagged(2), Tagged(4));
    left /= right;
    assert_eq!(left.data(), [Tagged(8023), Tagged(8042)]);
}

#[test]
fn vec3_compound_operators_use_element_assignment_operators() {
    let mut left = Vec3::new(Tagged(2), Tagged(4), Tagged(1));
    let right = Vec3::new(Tagged(3), Tagged(2), Tagged(5));
    left += right;
    assert_eq!(left.data(), [Tagged(2023), Tagged(2042), Tagged(2015)]);
    left = Vec3::new(Tagged(2), Tagged(4), Tagged(1));
    left -= right;
    assert_eq!(left.data(), [Tagged(4023), Tagged(4042), Tagged(4015)]);
    left = Vec3::new(Tagged(2), Tagged(4), Tagged(1));
    left *= right;
    assert_eq!(left.data(), [Tagged(6023), Tagged(6042), Tagged(6015)]);
    left = Vec3::new(Tagged(2), Tagged(4), Tagged(1));
    left /= right;
    assert_eq!(left.data(), [Tagged(8023), Tagged(8042), Tagged(8015)]);
}

#[test]
fn vec4_compound_add_subtract_use_element_assignment_operators() {
    let mut left = Vec4::new(Tagged(2), Tagged(4), Tagged(1), Tagged(6));
    let right = Vec4::new(Tagged(3), Tagged(2), Tagged(5), Tagged(7));
    left += right;
    assert_eq!(
        left.data(),
        [Tagged(2023), Tagged(2042), Tagged(2015), Tagged(2067)]
    );
    left = Vec4::new(Tagged(2), Tagged(4), Tagged(1), Tagged(6));
    left -= right;
    assert_eq!(
        left.data(),
        [Tagged(4023), Tagged(4042), Tagged(4015), Tagged(4067)]
    );
}

#[test]
fn vec4_compound_multiply_divide_use_element_assignment_operators() {
    let mut left = Vec4::new(Tagged(2), Tagged(4), Tagged(1), Tagged(6));
    let right = Vec4::new(Tagged(3), Tagged(2), Tagged(5), Tagged(7));
    left *= right;
    assert_eq!(
        left.data(),
        [Tagged(6023), Tagged(6042), Tagged(6015), Tagged(6067)]
    );
    left = Vec4::new(Tagged(2), Tagged(4), Tagged(1), Tagged(6));
    left /= right;
    assert_eq!(
        left.data(),
        [Tagged(8023), Tagged(8042), Tagged(8015), Tagged(8067)]
    );
}
