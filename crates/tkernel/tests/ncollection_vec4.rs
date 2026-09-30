//! Translations of all five legacy Vec4 declarations.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{Mat4, Vec3, Vec4};
#[test]
fn NCollection_Vec4Test__BasicConstruction() {
    let first = Vec4::<f32>::default();
    assert_eq!((first.x, first.y, first.z, first.w), (0.0, 0.0, 0.0, 0.0));
    let second = Vec4::new(1.0_f32, 2.0, 3.0, 4.0);
    assert_eq!(
        (second.x, second.y, second.z, second.w),
        (1.0, 2.0, 3.0, 4.0)
    );
}
#[test]
fn NCollection_Vec4Test__XyzMethod() {
    let vector = Vec4::new(2.0_f32, 3.0, 4.0, 5.0);
    let three = vector.xyz();
    assert_eq!((three.x, three.y, three.z), (2.0, 3.0, 4.0));
}
#[test]
fn NCollection_Vec4Test__MatrixMultiplicationAndTransformation() {
    let mut matrix = Mat4::<f32>::default();
    matrix.translate(Vec3::new(4.0, 3.0, 1.0));
    let mut points: [Vec4<f32>; 8] = std::array::from_fn(|index| {
        Vec4::new(
            -1.0 + 2.0 * (index / 4) as f32,
            -1.0 + 2.0 * ((index / 2) % 2) as f32,
            -1.0 + 2.0 * (index % 2) as f32,
            1.0,
        )
    });
    let mut result = [Vec3::<f32>::default(); 8];
    for index in 0..8 {
        points[index] = matrix * points[index];
        result[index] = points[index].xyz() / points[index].w;
    }
    assert!((result[7].square_modulus() - 45.0).abs() <= 0.001);
    let expected = 3.0 * 3.0 + 2.0 * 2.0 + 0.0 * 0.0;
    assert!((result[0].square_modulus() - expected).abs() <= 0.001);
}
#[test]
fn NCollection_Vec4Test__ComponentAccess() {
    let mut vector = Vec4::new(1.5_f32, 2.5, 3.5, 4.5);
    assert_eq!(
        (vector.x, vector.y, vector.z, vector.w),
        (1.5, 2.5, 3.5, 4.5)
    );
    vector.x = 10.0;
    vector.y = 20.0;
    vector.z = 30.0;
    vector.w = 40.0;
    assert_eq!(
        (vector.x, vector.y, vector.z, vector.w),
        (10.0, 20.0, 30.0, 40.0)
    );
}
#[test]
fn NCollection_Vec4Test__HomogeneousCoordinateDivision() {
    let vector = Vec4::new(8.0_f32, 12.0, 16.0, 4.0);
    let result = vector.xyz() / vector.w;
    assert_eq!((result.x, result.y, result.z), (2.0, 3.0, 4.0));
}
