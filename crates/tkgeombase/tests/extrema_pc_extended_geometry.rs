// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of ExtremaPC_ExtendedGeometry_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871: ExtremaPC against the classic
//! Extrema_ExtPC ([`ExtPC`]) on transformed, degenerate and random configurations.
#![allow(non_snake_case, dead_code)]
mod common;
mod extrema_pc_random;
#[allow(unused_imports)]
use common::*;
use extrema_pc_random::random_points;
#[allow(unused_imports)]
use prelude::*;
use rustycad_tkgeombase::extrema::ExtPC;

const THE_TOL: f64 = 1.0e-6;

//==================================================================================================
// Test fixture for extended geometry tests
//==================================================================================================

/// `Extrema_ExtPC(P, C)` with the native default TolF.
fn old_ext_pc<'a>(point: Point3, adaptor: &'a GeomAdaptorCurve) -> ExtPC<'a> {
    ExtPC::with_curve(point, adaptor, 1.0e-10).unwrap()
}

// Compares minimum distance between new and old implementations
fn CompareMinDistance(theNew: &ExtremaResult, theOld: &ExtPC, theTestName: &str, theTol: f64) {
    if !theOld.is_done() || theOld.nb_ext().unwrap() == 0 {
        return;
    }

    assert!(theNew.is_done(), "{theTestName}: New implementation failed");
    assert!(theNew.nb_ext() > 0, "{theTestName}: No extrema found");

    let mut aOldMinSqDist: f64 = f64::MAX;
    for i in 1..=theOld.nb_ext().unwrap() {
        if theOld.square_distance(i).unwrap() < aOldMinSqDist {
            aOldMinSqDist = theOld.square_distance(i).unwrap();
        }
    }

    let aNewMinDist: f64 = theNew.min_square_distance().sqrt();
    let aOldMinDist: f64 = aOldMinSqDist.sqrt();

    EXPECT_NEAR(aNewMinDist, aOldMinDist, theTol);
}

// Compares maximum distance between new and old implementations
fn CompareMaxDistance(theNew: &ExtremaResult, theOld: &ExtPC, theTestName: &str, theTol: f64) {
    if !theOld.is_done() || theOld.nb_ext().unwrap() == 0 {
        return;
    }

    assert!(theNew.is_done(), "{theTestName}: New implementation failed");
    assert!(theNew.nb_ext() > 0, "{theTestName}: No extrema found");

    let mut aOldMaxSqDist: f64 = 0.0;
    for i in 1..=theOld.nb_ext().unwrap() {
        if theOld.square_distance(i).unwrap() > aOldMaxSqDist {
            aOldMaxSqDist = theOld.square_distance(i).unwrap();
        }
    }

    let aNewMaxDist: f64 = theNew.max_square_distance().sqrt();
    let aOldMaxDist: f64 = aOldMaxSqDist.sqrt();

    EXPECT_NEAR(aNewMaxDist, aOldMaxDist, theTol);
}

// Compares both min and max distances
fn CompareMinMax(theNew: &ExtremaResult, theOld: &ExtPC, theTestName: &str, theTol: f64) {
    CompareMinDistance(theNew, theOld, theTestName, theTol);
    CompareMaxDistance(theNew, theOld, theTestName, theTol);
}

/// `gp_Ax2(P, D)`.
fn ax2(p: Point3, d: Direction3) -> Axis2 {
    Axis2::new(p, d).unwrap()
}

/// A Geom curve handle as a whole-range adaptor.
fn whole<C>(curve: C) -> GeomAdaptorCurve
where
    C: rustycad_tkernel::standard::RuntimeTyped + rustycad_tkernel::standard::Transient,
{
    GeomAdaptorCurve::with_curve(&Handle::new(curve).cast()).unwrap()
}

/// A Geom curve handle as an adaptor on `[u1, u2]`.
fn ranged<C>(curve: C, u1: f64, u2: f64) -> GeomAdaptorCurve
where
    C: rustycad_tkernel::standard::RuntimeTyped + rustycad_tkernel::standard::Transient,
{
    GeomAdaptorCurve::with_range(&Handle::new(curve).cast(), u1, u2).unwrap()
}

// Runs the new (MinMax, no endpoints) and old implementations on one point and
// compares them with `compare`.
fn compare_point(
    anAdaptor: &GeomAdaptorCurve,
    aPoint: Point3,
    compare: impl FnOnce(&ExtremaResult, &ExtPC),
) {
    let mut anExtPC = ExtremaPC::Curve::from_adaptor(anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, MM).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, anAdaptor);

    compare(aNewResult, &anOldExtPC);
}

include!("extrema_pc_extended_geometry/suites_0.rs");
include!("extrema_pc_extended_geometry/suites_1.rs");
include!("extrema_pc_extended_geometry/suites_2.rs");
