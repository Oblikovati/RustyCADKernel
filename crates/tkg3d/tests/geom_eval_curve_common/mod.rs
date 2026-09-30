//! Helpers of the GeomEval curve suites: the one-based `NCollection_Array1` literals
//! they build with `SetValue`, and the three `EXPECT_NEAR` on vector components that
//! they repeat.
#![allow(dead_code)]
use super::common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The three components of `a` and `b` agree within `tolerance` (three native
/// `EXPECT_NEAR`, X then Y then Z); `expect_near_vec(d1, fd, 1e-5)`.
pub fn expect_near_vec(a: Vector3, b: Vector3, tolerance: f64) {
    EXPECT_NEAR(a.x(), b.x(), tolerance);
    EXPECT_NEAR(a.y(), b.y(), tolerance);
    EXPECT_NEAR(a.z(), b.z(), tolerance);
}

/// Poles `(x, y, z)` stored at 1..=n; `poles(&[(0.0, 0.0, 0.0)])`.
pub fn poles(points: &[(f64, f64, f64)]) -> Array1<'static, Point3> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, &(x, y, z)) in points.iter().enumerate() {
        aPoles
            .set_value(i as i32 + 1, Point3::new(x, y, z))
            .unwrap();
    }
    aPoles
}

/// Weights stored at 1..=n; `weights(&[1.0, 2.0, 1.0])`.
pub fn weights(values: &[f64]) -> Array1<'static, f64> {
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, &w) in values.iter().enumerate() {
        aWeights.set_value(i as i32 + 1, w).unwrap();
    }
    aWeights
}
