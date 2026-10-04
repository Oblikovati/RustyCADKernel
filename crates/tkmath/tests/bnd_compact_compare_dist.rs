//! Direct Bnd_B2/B3 protected-hook behavior compared with tools/oracle output.

use rustycad_tkmath::{bnd::CompactBox2, bnd::CompactBox3, gp::Xy, gp::Xyz};

#[test]
fn bnd_b2_compare_dist_matches_native_strict_scalar_predicate() {
    assert!(!CompactBox2::<f64>::compare_dist(&[1.0, 2.0], &[1.0, -2.0]));
    assert!(CompactBox2::<f64>::compare_dist(
        &[1.0, 2.0],
        &[1.0001, 0.0]
    ));
    assert!(!CompactBox2::<f64>::compare_dist(
        &[1.0, 2.0],
        &[f64::NAN, 0.0]
    ));
    assert!(CompactBox2::<f64>::compare_dist(&[-1.0, 0.0], &[0.0, 0.0]));
    assert!(!CompactBox2::<f32>::compare_dist(&[1.0, 2.0], &[1.0, -2.0]));
}

#[test]
fn bnd_b3_compare_dist_matches_native_strict_scalar_predicate() {
    assert!(!CompactBox3::<f64>::compare_dist(
        &[1.0, 2.0, 3.0],
        &[-1.0, 2.0, 3.0]
    ));
    assert!(CompactBox3::<f64>::compare_dist(
        &[1.0, 2.0, 3.0],
        &[0.0, 0.0, 3.0001]
    ));
    assert!(!CompactBox3::<f64>::compare_dist(
        &[1.0, 2.0, 3.0],
        &[0.0, f64::NAN, 0.0]
    ));
    assert!(!CompactBox3::<f32>::compare_dist(
        &[1.0, 2.0, 3.0],
        &[-1.0, 2.0, 3.0]
    ));
}

#[test]
fn bnd_b2_compare_dist_d_uses_double_coordinates_for_float_boxes() {
    assert!(!CompactBox2::<f32>::compare_dist_d(
        Xy::new(1.0, 2.0),
        Xy::new(-1.0, 2.0)
    ));
    assert!(CompactBox2::<f32>::compare_dist_d(
        Xy::new(1.0, 2.0),
        Xy::new(1.0, 2.1)
    ));
    assert!(!CompactBox2::<f32>::compare_dist_d(
        Xy::new(1.0, 2.0),
        Xy::new(f64::NAN, 0.0)
    ));
}

#[test]
fn bnd_b3_compare_dist_d_uses_double_coordinates_for_float_boxes() {
    assert!(!CompactBox3::<f32>::compare_dist_d(
        Xyz::new(1.0, 2.0, 3.0),
        Xyz::new(0.0, -2.0, 3.0)
    ));
    assert!(CompactBox3::<f32>::compare_dist_d(
        Xyz::new(1.0, 2.0, 3.0),
        Xyz::new(0.0, 0.0, 3.1)
    ));
    assert!(!CompactBox3::<f32>::compare_dist_d(
        Xyz::new(1.0, 2.0, 3.0),
        Xyz::new(0.0, f64::NAN, 0.0)
    ));
}
