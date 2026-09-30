//! The statements that the GeomEval elementary surface suites repeat verbatim:
//! the central differences `(a.XYZ() - b.XYZ()) / (2.0 * Precision::Confusion())`,
//! the per-coordinate `EXPECT_NEAR` triples and the implicit-equation residual.
#![allow(non_snake_case, dead_code)]
use super::common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};

/// `gp_Vec((a.XYZ() - b.XYZ()) / (2.0 * Precision::Confusion()))`.
pub fn central_difference(forward: Xyz, backward: Xyz) -> Xyz {
    (forward - backward) / (2.0 * precision::confusion())
}

/// The finite difference of two points.
pub fn point_difference(forward: Point3, backward: Point3) -> Xyz {
    central_difference(forward.xyz(), backward.xyz())
}

/// The finite difference of two vectors.
pub fn vector_difference(forward: Vector3, backward: Vector3) -> Xyz {
    central_difference(*forward.xyz(), *backward.xyz())
}

/// `EXPECT_NEAR` on X, then Y, then Z.
pub fn EXPECT_XYZ_NEAR(actual: Xyz, expected: Xyz, tolerance: f64) {
    EXPECT_NEAR(actual.x(), expected.x(), tolerance);
    EXPECT_NEAR(actual.y(), expected.y(), tolerance);
    EXPECT_NEAR(actual.z(), expected.z(), tolerance);
}

/// `EXPECT_NEAR` on the coordinates of a vector.
pub fn EXPECT_VEC_NEAR(actual: Vector3, expected: Xyz, tolerance: f64) {
    EXPECT_XYZ_NEAR(*actual.xyz(), expected, tolerance);
}

/// `A1 X^2 + A2 Y^2 + A3 Z^2 + 2 (B1 XY + B2 XZ + B3 YZ) + 2 (C1 X + C2 Y + C3 Z) + D`,
/// written as the suites write it.
pub fn quadric_residual(coefficients: [f64; 10], point: Point3) -> f64 {
    let [aA1, aA2, aA3, aB1, aB2, aB3, aC1, aC2, aC3, aD] = coefficients;
    let (aX, aY, aZ) = (point.x(), point.y(), point.z());
    aA1 * aX * aX
        + aA2 * aY * aY
        + aA3 * aZ * aZ
        + 2.0 * (aB1 * aX * aY + aB2 * aX * aZ + aB3 * aY * aZ)
        + 2.0 * (aC1 * aX + aC2 * aY + aC3 * aZ)
        + aD
}
