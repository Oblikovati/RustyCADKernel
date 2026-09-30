//! Direct CSLib_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{Array1, Array2, Sequence};
use rustycad_tkmath::cslib::{
    self, Class2d, Class2dResult, DerivativeStatus, NormalPolyDef, NormalStatus,
};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};
use std::f64::consts::PI;

/// Precision::Infinite().
const PRECISION_INFINITE: f64 = 2.0e100;

/// Precision::IsInfinite(R).
fn is_infinite(value: f64) -> bool {
    value.abs() >= 0.5 * PRECISION_INFINITE
}

fn assert_near(left: f64, right: f64, tolerance: f64, what: &str) {
    assert!(
        (left - right).abs() <= tolerance,
        "{what}: {left} != {right}"
    );
}

// Helper function for comparing directions with tolerance
fn checkDirEqual(theDir1: Direction3, theDir2: Direction3, theTolerance: f64) {
    assert_near(
        theDir1.x(),
        theDir2.x(),
        theTolerance,
        "X components differ",
    );
    assert_near(
        theDir1.y(),
        theDir2.y(),
        theTolerance,
        "Y components differ",
    );
    assert_near(
        theDir1.z(),
        theDir2.z(),
        theTolerance,
        "Z components differ",
    );
}

// Helper function for comparing vectors with tolerance
fn checkVecEqual(theVec1: Vector3, theVec2: Vector3, theTolerance: f64) {
    assert_near(
        theVec1.x(),
        theVec2.x(),
        theTolerance,
        "X components differ",
    );
    assert_near(
        theVec1.y(),
        theVec2.y(),
        theTolerance,
        "Y components differ",
    );
    assert_near(
        theVec1.z(),
        theVec2.z(),
        theTolerance,
        "Z components differ",
    );
}

/// The value a native `gp_Dir aNormal;` output holds: the written normal, else the default +X.
fn outputNormal(theNormal: Option<Direction3>) -> Direction3 {
    theNormal.unwrap_or(Direction3::X)
}

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

fn unitSquare() -> Array1<'static, Point2> {
    let mut aPnts = Array1::with_bounds(1, 4).unwrap();
    aPnts[1] = Point2::new(0.0, 0.0);
    aPnts[2] = Point2::new(1.0, 0.0);
    aPnts[3] = Point2::new(1.0, 1.0);
    aPnts[4] = Point2::new(0.0, 1.0);
    aPnts
}

fn coefficients(values: &[f64]) -> Array1<'static, f64> {
    let mut aLambda = Array1::with_bounds(0, values.len() as i32 - 1).unwrap();
    for (index, value) in values.iter().enumerate() {
        aLambda[index as i32] = *value;
    }
    aLambda
}

include!("cslib/suites_1.rs");
include!("cslib/suites_2.rs");
include!("cslib/suites_3.rs");
include!("cslib/suites_4.rs");
