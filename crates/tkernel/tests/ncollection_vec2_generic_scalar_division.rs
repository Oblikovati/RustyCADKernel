//! Regressions for generic NCollection_Vec2 scalar division paths.
use rustycad_tkernel::ncollection::Vec2;
use std::ops::{Div, DivAssign};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Tagged(i32);

impl Div for Tagged {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self(1000 + self.0 * 10 + rhs.0)
    }
}

impl DivAssign for Tagged {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = 2000 + self.0 * 10 + rhs.0;
    }
}

#[test]
fn vec2_scalar_value_division_uses_element_div() {
    let result = Vec2::new(Tagged(2), Tagged(4)) / Tagged(3);
    assert_eq!(result, Vec2::new(Tagged(1023), Tagged(1043)));
}

#[test]
fn vec2_scalar_assignment_uses_element_div_assign() {
    let mut value = Vec2::new(Tagged(2), Tagged(4));
    value /= Tagged(3);
    assert_eq!(value, Vec2::new(Tagged(2023), Tagged(2043)));
}

#[test]
fn vec2_integer_scalar_division_remains_supported() {
    assert_eq!(Vec2::new(15_i32, -9) / 3, Vec2::new(5, -3));
    let mut value = Vec2::new(15_i32, -9);
    value /= 3;
    assert_eq!(value, Vec2::new(5, -3));
}

impl rustycad_tkernel::ncollection::VectorArithmeticComponent for Tagged {}
