//! Matrix algebra and native OCCT_DEBUG differences beyond OCC22595.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Matrix3, Xyz};
#[test]
fn rows_columns_coefficients_and_profile_checks() {
    let x = Xyz::new(1.0, 2.0, 3.0);
    let y = Xyz::new(4.0, 5.0, 6.0);
    let z = Xyz::new(7.0, 8.0, 9.0);
    let mut matrix = Matrix3::from_columns(x, y, z);
    assert_eq!(matrix.column(1).unwrap(), x);
    matrix.set_rows(x, y, z);
    assert_eq!(matrix.row(2).unwrap(), y);
    matrix.set_columns(x, y, z);
    matrix.set_column(2, x).unwrap();
    matrix.set_row(3, z).unwrap();
    assert_eq!(matrix.coefficients()[2], *z.coordinates());
    matrix.set_value(1, 1, 17.0).unwrap();
    *matrix.change_value(1, 2).unwrap() = 18.0;
    assert_eq!(matrix.value(1, 1).unwrap(), 17.0);
    assert!(matrix.value(0, 1).is_err());
    assert!(matrix.change_value(1, 4).is_err());
    if cfg!(feature = "occt-debug-checks") {
        assert!(matrix.row(0).is_err());
    } else {
        assert_eq!(matrix.row(0).unwrap(), matrix.row(3).unwrap());
    }
}
#[test]
fn algebra_preserves_matrix_vector_and_pre_multiplication_order() {
    let a = Matrix3::new([[1.0, 2.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    let mut b = Matrix3::default();
    b.set_scale(2.0);
    let xyz = Xyz::new(1.0, 2.0, 3.0);
    assert_eq!(a * xyz, Xyz::new(5.0, 2.0, 3.0));
    assert_eq!(xyz * a, a * xyz);
    let mut vector = xyz;
    vector *= a;
    assert_eq!(vector, a * xyz);
    let mut result = a;
    result += b;
    result -= b;
    assert_eq!(result, a);
    result *= 2.0;
    assert_eq!(result, 2.0 * a);
    result.divide(2.0).unwrap();
    assert_eq!(result, a);
    result.pre_multiply(b);
    assert_eq!(result, b * a);
    result.transpose();
    assert_eq!(result, (b * a).transposed());
    let before = result;
    assert!(result.divide(0.0).is_err());
    assert_eq!(result, before);
    assert_eq!(a.determinant(), 1.0);
    assert!(!a.is_singular());
    let mut inverse = a;
    inverse.invert().unwrap();
    assert_eq!(inverse, a.inverted().unwrap());
    let mut identity = Matrix3::default();
    identity.set_identity();
    assert_eq!(a * inverse, identity);
    result = a;
    result.power(3).unwrap();
    assert_eq!(result, a * a * a);
    assert_eq!(a.powered(-1).unwrap(), inverse);
}
#[test]
fn rotation_products_dump_and_singular_profile() {
    let mut matrix = Matrix3::default();
    matrix.set_rotation(Xyz::new(0.0, 0.0, 1.0), 0.0).unwrap();
    assert_eq!(matrix.diagonal(), Xyz::new(1.0, 1.0, 1.0));
    matrix.set_diagonal(2.0, 3.0, 4.0);
    assert_eq!(matrix.diagonal(), Xyz::new(2.0, 3.0, 4.0));
    let vector = Xyz::new(1.0, 2.0, 3.0);
    matrix.set_cross(vector);
    assert_eq!(matrix * Xyz::new(1.0, 0.0, 0.0), Xyz::new(0.0, 3.0, -2.0));
    matrix.set_dot(vector);
    assert_eq!(matrix * vector, vector * 14.0);
    let mut stream = DumpStream::default();
    Matrix3::default().dump_json(&mut stream).unwrap();
    assert_eq!(stream.text(), "\"gp_Mat\": [0, 0, 0, 0, 0, 0, 0, 0, 0]");
    let singular = Matrix3::default().inverted();
    if cfg!(feature = "occt-debug-checks") {
        assert!(singular.is_err());
    } else {
        assert!(singular.unwrap().value(1, 1).unwrap().is_nan());
    }
}
