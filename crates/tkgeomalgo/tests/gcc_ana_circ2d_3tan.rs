// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Direct translation of GccAna_Circ2d3Tan_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
// The expected lengths are copied digit for digit from the upstream suite.
#![allow(non_snake_case, clippy::excessive_precision)]
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCircle;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::gcc_ana::Circ2d3Tan as GccAna_Circ2d3Tan;
use rustycad_tkgeomalgo::gcc_ent::{Position, QualifiedCirc, QualifiedLin};
use rustycad_tkgeombase::gcpnts::AbscissaPoint as GCPnts_AbscissaPoint;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Line2, Point2};

fn makeCircle(theX: f64, theY: f64, theRadius: f64) -> Circle2 {
    let anAxis = Axis2d::new(Point2::new(theX, theY), Direction2::new(1.0, 0.0).unwrap());
    Circle2::from_x_axis(anAxis, theRadius, true).unwrap()
}

fn makeQualifiedCircle(theX: f64, theY: f64, theRadius: f64) -> QualifiedCirc {
    QualifiedCirc::new(makeCircle(theX, theY, theRadius), Position::Unqualified)
}

fn makeQualifiedLine(theX: f64, theY: f64, theDX: f64, theDY: f64) -> QualifiedLin {
    let aLine = Line2::new(
        Point2::new(theX, theY),
        Direction2::new(theDX, theDY).unwrap(),
    );
    QualifiedLin::new(aLine, Position::Unqualified)
}

fn circleLength(theCircle: Circle2) -> f64 {
    let aCurve: Handle<Geom2dCircle> = Handle::new(Geom2dCircle::from_circ2d(theCircle));
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve.cast()).unwrap();
    GCPnts_AbscissaPoint::length(&anAdaptor).unwrap()
}

// makeExpectedLengths copies the values into a 1-based array; a slice reads the same.
fn verifySolutionLengths(theSolver: &GccAna_Circ2d3Tan, theExpected: &[f64]) {
    assert!(theSolver.is_done());
    assert!(theSolver.nb_solutions() >= theExpected.len() as i32);
    for (anIndex, &anExpectedLength) in (1..).zip(theExpected) {
        let aTolerance: f64 = 1.0e-7
            * (if anExpectedLength > 1.0 {
                anExpectedLength
            } else {
                1.0
            });
        let aSolution: Circle2 = theSolver.this_solution(anIndex).unwrap();
        assert!(aSolution.radius() > 0.0);
        let aLength: f64 = circleLength(aSolution);
        assert!(
            (aLength - anExpectedLength).abs() <= aTolerance,
            "Unexpected circumference for solution {anIndex}: {aLength} vs {anExpectedLength}"
        );
    }
}

fn verifySolverDoesNotFail(theSolver: &GccAna_Circ2d3Tan) {
    assert!(theSolver.is_done());
    for anIndex in 1..=theSolver.nb_solutions() {
        assert!(
            theSolver.this_solution(anIndex).unwrap().radius() > 0.0,
            "Invalid zero-radius solution {anIndex}"
        );
    }
}

// Migrated from tests/geometry/circ2d3Tan/CircleCircleCircle_11.
#[test]
fn GccAna_Circ2d3TanTest__CircleCircleCircle_11() {
    let aCircle1 = makeQualifiedCircle(0.0, 0.0, 50.0);
    let aCircle2 = makeQualifiedCircle(20.0, 0.0, 10.0);
    let aCircle3 = makeQualifiedCircle(0.0, 20.0, 10.0);
    let aSolver =
        GccAna_Circ2d3Tan::circ_circ_circ(aCircle1, aCircle2, aCircle3, precision::confusion())
            .unwrap();
    let anExpectedValues = [
        168.36566348025758,
        244.52937099154383,
        131.42863607625242,
        182.73062928272694,
        182.7306292827268,
        131.42863607625236,
        94.936311385359318,
        178.56704904481091,
    ];
    verifySolutionLengths(&aSolver, &anExpectedValues);
}

// Migrated from tests/geometry/circ2d3Tan/CircleCirclePoint_11.
#[test]
fn GccAna_Circ2d3TanTest__CircleCirclePoint_11() {
    let aCircle1 = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aCircle2 = makeQualifiedCircle(30.0, 0.0, 20.0);
    let aSolver = GccAna_Circ2d3Tan::circ_circ_pnt(
        aCircle1,
        aCircle2,
        Point2::new(10.0, 10.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[13.802267767659149, 80.445511840034683]);
}

// Migrated from tests/geometry/circ2d3Tan/CircleCirclePoint_12.
#[test]
fn GccAna_Circ2d3TanTest__CircleCirclePoint_12() {
    let aCircle1 = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aCircle2 = makeQualifiedCircle(30.0, 0.0, 20.0);
    let aSolver = GccAna_Circ2d3Tan::circ_circ_pnt(
        aCircle1,
        aCircle2,
        Point2::new(7.0, 30.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(
        &aSolver,
        &[951.90257403770727, 66.177052461460107, 190.20396243259333],
    );
}

// Migrated from tests/geometry/circ2d3Tan/CircleLinLin_11.
#[test]
fn GccAna_Circ2d3TanTest__CircleLinLin_11() {
    let aCircle = makeQualifiedCircle(0.0, 120.0, 20.0);
    let aLine1 = makeQualifiedLine(0.0, 0.0, 10.0, 20.0);
    let aLine2 = makeQualifiedLine(0.0, 0.0, 10.0, -40.0);
    let aSolver =
        GccAna_Circ2d3Tan::circ_lin_lin(aCircle, aLine1, aLine2, precision::confusion()).unwrap();
    let anExpectedValues = [
        461.86006847878718,
        163.75801021417183,
        321.80336707682847,
        235.02950419226329,
    ];
    verifySolutionLengths(&aSolver, &anExpectedValues);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_11.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_11() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(20.0, 0.0),
        Point2::new(15.0, 5.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[157.07963267948966, 31.415926535897931]);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_21.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_21() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(5.0, 0.0),
        Point2::new(0.0, 5.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[45.424379479936995, 26.383452602115419]);
}

// Migrated from tests/geometry/circ2d3Tan/LinLinLin_11.
#[test]
fn GccAna_Circ2d3TanTest__LinLinLin_11() {
    let aLine1 = makeQualifiedLine(0.0, 0.0, 10.0, 20.0);
    let aLine2 = makeQualifiedLine(0.0, 0.0, 10.0, -40.0);
    let aLine3 = makeQualifiedLine(160.0, 0.0, -40.0, 10.0);
    let aSolver =
        GccAna_Circ2d3Tan::lin_lin_lin(aLine1, aLine2, aLine3, precision::confusion()).unwrap();
    let anExpectedValues = [
        213.09795279419643,
        284.90187851033369,
        131.38343888467227,
        63.235238531994284,
    ];
    verifySolutionLengths(&aSolver, &anExpectedValues);
}

// Migrated from tests/geometry/circ2d3Tan/LinLinPoint_11.
#[test]
fn GccAna_Circ2d3TanTest__LinLinPoint_11() {
    let aLine1 = makeQualifiedLine(0.0, 0.0, 10.0, 20.0);
    let aLine2 = makeQualifiedLine(0.0, 0.0, 10.0, -40.0);
    let aSolver = GccAna_Circ2d3Tan::lin_lin_pnt(
        aLine1,
        aLine2,
        Point2::new(10.0, 80.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[269.03484941268533, 130.52381207643296]);
}

// Migrated from tests/geometry/circ2d3Tan/LinPointPoint_11.
#[test]
fn GccAna_Circ2d3TanTest__LinPointPoint_11() {
    let aLine = makeQualifiedLine(0.0, 0.0, 10.0, 20.0);
    let aSolver = GccAna_Circ2d3Tan::lin_pnt_pnt(
        aLine,
        Point2::new(10.0, 10.0),
        Point2::new(100.0, 10.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[419.71016104587477, 282.77131205819785]);
}

// Migrated from tests/geometry/circ2d3Tan/CircleCirclePoint_14.
#[test]
fn GccAna_Circ2d3TanTest__CircleCirclePoint_14() {
    let aCircle1 = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aCircle2 = makeQualifiedCircle(30.0, 0.0, 20.0);
    let aSolver = GccAna_Circ2d3Tan::circ_circ_pnt(
        aCircle1,
        aCircle2,
        Point2::new(30.0, -20.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(
        &aSolver,
        &[125.6637061435916, 376.99111854338514, 376.99111854338514],
    );
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_12.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_12() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(20.0, 0.0),
        Point2::new(10.0, 0.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolverDoesNotFail(&aSolver);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_13.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_13() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(20.0, 0.0),
        Point2::new(0.0, 10.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[157.07963267948978, 157.07963267948978]);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_14.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_14() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(20.0, 0.0),
        Point2::new(5.0, 5.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolverDoesNotFail(&aSolver);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_15.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_15() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(20.0, 0.0),
        Point2::new(-20.0, 0.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[157.07963267948966, 157.07963267948966]);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_22.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_22() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(5.0, 0.0),
        Point2::new(0.0, 0.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[31.415926535897931, 31.415926535897931]);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_24.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_24() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(5.0, 0.0),
        Point2::new(0.0, 10.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[39.269908169872416]);
}

// Migrated from tests/geometry/circ2d3Tan/CirclePointPoint_31.
#[test]
fn GccAna_Circ2d3TanTest__CirclePointPoint_31() {
    let aCircle = makeQualifiedCircle(0.0, 0.0, 10.0);
    let aSolver = GccAna_Circ2d3Tan::circ_pnt_pnt(
        aCircle,
        Point2::new(10.0, 0.0),
        Point2::new(0.0, 10.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[62.831853071795862]);
}

// Migrated from tests/geometry/circ2d3Tan/PointPointPoint_11.
#[test]
fn GccAna_Circ2d3TanTest__PointPointPoint_11() {
    let aSolver = GccAna_Circ2d3Tan::pnt_pnt_pnt(
        Point2::new(0.0, 50.0),
        Point2::new(30.0, 20.0),
        Point2::new(150.0, 150.0),
        precision::confusion(),
    )
    .unwrap();
    verifySolutionLengths(&aSolver, &[566.81157580298293]);
}
