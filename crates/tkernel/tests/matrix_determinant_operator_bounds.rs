//! Determinants need only element copying and binary arithmetic, matching OCCT templates.
use rustycad_tkernel::ncollection::{Mat3, Mat4};
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug)]
struct ArithmeticOnly(i32);

impl Add for ArithmeticOnly {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Mul for ArithmeticOnly {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Sub for ArithmeticOnly {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

#[test]
fn determinant_accepts_integer_and_arithmetic_only_elements() {
    let values3 = [2, 0, 0, 1, 11, 0, 0, 0, 1];
    let values4 = [2, 0, 0, 0, 1, 11, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
    let mat3_int = Mat3::from_data(values3).determinant();
    let mat3_ops = Mat3::from_data(values3.map(ArithmeticOnly)).determinant().0;
    let mat4_int = Mat4::from_data(values4).determinant_mat3();
    let mat4_ops = Mat4::from_data(values4.map(ArithmeticOnly))
        .determinant_mat3()
        .0;
    let actual = format!(
        "Mat3-int {mat3_int}\nMat3-arithmetic-only {mat3_ops}\nMat4-int {mat4_int}\nMat4-arithmetic-only {mat4_ops}"
    );
    assert_eq!(
        actual,
        include_str!("fixtures/matrix_determinant_operator_bounds.expected").trim_end()
    );
}

fn mapped_rust_item(upstream_id: &str) -> &'static str {
    let mappings = include_str!("../../../porting/parity.json");
    let marker = format!("\"upstream_id\": \"{upstream_id}\"");
    let row = mappings
        .split(&marker)
        .nth(1)
        .expect("mapping selector exists");
    let row_end = row.find("\n    },").expect("mapping row ends");
    row[..row_end]
        .lines()
        .find_map(|line| line.trim().strip_prefix("\"rust_item\": \""))
        .and_then(|item| item.strip_suffix("\","))
        .expect("mapping row has one Rust target")
}

#[test]
fn determinant_selectors_map_to_matching_rust_methods() {
    assert_eq!(
        mapped_rust_item("NCollection_Mat3::Determinant() const noexcept"),
        "rustycad_tkernel::ncollection::Mat3::determinant"
    );
    assert_eq!(
        mapped_rust_item("NCollection_Mat4::DeterminantMat3() const noexcept"),
        "rustycad_tkernel::ncollection::Mat4::determinant_mat3"
    );
}
