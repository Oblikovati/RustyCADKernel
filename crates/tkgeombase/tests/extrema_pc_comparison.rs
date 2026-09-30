//! Direct translation of ExtremaPC_Comparison_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871: the new ExtremaPC against the classic
//! Extrema_ExtPC ([`ExtPC`]).
#![allow(non_snake_case, dead_code)]
mod common;
mod extrema_pc_random;
#[allow(unused_imports)]
use common::*;
use extrema_pc_random::random_points;
#[allow(unused_imports)]
use prelude::*;
use rustycad_tkgeombase::extrema::ExtPC;

// Test fixture for comparison tests between ExtremaPC and Extrema_ExtPC.
const THE_TOL: f64 = 1.0e-6;
const THE_TOL_REL: f64 = 1.0e-4; // Relative tolerance for distance comparison

/// `Extrema_ExtPC(P, C)` with the native default TolF.
fn old_ext_pc<'a>(point: Point3, adaptor: &'a GeomAdaptorCurve) -> ExtPC<'a> {
    ExtPC::with_curve(point, adaptor, 1.0e-10).unwrap()
}

// Compare minimum distances found by both implementations.
fn CompareMinDistance(theNew: &ExtremaResult, theOld: &ExtPC, theTestName: &str) {
    // Both must complete successfully
    assert!(theNew.is_done(), "{theTestName}: New implementation failed");
    assert!(theOld.is_done(), "{theTestName}: Old implementation failed");

    // Find minimum from old implementation
    let mut aOldMinSqDist: f64 = f64::MAX;
    let mut aOldMinIdx: i32 = -1;
    for i in 1..=theOld.nb_ext().unwrap() {
        let aSqDist: f64 = theOld.square_distance(i).unwrap();
        if aSqDist < aOldMinSqDist {
            aOldMinSqDist = aSqDist;
            aOldMinIdx = i;
        }
    }

    // Find minimum from new implementation
    let aNewMinSqDist: f64 = theNew.min_square_distance();
    let aNewMinIdx: usize = theNew.min_index();
    let hasOldMinimum: bool = aOldMinIdx >= 1;
    let hasNewMinimum: bool = aNewMinIdx != ExtremaResult::INVALID_INDEX;

    // Both should find extrema (or both find none)
    if !hasOldMinimum && !hasNewMinimum {
        return; // Both found no extrema - OK
    }

    assert!(
        hasOldMinimum,
        "{theTestName}: Old implementation found no minimum"
    );
    assert!(
        hasNewMinimum,
        "{theTestName}: New implementation found no minimum"
    );

    // Compare minimum distances
    let aOldDist: f64 = aOldMinSqDist.sqrt();
    let aNewDist: f64 = aNewMinSqDist.sqrt();

    // Use relative tolerance for non-zero distances
    if aOldDist > THE_TOL {
        let aRelDiff: f64 = (aNewDist - aOldDist).abs() / aOldDist;
        EXPECT_LT(aRelDiff, THE_TOL_REL);
    } else {
        EXPECT_NEAR(aNewDist, aOldDist, THE_TOL);
    }

    // Compare parameters (approximately)
    let aOldParam: f64 = theOld.point(aOldMinIdx).unwrap().parameter();
    let aNewParam: f64 = theNew[aNewMinIdx].parameter;

    // Parameters should be close (unless there are multiple minima with same distance)
    if (aNewDist - aOldDist).abs() < THE_TOL {
        // Only check parameter if distances are nearly identical
        let aParamDiff: f64 = (aNewParam - aOldParam).abs();
        // Allow some tolerance since both are valid minima
        if aParamDiff > 0.1 {
            // More than 0.1 parameter difference
            // Verify that both are actually valid minima by checking distance
            EXPECT_NEAR(aNewDist, aOldDist, THE_TOL);
        }
    }
}

// Verify that new implementation finds distance at least as good as old.
fn VerifyNewNotWorse(theNew: &ExtremaResult, theOld: &ExtPC, theTestName: &str) {
    if !theOld.is_done() || theOld.nb_ext().unwrap() == 0 {
        return; // Old implementation found nothing - nothing to compare
    }

    assert!(theNew.is_done(), "{theTestName}: New implementation failed");

    // Find minimum from old implementation
    let mut aOldMinSqDist: f64 = f64::MAX;
    for i in 1..=theOld.nb_ext().unwrap() {
        let aSqDist: f64 = theOld.square_distance(i).unwrap();
        if aSqDist < aOldMinSqDist {
            aOldMinSqDist = aSqDist;
        }
    }

    // New implementation should find distance at least as good
    if theNew.nb_ext() > 0 {
        let aNewMinSqDist: f64 = theNew.min_square_distance();
        // New should be <= old (with tolerance)
        EXPECT_LE(
            aNewMinSqDist,
            aOldMinSqDist * (1.0 + THE_TOL_REL) + THE_TOL * THE_TOL,
        );
    }
}

/// `gp_Ax2(P, D)`.
fn ax2(p: Point3, d: Direction3) -> Axis2 {
    Axis2::new(p, d).unwrap()
}

/// The cubic B-spline (0,0)-(1,2)-(2,2)-(3,0) of several tests.
fn cubic_bspline() -> Handle<GeomBSplineCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(2.0, 2.0, 0.0);
    aPoles[4] = Point3::new(3.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

include!("extrema_pc_comparison/suites_0.rs");
include!("extrema_pc_comparison/suites_1.rs");
