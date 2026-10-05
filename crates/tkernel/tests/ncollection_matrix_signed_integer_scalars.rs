//! Signed integer matrix operations match OCCT within non-overflowing arithmetic.

use rustycad_tkernel::ncollection::{Mat3, Mat4, MatrixScalar, Vec3};
use std::fmt::Debug;
use std::ops::Neg;

fn diagonal3<T: MatrixScalar>(a: T, b: T, c: T) -> Mat3<T> {
    let zero = T::from(0);
    Mat3::from_data([a, zero, zero, zero, b, zero, zero, zero, c])
}

fn diagonal4<T: MatrixScalar>(a: T, b: T, c: T, d: T) -> Mat4<T> {
    let zero = T::from(0);
    Mat4::from_data([
        a, zero, zero, zero, zero, b, zero, zero, zero, zero, c, zero, zero, zero, zero, d,
    ])
}

fn mat3_unit_inverse<T: MatrixScalar + Debug>() {
    let identity = diagonal3(T::from(1), T::from(1), T::from(1));
    let mut inverse = Mat3::from_data([T::from(7); 9]);
    let mut determinant = T::from(0);

    assert!(identity.inverted_into(&mut inverse, &mut determinant));
    assert_eq!(determinant, T::from(1));
    assert_eq!(inverse.data(), identity.data());
    assert!(identity.inverted_to(&mut inverse));
    assert_eq!(
        identity
            .inverted()
            .expect("unit matrix is invertible")
            .data(),
        identity.data()
    );
}

fn mat3_negative_inverse<T: MatrixScalar + Debug + Neg<Output = T>>() {
    let matrix = diagonal3(-T::from(1), T::from(1), T::from(1));
    let mut inverse = Mat3::zero();
    let mut determinant = T::from(0);

    assert_eq!(matrix.determinant(), -T::from(1));
    assert!(matrix.inverted_into(&mut inverse, &mut determinant));
    assert_eq!(determinant, -T::from(1));
    assert_eq!(inverse.data(), matrix.data());
    assert_eq!(
        matrix.adjoint().data(),
        diagonal3(T::from(1), -T::from(1), -T::from(1)).data()
    );
}

fn mat3_truncating_inverse<T: MatrixScalar + Debug>() {
    let matrix = diagonal3(T::from(2), T::from(1), T::from(1));
    let mut inverse = Mat3::zero();
    let mut determinant = T::from(0);

    assert!(matrix.inverted_into(&mut inverse, &mut determinant));
    assert_eq!(determinant, T::from(2));
    assert_eq!(
        inverse.data(),
        diagonal3(T::from(0), T::from(1), T::from(1)).data()
    );
}

fn mat3_singular_output<T: MatrixScalar + Debug>() {
    let matrix = diagonal3(T::from(0), T::from(1), T::from(1));
    let mut output = Mat3::from_data([T::from(7); 9]);
    let mut determinant = T::from(7);

    assert!(!matrix.inverted_into(&mut output, &mut determinant));
    assert_eq!(determinant, T::from(0));
    assert_eq!(output.data(), [T::from(7); 9]);
    assert!(!matrix.inverted_to(&mut output));
    assert!(matrix.inverted().is_err());
}

fn mat4_unit_inverse<T: MatrixScalar + Debug + Neg<Output = T>>() {
    let identity = diagonal4(T::from(1), T::from(1), T::from(1), T::from(1));
    let mut inverse = Mat4::zero();
    let mut determinant = T::from(0);

    assert!(identity.inverted_into(&mut inverse, &mut determinant));
    assert_eq!(determinant, T::from(1));
    assert_eq!(
        identity.adjoint().data(),
        diagonal4(-T::from(1), -T::from(1), -T::from(1), -T::from(1)).data()
    );
    assert_eq!(inverse.data(), identity.data());
    assert!(identity.inverted_to(&mut inverse));
    assert_eq!(
        identity
            .inverted()
            .expect("unit matrix is invertible")
            .data(),
        identity.data()
    );
}

fn mat4_negative_inverse<T: MatrixScalar + Debug + Neg<Output = T>>() {
    let matrix = diagonal4(-T::from(1), T::from(1), T::from(1), T::from(1));
    let mut inverse = Mat4::zero();
    let mut determinant = T::from(0);

    assert!(matrix.inverted_into(&mut inverse, &mut determinant));
    assert_eq!(determinant, -T::from(1));
    assert_eq!(matrix.adjoint().data(), matrix.data());
    assert_eq!(inverse.data(), matrix.data());
}

fn mat4_reciprocal_truncates<T: MatrixScalar + Debug + Neg<Output = T>>() {
    let matrix = diagonal4(T::from(2), T::from(1), T::from(1), T::from(1));
    let mut inverse = Mat4::from_data([T::from(7); 16]);
    let mut determinant = T::from(0);

    assert!(matrix.inverted_into(&mut inverse, &mut determinant));
    assert_eq!(determinant, T::from(2));
    assert_eq!(inverse.data(), [T::from(0); 16]);
}

fn mat4_singular_cofactors<T: MatrixScalar + Debug + Neg<Output = T>>() {
    let matrix = diagonal4(T::from(0), T::from(1), T::from(1), T::from(1));
    let mut output = Mat4::from_data([T::from(7); 16]);
    let mut determinant = T::from(7);

    assert!(!matrix.inverted_into(&mut output, &mut determinant));
    assert_eq!(determinant, T::from(0));
    assert_eq!(
        output.data(),
        diagonal4(T::from(1), T::from(0), T::from(0), T::from(0)).data()
    );
    assert!(!matrix.inverted_to(&mut output));
    assert!(matrix.inverted().is_err());
}

fn mat4_translation<T: MatrixScalar + Debug + Neg<Output = T>>() {
    let mut matrix = diagonal4(T::from(2), T::from(3), T::from(4), T::from(1));

    matrix.translate(Vec3::new(T::from(1), -T::from(2), T::from(3)));

    assert_eq!(
        matrix.get_column(3).data(),
        [T::from(2), -T::from(6), T::from(12), T::from(1)]
    );
}

#[test]
fn i32_matrix_adjoint_inverse_and_translation_match_native() {
    mat3_unit_inverse::<i32>();
    mat3_negative_inverse::<i32>();
    mat3_truncating_inverse::<i32>();
    mat3_singular_output::<i32>();
    mat4_unit_inverse::<i32>();
    mat4_negative_inverse::<i32>();
    mat4_reciprocal_truncates::<i32>();
    mat4_singular_cofactors::<i32>();
    mat4_translation::<i32>();
}

#[test]
fn i64_matrix_adjoint_inverse_and_translation_match_native() {
    mat3_unit_inverse::<i64>();
    mat3_negative_inverse::<i64>();
    mat3_truncating_inverse::<i64>();
    mat3_singular_output::<i64>();
    mat4_unit_inverse::<i64>();
    mat4_negative_inverse::<i64>();
    mat4_reciprocal_truncates::<i64>();
    mat4_singular_cofactors::<i64>();
    mat4_translation::<i64>();
}
