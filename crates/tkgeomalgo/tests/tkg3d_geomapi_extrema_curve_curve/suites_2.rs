// Migrated from tests/lowalgos/extcc/bug29712_3.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_3_BoundedParallelSegments() {
    let aCurve1 = trimmed(&line(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), 0.0, 1.0);
    let aCurve2 = trimmed(&line(Point3::new(3.0, 4.0, 0.0), dir(1.0, 0.0, 0.0)), 0.0, 1.0);
    checkOneFiniteExtremum(&aCurve1, &aCurve2, 20.0_f64.sqrt());
}

// Migrated from tests/lowalgos/extcc/bug29712_4.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_4_ParallelTrimmedLines() {
    let aCurve1 = trimmed(&line(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), 0.0, 1.0);
    let aCurve2 = trimmed(&line(Point3::new(0.0, 4.0, 0.0), dir(1.0, 0.0, 0.0)), 0.0, 1.0);
    checkParallelExtrema(&aCurve1, &aCurve2, 4.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_5.  The source checks that both
// orientations still produce a finite result for separated trimmed lines.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_5_SeparatedTrimmedLines() {
    let aCurve1 = trimmed(
        &line(Point3::new(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)),
        -1.0e100,
        5.0,
    );
    let aCurve2 = trimmed(&line(Point3::new(5.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)), 10.0, 20.0);

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCurve1, &aCurve2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 1);
    let anExtrema21 = ExtremaCurveCurve::with_curves(&aCurve2, &aCurve1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 1);
}

// Migrated from tests/lowalgos/extcc/bug29712_6.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_6_ParallelLines() {
    let aCurve1 = line(Point3::new(1.0, 2.0, 3.0), dir(0.0, 1.0, 0.0));
    let aCurve2 = line(Point3::new(10.0, 9.0, 8.0), dir(0.0, 1.0, 0.0));
    checkParallelExtrema(&aCurve1, &aCurve2, 106.0_f64.sqrt());
}

// Migrated from tests/lowalgos/extcc/bug29712_7.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_7_ParallelTrimmedLine() {
    let aCurve1 = line(Point3::new(1.0, 2.0, 3.0), dir(0.0, 1.0, 0.0));
    let aCurve2 = trimmed(&line(Point3::new(10.0, 9.0, 8.0), dir(0.0, 1.0, 0.0)), -1.0, 2.0);
    checkParallelExtrema(&aCurve1, &aCurve2, 106.0_f64.sqrt());
}

// Migrated from tests/lowalgos/extcc/bug29712_8.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_8_ParallelTrimmedFirstLine() {
    let aCurve1 = trimmed(
        &line(Point3::new(1.0, 2.0, 3.0), dir(0.0, 1.0, 0.0)),
        -100.0,
        200.0,
    );
    let aCurve2 = line(Point3::new(10.0, 9.0, 8.0), dir(0.0, 1.0, 0.0));
    checkParallelExtrema(&aCurve1, &aCurve2, 106.0_f64.sqrt());
}

// Migrated from tests/lowalgos/extcc/bug29712_9.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_9_OppositeDirectionLines() {
    let aCurve1 = trimmed(&line(Point3::new(1.0, 0.0, 0.0), dir(0.0, 0.0, -1.0)), -5.0, 2.0);
    let aCurve2 = trimmed(
        &line(Point3::new(5.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)),
        -1.0e100,
        9.9e-8,
    );
    checkParallelExtrema(&aCurve1, &aCurve2, 4.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_11.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_11_SeparatedOppositeLines() {
    let aCurve1 = trimmed(&line(Point3::new(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)), 0.0, 3.0);
    let aCurve2 = trimmed(
        &line(Point3::new(5.0, 0.0, 0.0), dir(0.0, 0.0, -1.0)),
        -9.9e-8,
        1.0e100,
    );
    checkOneFiniteExtremum(&aCurve1, &aCurve2, 4.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_12.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_12_OppositeTrimmedSegments() {
    let aCurve1 = trimmed(&line(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)), 0.0, 1.0);
    let aCurve2 = trimmed(&line(Point3::new(0.0, 4.0, 0.0), dir(-1.0, 0.0, 0.0)), -2.0, -1.0);
    checkOneFiniteExtremum(&aCurve1, &aCurve2, 4.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_13.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_13_SeparatedConcentricArcs() {
    checkFiniteConcentricExtremumExists(4.0, 6.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_14.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_14_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, -8.1, -6.1);
}

// Migrated from tests/lowalgos/extcc/bug29712_15.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_15_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, -1.9, 0.1);
}

// Migrated from tests/lowalgos/extcc/bug29712_16.  The common radial arc
// produces a finite minimum and must not be reported as infinitely parallel.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_16_FiniteConcentricMinimum() {
    checkFiniteConcentricExtremum(-2.0, 0.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_17.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_17_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, -3.6, -1.6);
}

// Migrated from tests/lowalgos/extcc/bug29712_18.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_18_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, -3.5, -1.5);
}

// Migrated from tests/lowalgos/extcc/bug29712_19.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_19_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, -3.4, -1.4);
}

// Migrated from tests/lowalgos/extcc/bug29712_21.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_21_SeparatedConcentricArcs() {
    checkFiniteConcentricExtremumExists(-3.2, -1.2);
}

// Migrated from tests/lowalgos/extcc/bug29712_22.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_22_SeparatedConcentricArcs() {
    checkFiniteConcentricExtremumExists(-3.1, -1.1);
}

// Migrated from tests/lowalgos/extcc/bug29712_23.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_23_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, -9.6, -7.6);
}

// Migrated from tests/lowalgos/extcc/bug29712_24.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_24_SeparatedConcentricArcs() {
    checkFiniteConcentricExtremumExists(-9.5, -7.5);
}

// Migrated from tests/lowalgos/extcc/bug29712_25.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_25_SeparatedConcentricArcs() {
    checkFiniteConcentricExtremumExists(-8.3, -6.3);
}

// Migrated from tests/lowalgos/extcc/bug29712_26.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_26_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, -8.2, -6.2);
}

// Migrated from tests/lowalgos/extcc/bug29712_27.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_27_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(3.0, 0.0, 2.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_28.  The crossing arc pair
// must return one finite extremum rather than an infinite solution set.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_28_FiniteConcentricMinimum() {
    checkFiniteConcentricExtremum(3.0, 5.0);
}

// Migrated from tests/lowalgos/extcc/bug29712_29.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_29_SeparatedConcentricArcs() {
    checkFiniteConcentricExtremumExists(3.1, 5.1);
}

// Migrated from tests/lowalgos/extcc/bug29712_31.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_31_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(PI, -9.5, -7.5);
}

// Migrated from tests/lowalgos/extcc/bug29712_32.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_32_SeparatedConcentricArcs() {
    checkFiniteConcentricExtremumExists(-9.4, -7.4);
}

// Migrated from tests/lowalgos/extcc/bug29712_33.
#[test]
fn GeomAPI_ExtremaCurveCurve_Test__OCC29712_33_ParallelConcentricArcs() {
    checkOCC29712ConcentricCircles(PI, 0.0, 2.0);
}
