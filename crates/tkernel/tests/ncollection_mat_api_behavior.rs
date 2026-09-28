//! Supplemental NCollection_Mat3/Mat4 API behavior pinned against native OCCT.
use rustycad_tkernel::ncollection::{Mat3, Mat4, Vec3, Vec4};
use rustycad_tkernel::standard::dump::DumpStream;

fn sample4() -> Mat4<f64> {
    Mat4::from_data(std::array::from_fn(|i| (i * i % 7) as f64 - 2.0))
}

#[test]
fn mat4_arithmetic_members() {
    let (a, b) = (sample4(), Mat4::<f64>::identity().multiplied_scalar(2.0));
    assert_eq!((Mat4::<f64>::rows(), Mat4::<f64>::cols()), (4, 4));
    assert_eq!(Mat4::product(&a, &b), a * b);
    assert_eq!(a.multiplied(&b), a * 2.0);
    let mut c = a;
    c.multiply(&b);
    c.divide(2.0);
    assert_eq!(c, a);
    assert_eq!(a.divided(2.0), a / 2.0);
    assert_eq!(a.added(&a), a * 2.0);
    assert_eq!(a.subtracted(&a), Mat4::zero());
    assert_eq!(-a, a.negated());
    assert_eq!(a + a - a, a);
    let mut d = a;
    d += a;
    d -= a;
    d *= 2.0;
    d /= 2.0;
    d *= Mat4::identity();
    assert_eq!(d, a);
    d.transpose();
    assert_eq!(d, a.transposed());
    assert!(a.is_equal(&a) && !a.is_equal(&b));
}

#[test]
fn mat4_product_starts_each_element_at_zero() {
    // Row 0 of `a` is all -1, so every product with a zero is -0.
    let mut a = Mat4::<f64>::zero();
    a.set_row(0, Vec4::new(-1.0, -1.0, -1.0, -1.0));
    // Native Multiply sums from (Element_t)0, and 0 + -0 is +0.
    assert!(
        Mat4::product(&a, &Mat4::zero())
            .get_value(0, 0)
            .is_sign_positive()
    );
    // Matrix * vector starts from the first product, so the -0 terms stay -0.
    let v = a * Vec4::new(0.0, 0.0, 0.0, 0.0);
    assert!(v.x.is_sign_negative());
}

#[test]
fn mat4_adjoint_determinant_and_blocks() {
    let a = sample4();
    let adjoint = a.adjoint();
    let mut inverse = Mat4::default();
    let mut determinant = 0.0;
    assert!(a.inverted_into(&mut inverse, &mut determinant));
    // Native Adjoint is the negated adjugate: Adjoint(I) = -I, and inverse = -Adjoint^T / det.
    assert_eq!(Mat4::<f64>::identity().adjoint(), -Mat4::<f64>::identity());
    let scaled = adjoint.transposed().multiplied_scalar(-1.0 / determinant);
    for (x, y) in scaled.data().iter().zip(inverse.data()) {
        assert!((x - y).abs() < 1e-12);
    }
    let mut other = Mat4::default();
    assert!(a.inverted_to(&mut other));
    assert_eq!(other, inverse);
    let block: Mat3<f64> = a.get_mat3();
    assert_eq!(block.get_row(2), a.get_row(2).xyz());
    assert_eq!(block.determinant(), a.determinant_mat3());
    let mut changed = a;
    changed.change_data()[3][0] = 42.0;
    assert_eq!(changed.get_value(0, 3), 42.0);
    let mut single = Mat4::<f32>::default();
    single.convert_from(&a);
    assert_eq!(single.get_value(1, 2), a.get_value(1, 2) as f32);
    assert_eq!(Mat4::<f32>::from_other(&a), single);
}

#[test]
fn mat3_members() {
    let mut a = Mat3::<f64>::identity();
    a.set_row(0, Vec3::new(2.0, 1.0, 0.0));
    a.set_column(2, Vec3::new(0.0, 0.0, 4.0));
    assert_eq!(a.determinant(), 8.0);
    let inverse = a.inverted().unwrap();
    assert_eq!(a * inverse, Mat3::identity());
    // A singular matrix leaves the output untouched, unlike Mat4.
    let singular = Mat3::<f64>::zero();
    let mut output = a;
    let mut determinant = 5.0;
    assert!(!singular.inverted_into(&mut output, &mut determinant));
    assert_eq!((output, determinant), (a, 0.0));
    assert!(!singular.inverted_to(&mut output));
    assert_eq!(
        singular.inverted().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        a.adjoint().get_row(0),
        Vec3::cross(&a.get_row(1), &a.get_row(2))
    );
    assert_eq!(a * Vec3::new(1.0, 1.0, 1.0), Vec3::new(3.0, 1.0, 4.0));
    assert!(Mat3::<f64>::zero().is_zero() && Mat3::<f64>::identity().is_identity());
    assert_eq!(Mat3::product(&a, &inverse), a.multiplied(&inverse));
    assert_eq!(a.get_diagonal(), Vec3::new(2.0, 1.0, 4.0));
    assert_eq!(Mat3::from_data(a.data()), a);
}

#[test]
fn matrix_json_lists_rows() {
    let mut stream = DumpStream::default();
    Mat3::<f32>::identity().dump_json(&mut stream).unwrap();
    assert_eq!(
        stream.text(),
        "\"NCollection_Mat3\": [1, 0, 0, 0, 1, 0, 0, 0, 1]"
    );
    let mut matrix = Mat4::<f64>::identity();
    matrix.set_value(0, 3, 5.0);
    let mut stream = DumpStream::default();
    matrix.dump_json(&mut stream).unwrap();
    assert!(
        stream
            .text()
            .starts_with("\"NCollection_Mat4\": [1, 0, 0, 5, 0, 1")
    );
}
