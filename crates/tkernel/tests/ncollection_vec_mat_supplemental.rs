//! NCollection_Vec2/3/4 and NCollection_Mat4 members beyond the legacy suites:
//! the operator quirks that decide result bits, and matrix access/inversion.
use rustycad_tkernel::ncollection::{Mat4, Vec2, Vec3, Vec4};

#[test]
fn vec3_and_vec4_scalar_division_multiplies_by_the_reciprocal() {
    let value = Vec3::new(1.0_f64, 2.0, 10.0);
    let divided = value / 3.0;
    assert_eq!(divided.z.to_bits(), (10.0 * (1.0 / 3.0_f64)).to_bits());
    assert_ne!(divided.z.to_bits(), (10.0 / 3.0_f64).to_bits());
    let four = Vec4::new(10.0_f32, 0.0, 0.0, 1.0) / 3.0;
    assert_eq!(four.x.to_bits(), (10.0 * (1.0 / 3.0_f32)).to_bits());
}

#[test]
fn vec2_scalar_division_divides_each_component() {
    let divided = Vec2::new(10.0_f64, 1.0) / 3.0;
    assert_eq!(divided.x.to_bits(), (10.0 / 3.0_f64).to_bits());
    let mut assigned = Vec2::new(10.0_f64, 1.0);
    assigned /= 3.0;
    assert_eq!(assigned, divided);
}

#[test]
fn componentwise_operators_and_comparisons_follow_native_ties() {
    let left = Vec3::new(-0.0_f64, 1.0, f64::NAN);
    let right = Vec3::new(0.0_f64, 1.0, 2.0);
    // `a < b ? a : b` keeps the right operand on ties and NaN comparisons.
    let minimum = left.cwise_min(&right);
    assert_eq!(minimum.x.to_bits(), 0.0_f64.to_bits());
    assert_eq!(minimum.z, 2.0);
    assert_eq!(left.cwise_max(&right).z, 2.0);
    let sum = Vec4::new(1, 2, 3, 4) + Vec4::from_value(1);
    assert_eq!(sum.data(), [2, 3, 4, 5]);
    assert_eq!(
        (Vec2::new(6.0, 8.0) / Vec2::new(2.0, 4.0)).data(),
        [3.0, 2.0]
    );
    assert_eq!(Vec3::new(1, 2, 3).dot(&Vec3::new(4, 5, 6)), 32);
    assert_eq!(-Vec2::new(1, -2), Vec2::new(-1, 2));
}

#[test]
fn component_extrema_use_the_native_comparison_trees() {
    assert_eq!(Vec2::new(1, 2).max_comp(), 2);
    assert_eq!(Vec3::new(1, 3, 2).max_comp(), 3);
    assert_eq!(Vec3::new(2, 1, 3).min_comp(), 1);
    assert_eq!(Vec4::new(1, 4, 2, 3).max_comp(), 4);
    assert_eq!(Vec4::new(4, 1, 2, 3).min_comp(), 1);
}

#[test]
fn indexing_matches_named_components() {
    let mut value = Vec4::new(1.0_f64, 2.0, 3.0, 4.0);
    value[3] = 5.0;
    assert_eq!(
        (value[0], value[1], value[2], value.w),
        (1.0, 2.0, 3.0, 5.0)
    );
    assert_eq!(Vec3::<f64>::length(), 3);
}

#[test]
#[should_panic(expected = "component 2: expected < 2")]
fn indexing_past_the_dimension_panics() {
    let value = Vec2::new(1.0_f64, 2.0);
    let _ = value[2];
}

#[test]
fn vec3_metric_members() {
    let mut value = Vec3::new(3.0_f64, 0.0, 4.0);
    assert_eq!(value.modulus(), 5.0);
    value.normalize();
    assert_eq!(value, Vec3::new(0.6, 0.0, 0.8));
    let axes = (Vec3::<f64>::dx(), Vec3::<f64>::dy(), Vec3::<f64>::dz());
    assert_eq!(Vec3::cross(&axes.0, &axes.1), axes.2);
    let middle = Vec3::<f64>::lerp(&Vec3::from_value(0.0_f64), &Vec3::from_value(2.0), 0.5);
    assert_eq!(middle, Vec3::from_value(1.0));
    assert_eq!(Vec2::new(-1.0_f32, 2.0).cwise_abs(), Vec2::new(1.0, 2.0));
}

#[test]
fn mat4_access_by_row_column_and_diagonal() {
    let mut matrix = Mat4::<f64>::identity();
    assert!(matrix.is_identity());
    matrix.set_column3(3, Vec3::new(5.0, 6.0, 7.0));
    matrix.set_value(0, 1, 2.0);
    assert_eq!(matrix.get_column(3), Vec4::new(5.0, 6.0, 7.0, 1.0));
    assert_eq!(matrix.get_row(0), Vec4::new(1.0, 2.0, 0.0, 5.0));
    assert_eq!(matrix.data()[12], 5.0);
    assert_eq!(matrix.transposed().get_value(1, 0), 2.0);
    matrix.set_diagonal(Vec4::new(2.0, 3.0, 4.0, 1.0));
    assert_eq!(matrix.get_diagonal(), Vec4::new(2.0, 3.0, 4.0, 1.0));
    assert!(!matrix.is_identity());
    assert!(Mat4::<f32>::zero().is_zero());
}

#[test]
fn mat4_inversion_scales_the_cofactors_by_the_reciprocal_determinant() {
    let mut matrix = Mat4::<f64>::identity();
    matrix.set_diagonal(Vec4::new(2.0, 4.0, 8.0, 1.0));
    matrix.set_column3(3, Vec3::new(1.0, 2.0, 3.0));
    let inverse = matrix.inverted().unwrap();
    assert_eq!(inverse.get_diagonal(), Vec4::new(0.5, 0.25, 0.125, 1.0));
    assert_eq!(inverse.get_column(3), Vec4::new(-0.5, -0.5, -0.375, 1.0));
    assert!((matrix * inverse).is_identity());
}

#[test]
fn mat4_singular_inversion_reports_and_keeps_unscaled_cofactors() {
    let singular = Mat4::<f64>::zero();
    let failure = singular.inverted().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    let mut output = Mat4::identity();
    let mut determinant = 1.0;
    assert!(!singular.inverted_into(&mut output, &mut determinant));
    assert_eq!(determinant, 0.0);
    assert!(output.is_zero());
}
