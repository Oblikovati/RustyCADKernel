//! Checks Mat3/Mat4 component arithmetic with elements that only support += and -=.
use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::ops::{AddAssign, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
struct AssignmentOnly(i32);

impl AddAssign for AssignmentOnly {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for AssignmentOnly {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

fn values<const N: usize>(items: [AssignmentOnly; N]) -> [i32; N] {
    items.map(|item| item.0)
}

fn matrix3(start: i32) -> Mat3<AssignmentOnly> {
    Mat3::from_data(std::array::from_fn(|index| {
        AssignmentOnly(start + index as i32)
    }))
}

fn matrix4(start: i32) -> Mat4<AssignmentOnly> {
    Mat4::from_data(std::array::from_fn(|index| {
        AssignmentOnly(start + index as i32)
    }))
}

const MAT3_SUM: [i32; 9] = [10, 12, 14, 16, 18, 20, 22, 24, 26];
const MAT4_SUM: [i32; 16] = [
    10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40,
];

fn assert_data<const N: usize>(actual: [AssignmentOnly; N], expected: [i32; N]) {
    assert_eq!(values(actual), expected);
}

#[test]
fn mat3_sum_and_difference_require_element_assignment_traits() {
    let left = matrix3(0);
    let right = matrix3(10);
    assert_data(left.added(&right).data(), MAT3_SUM);
    assert_data(left.subtracted(&right).data(), [-10; 9]);
    assert_data((left + right).data(), MAT3_SUM);
    assert_data((left - right).data(), [-10; 9]);
    let mut assigned = left;
    assigned.add(&right);
    assert_data(assigned.data(), MAT3_SUM);
    assigned.subtract(&right);
    assert_data(assigned.data(), std::array::from_fn(|index| index as i32));
    assigned += right;
    assert_data(assigned.data(), MAT3_SUM);
    assigned -= right;
    assert_data(assigned.data(), std::array::from_fn(|index| index as i32));
}

#[test]
fn mat4_sum_and_difference_require_element_assignment_traits() {
    let left = matrix4(0);
    let right = matrix4(10);
    assert_data(left.added(&right).data(), MAT4_SUM);
    assert_data(left.subtracted(&right).data(), [-10; 16]);
    assert_data((left + right).data(), MAT4_SUM);
    assert_data((left - right).data(), [-10; 16]);
    let mut assigned = left;
    assigned.add(&right);
    assert_data(assigned.data(), MAT4_SUM);
    assigned.subtract(&right);
    assert_data(assigned.data(), std::array::from_fn(|index| index as i32));
    assigned += right;
    assert_data(assigned.data(), MAT4_SUM);
    assigned -= right;
    assert_data(assigned.data(), std::array::from_fn(|index| index as i32));
}
