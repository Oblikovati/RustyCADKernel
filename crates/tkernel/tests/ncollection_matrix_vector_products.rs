//! Matrix-vector products retain only the element operations used by OCCT.

use rustycad_tkernel::ncollection::{Mat3, Mat4, Vec3, Vec4};
use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug)]
struct BinaryOnly(i32);

impl Add for BinaryOnly {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Mul for BinaryOnly {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}

#[test]
fn mat3_vector_product_uses_only_copy_add_and_multiply() {
    let matrix = Mat3::from_data(std::array::from_fn(|index| BinaryOnly(index as i32 + 1)));
    let vector = Vec3::new(BinaryOnly(2), BinaryOnly(3), BinaryOnly(4));

    let product = matrix * vector;

    assert_eq!([product.x.0, product.y.0, product.z.0], [42, 51, 60]);
}

#[test]
fn mat4_vector_product_uses_only_copy_add_and_multiply() {
    let matrix = Mat4::from_data(std::array::from_fn(|index| BinaryOnly(index as i32 + 1)));
    let vector = Vec4::new(BinaryOnly(1), BinaryOnly(2), BinaryOnly(3), BinaryOnly(4));

    let product = matrix * vector;

    assert_eq!(
        [product.x.0, product.y.0, product.z.0, product.w.0],
        [90, 100, 110, 120]
    );
}

#[test]
fn signed_integer_products_preserve_native_column_major_order() {
    let mat3 = Mat3::from_data(std::array::from_fn(|index| index as i32 + 1));
    let mat4 = Mat4::from_data(std::array::from_fn(|index| index as i32 + 1));

    let product3 = mat3 * Vec3::new(2, 3, 4);
    let product4 = mat4 * Vec4::new(1, 2, 3, 4);

    assert_eq!(product3, Vec3::new(42, 51, 60));
    assert_eq!(product4, Vec4::new(90, 100, 110, 120));
}
