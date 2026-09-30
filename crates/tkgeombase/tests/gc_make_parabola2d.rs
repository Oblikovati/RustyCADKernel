//! Direct translation of GC_MakeParabola2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkgeombase::gc::MakeParabola2d;
use rustycad_tkmath::gp::{Axis2d, Direction2, Parabola2, Point2, StandardDirection2};

/// The expected parabola of CheckParabola2d.
struct ExpectedParabola {
    focal: f64,
    vertex: (f64, f64),
    parameter: f64,
    coefficients: [f64; 6],
}

fn CheckParabola2d(
    theAxes: Axis2d,
    theFocus: Point2,
    theSense: bool,
    theExpected: ExpectedParabola,
) {
    let aCompareTol: f64 = 1.0e-12;
    let aPrb = MakeParabola2d::from_directrix(theAxes, theFocus, theSense).unwrap();
    assert!(
        !aPrb.value().unwrap().is_null(),
        "GC_MakeParabola2d should produce a non-null result"
    );
    let aParab: Parabola2 = aPrb.value().unwrap().parab2d().unwrap();
    let aVert: Point2 = aParab.location();
    EXPECT_NEAR(aParab.focal(), theExpected.focal, aCompareTol);
    EXPECT_NEAR(aVert.x(), theExpected.vertex.0, aCompareTol);
    EXPECT_NEAR(aVert.y(), theExpected.vertex.1, aCompareTol);
    EXPECT_NEAR(aParab.parameter(), theExpected.parameter, aCompareTol);
    let aF: [f64; 6] = aParab.coefficients();
    for (value, expected) in aF.iter().zip(theExpected.coefficients) {
        EXPECT_NEAR(*value, expected, aCompareTol);
    }
}

fn y_axis_at(x: f64, y: f64) -> Axis2d {
    Axis2d::new(
        Point2::new(x, y),
        Direction2::from_standard(StandardDirection2::Y),
    )
}

#[test]
fn GC_MakeParabola2d_Test__OCC26747_1_ParabolaOpeningRight() {
    let anAxes = y_axis_at(0.0, 3.0);
    let aFocus = Point2::new(1.0, 3.0);
    let aCoeffs: [f64; 6] = [0.0, 1.0, 0.0, -1.0, -3.0, 10.0];
    let expected = ExpectedParabola {
        focal: 0.5,
        vertex: (0.5, 3.0),
        parameter: 1.0,
        coefficients: aCoeffs,
    };
    CheckParabola2d(anAxes, aFocus, true, expected);
}

#[test]
fn GC_MakeParabola2d_Test__OCC26747_2_ParabolaOpeningLeft() {
    let anAxes = y_axis_at(0.0, 0.0);
    let aFocus = Point2::new(-1.0, 3.0);
    let aCoeffs: [f64; 6] = [0.0, 1.0, 0.0, 1.0, -3.0, 10.0];
    let expected = ExpectedParabola {
        focal: 0.5,
        vertex: (-0.5, 3.0),
        parameter: 1.0,
        coefficients: aCoeffs,
    };
    CheckParabola2d(anAxes, aFocus, false, expected);
}

#[test]
fn GC_MakeParabola2d_Test__OCC26747_3_DegenerateParabola() {
    let anAxes = y_axis_at(0.0, 0.0);
    let aFocus = Point2::new(0.0, 3.0);
    let aCoeffs: [f64; 6] = [0.0, 1.0, 0.0, 0.0, -3.0, 9.0];
    let expected = ExpectedParabola {
        focal: 0.0,
        vertex: (0.0, 3.0),
        parameter: 0.0,
        coefficients: aCoeffs,
    };
    CheckParabola2d(anAxes, aFocus, false, expected);
}
