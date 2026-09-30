//! Original OCC26746 surface sampling and sequential quartic evaluation from
//! gp_Torus_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg3d::geom::{GeomSurface, GeomToroidalSurface};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use std::f64::consts::PI;

// Retain the OCC26746 decimal inputs verbatim from the pinned regression.
#[allow(clippy::excessive_precision)]
fn torus_legacy_surface() -> GeomToroidalSurface {
    let frame = Axis3::new(
        Point3::new(55.52514413, 2.070076585, 73.83409062),
        Direction3::new(
            0.37231784651136368,
            0.58886674834874120,
            0.71736697293527607,
        )
        .unwrap(),
        Direction3::new(
            0.80682335496555135,
            0.17666016102759910,
            -0.56376170618524390,
        )
        .unwrap(),
    )
    .unwrap();
    GeomToroidalSurface::new(frame, 87.08479625, 23.14682176).unwrap()
}
fn torus_powers(value: f64) -> [f64; 4] {
    let square = value * value;
    [value, square, square * value, square * square]
}

fn quartic_group_0(
    coefficients: &Array1<'_, f64>,
    x: [f64; 4],
    y: [f64; 4],
    z: [f64; 4],
    delta: &mut f64,
) {
    *delta += coefficients[5] * x[3];
    *delta += coefficients[6] * y[3];
    *delta += coefficients[7] * z[3];
    *delta += coefficients[8] * x[2] * y[0];
    *delta += coefficients[9] * x[2] * z[0];
    *delta += coefficients[10] * y[2] * x[0];
    *delta += coefficients[11] * y[2] * z[0];
    *delta += coefficients[12] * z[2] * x[0];
    *delta += coefficients[13] * z[2] * y[0];
    *delta += coefficients[14] * x[1] * y[1];
    *delta += coefficients[15] * x[1] * z[1];
    *delta += coefficients[16] * y[1] * z[1];
}

fn quartic_group_1(
    coefficients: &Array1<'_, f64>,
    x: [f64; 4],
    y: [f64; 4],
    z: [f64; 4],
    delta: &mut f64,
) {
    *delta += coefficients[17] * x[1] * y[0] * z[0];
    *delta += coefficients[18] * x[0] * y[1] * z[0];
    *delta += coefficients[19] * x[0] * y[0] * z[1];
    *delta += coefficients[20] * x[2];
    *delta += coefficients[21] * y[2];
    *delta += coefficients[22] * z[2];
    *delta += coefficients[23] * x[1] * y[0];
    *delta += coefficients[24] * x[1] * z[0];
    *delta += coefficients[25] * y[1] * x[0];
    *delta += coefficients[26] * y[1] * z[0];
    *delta += coefficients[27] * z[1] * x[0];
    *delta += coefficients[28] * z[1] * y[0];
}

fn quartic_group_2(
    coefficients: &Array1<'_, f64>,
    x: [f64; 4],
    y: [f64; 4],
    z: [f64; 4],
    delta: &mut f64,
) {
    *delta += coefficients[29] * x[0] * y[0] * z[0];
    *delta += coefficients[30] * x[1];
    *delta += coefficients[31] * y[1];
    *delta += coefficients[32] * z[1];
    *delta += coefficients[33] * x[0] * y[0];
    *delta += coefficients[34] * x[0] * z[0];
    *delta += coefficients[35] * y[0] * z[0];
    *delta += coefficients[36] * x[0];
    *delta += coefficients[37] * y[0];
    *delta += coefficients[38] * z[0];
    *delta += coefficients[39];
}

fn torus_residual(coefficients: &Array1<'_, f64>, point: Point3) -> f64 {
    let (x, y, z) = (
        torus_powers(point.x()),
        torus_powers(point.y()),
        torus_powers(point.z()),
    );
    let mut delta = 0.0;
    quartic_group_0(coefficients, x, y, z, &mut delta);
    quartic_group_1(coefficients, x, y, z, &mut delta);
    quartic_group_2(coefficients, x, y, z, &mut delta);
    delta
}
fn check_torus_v_samples(surface: &GeomToroidalSurface, coefficients: &Array1<'_, f64>, u: f64) {
    let mut v = 0.0;
    for index in 0..=5 {
        let point = surface.value(u, v).unwrap();
        let delta = torus_residual(coefficients, point);
        assert!(
            delta.abs() <= 3e-7,
            "Torus residual {delta} at ({u},{v}); expected within 3e-7 of zero"
        );
        v = if index == 5 {
            2.0 * PI
        } else {
            v + 2.0 * PI / 5.0
        };
    }
}
#[test]
fn gp_TorusTest__OCC26746_CoefficientsCorrect() {
    let surface = torus_legacy_surface();
    let mut coefficients = Array1::with_bounds(5, 39).unwrap();
    surface
        .torus()
        .unwrap()
        .coefficients(&mut coefficients)
        .unwrap();
    let mut u = 0.0;
    for index in 0..=5 {
        check_torus_v_samples(&surface, &coefficients, u);
        u = if index == 5 {
            2.0 * PI
        } else {
            u + 2.0 * PI / 5.0
        };
    }
}
