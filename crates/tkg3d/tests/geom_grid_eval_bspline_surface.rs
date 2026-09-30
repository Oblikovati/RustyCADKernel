//! Direct translation of GeomGridEval_BSplineSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::geom_grid_eval::{
    GeomGridEvalBSplineSurface, GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3,
};
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-10;

fn CreateSimpleBSplineSurface() -> Handle<GeomBSplineSurface> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aPoles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    aPoles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    aPoles[(1, 2)] = Point3::new(0.0, 1.0, 0.0);
    aPoles[(2, 2)] = Point3::new(1.0, 1.0, 1.0); // Non-planar corner

    let mut aUKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aVKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aUMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    let mut aVMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();

    aUKnots[1] = 0.0;
    aUKnots[2] = 1.0;
    aVKnots[1] = 0.0;
    aVKnots[2] = 1.0;
    aUMults[1] = 2;
    aUMults[2] = 2;
    aVMults[1] = 2;
    aVMults[2] = 2;

    Handle::new(
        GeomBSplineSurface::new(
            &aPoles,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    )
}

fn CreateRationalBSplineSurface() -> Handle<GeomBSplineSurface> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 3).unwrap();

    for i in 1..=3 {
        for j in 1..=3 {
            let x: f64 = f64::from(i - 1) * 1.0;
            let y: f64 = f64::from(j - 1) * 1.0;
            let z: f64 = if i == 2 && j == 2 { 1.0 } else { 0.0 };
            aPoles[(i, j)] = Point3::new(x, y, z);
            aWeights[(i, j)] = if i == 2 && j == 2 { 2.0 } else { 1.0 };
        }
    }

    let mut aUKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aVKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aUMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    let mut aVMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();

    aUKnots[1] = 0.0;
    aUKnots[2] = 1.0;
    aVKnots[1] = 0.0;
    aVKnots[2] = 1.0;
    aUMults[1] = 3;
    aUMults[2] = 3;
    aVMults[1] = 3;
    aVMults[2] = 3;

    Handle::new(
        GeomBSplineSurface::with_weights(
            &aPoles,
            &aWeights,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (2, 2),
            (false, false),
        )
        .unwrap(),
    )
}

fn CreateMultiSpanBSplineSurface() -> Handle<GeomBSplineSurface> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 5, 1, 5).unwrap();

    for i in 1..=5 {
        for j in 1..=5 {
            let x: f64 = f64::from(i - 1) * 1.0;
            let y: f64 = f64::from(j - 1) * 1.0;
            let z: f64 = (f64::from(i - 1) * PI / 4.0).sin() * (f64::from(j - 1) * PI / 4.0).sin();
            aPoles[(i, j)] = Point3::new(x, y, z);
        }
    }

    let mut aUKnots: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    let mut aVKnots: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    let mut aUMults: Array1<'static, i32> = Array1::with_bounds(1, 3).unwrap();
    let mut aVMults: Array1<'static, i32> = Array1::with_bounds(1, 3).unwrap();

    aUKnots[1] = 0.0;
    aUKnots[2] = 0.5;
    aUKnots[3] = 1.0;
    aVKnots[1] = 0.0;
    aVKnots[2] = 0.5;
    aVKnots[3] = 1.0;
    aUMults[1] = 3;
    aUMults[2] = 2;
    aUMults[3] = 3;
    aVMults[1] = 3;
    aVMults[2] = 2;
    aVMults[3] = 3;

    Handle::new(
        GeomBSplineSurface::new(
            &aPoles,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (2, 2),
            (false, false),
        )
        .unwrap(),
    )
}

include!("geom_grid_eval_bspline_surface/suites_0.rs");
include!("geom_grid_eval_bspline_surface/suites_1.rs");
