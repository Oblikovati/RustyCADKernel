//! Direct translation of Geom2dGcc_Circ2d3Tan_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
// The expected lengths are copied digit for digit from the upstream suite.
#![allow(non_snake_case, non_camel_case_types, clippy::excessive_precision)]
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_gcc::{
    Circ2d3Tan as Geom2dGcc_Circ2d3Tan, QualifiedCurve, unqualified,
};
use rustycad_tkgeombase::gcpnts::AbscissaPoint as GCPnts_AbscissaPoint;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Point2};

fn circleLength(theCircle: Circle2) -> f64 {
    let aCurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_circ2d(theCircle)).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    GCPnts_AbscissaPoint::length(&anAdaptor).unwrap()
}

fn gp_Circ2d(theX: f64, theY: f64, theRadius: f64) -> Circle2 {
    let anAxis = Axis2d::new(Point2::new(theX, theY), Direction2::new(1.0, 0.0).unwrap());
    Circle2::from_x_axis(anAxis, theRadius, true).unwrap()
}

// Test fixture for Geom2dGcc_Circ2d3Tan testing
struct Geom2dGcc_Circ2d3TanTest {
    myTolerance: f64,
}

impl Geom2dGcc_Circ2d3TanTest {
    fn SetUp() -> Self {
        Self {
            myTolerance: precision::confusion(),
        }
    }

    /// Verify that a solution is tangent to all three input circles
    fn verifyTangencyConstraints(
        &self,
        theSolution: &Circle2,
        theCircles: [&Circle2; 3],
        theSolutionIndex: i32,
        theTolerance: f64,
    ) {
        let aSolCenter: Point2 = theSolution.location();
        let aSolRadius: f64 = theSolution.radius();

        // Check tangency to each input circle
        for (i, aCircle) in (1..).zip(theCircles) {
            let aCircCenter: Point2 = aCircle.location();
            let aCircRadius: f64 = aCircle.radius();

            let aDistanceCenters: f64 = aSolCenter.distance(aCircCenter);

            // For tangency: distance = |R1 - R2| (internal) or R1 + R2 (external)
            let aExpectedDistExt: f64 = aSolRadius + aCircRadius;
            let aExpectedDistInt: f64 = (aSolRadius - aCircRadius).abs();

            let isTangent: bool = (aDistanceCenters - aExpectedDistExt).abs() <= theTolerance
                || (aDistanceCenters - aExpectedDistInt).abs() <= theTolerance;

            assert!(
                isTangent,
                "Solution {theSolutionIndex} should be tangent to circle {i} (distance={aDistanceCenters}, \
                 expected external={aExpectedDistExt}, expected internal={aExpectedDistInt})"
            );
        }
    }

    /// Create a qualified curve from circle parameters
    fn createQualifiedCircle(&self, theX: f64, theY: f64, theRadius: f64) -> QualifiedCurve {
        let aCircle: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dCircle::from_circ2d(gp_Circ2d(theX, theY, theRadius))).cast();
        let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCircle).unwrap();
        unqualified(&anAdaptor).unwrap()
    }

    /// Verify solution validity; the bounds are (MinX, MaxX, MinY, MaxY, MaxRadius).
    fn verifySolutionValidity(
        &self,
        theSolution: &Circle2,
        theSolutionIndex: i32,
        theBounds: [f64; 5],
    ) {
        let [theMinX, theMaxX, theMinY, theMaxY, theMaxRadius] = theBounds;
        assert!(
            theSolution.radius() > 0.0,
            "Solution {theSolutionIndex} should have positive radius"
        );
        assert!(
            theSolution.radius() < theMaxRadius,
            "Solution {theSolutionIndex} should have reasonable radius"
        );

        let aCenter: Point2 = theSolution.location();
        assert!(
            aCenter.x() > theMinX,
            "Solution {theSolutionIndex} X coordinate should be reasonable"
        );
        assert!(
            aCenter.x() < theMaxX,
            "Solution {theSolutionIndex} X coordinate should be reasonable"
        );
        assert!(
            aCenter.y() > theMinY,
            "Solution {theSolutionIndex} Y coordinate should be reasonable"
        );
        assert!(
            aCenter.y() < theMaxY,
            "Solution {theSolutionIndex} Y coordinate should be reasonable"
        );
    }

    fn solve(&self, theQuals: [&QualifiedCurve; 3], theTolerance: f64) -> Geom2dGcc_Circ2d3Tan {
        Geom2dGcc_Circ2d3Tan::new(
            theQuals[0],
            theQuals[1],
            theQuals[2],
            theTolerance,
            [0.0, 0.0, 0.0],
        )
        .unwrap()
    }
}

// The default bounds of verifySolutionValidity.
const DEFAULT_BOUNDS: [f64; 5] = [-10000.0, 10000.0, -10000.0, 10000.0, 100000.0];

fn checkLengths(theSolver: &Geom2dGcc_Circ2d3Tan, theExpectedLengths: &[f64]) {
    for anIndex in 1..=theSolver.nb_solutions() {
        let anExpected: f64 = theExpectedLengths[anIndex as usize - 1];
        let aLength: f64 = circleLength(theSolver.this_solution(anIndex).unwrap());
        assert!((aLength - anExpected).abs() <= anExpected * 1.0e-8);
    }
}

// Test the exact BUC60622 regression case
#[test]
fn Geom2dGcc_Circ2d3TanTest__BUC60622_RegressionCase() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // BUC60622 test case: 3 circles that should produce 3 tangent solutions
    // circle c1 500 1800 500
    // circle c2 500 1900 400
    // circle c3 700 1900 200

    let aQual1 = t.createQualifiedCircle(500.0, 1800.0, 500.0);
    let aQual2 = t.createQualifiedCircle(500.0, 1900.0, 400.0);
    let aQual3 = t.createQualifiedCircle(700.0, 1900.0, 200.0);

    // Input circles for tangency verification
    let aInputCirc1 = gp_Circ2d(500.0, 1800.0, 500.0);
    let aInputCirc2 = gp_Circ2d(500.0, 1900.0, 400.0);
    let aInputCirc3 = gp_Circ2d(700.0, 1900.0, 200.0);

    // Solve for tangent circles with default tolerance
    let aTangentSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(aTangentSolver.is_done(), "Algorithm should succeed");

    let aNbSolutions: i32 = aTangentSolver.nb_solutions();

    // The regression test: we should find exactly 3 solutions for BUC60622 case
    assert_eq!(
        aNbSolutions, 3,
        "BUC60622 case should find exactly 3 solutions"
    );

    // Verify each solution is geometrically valid and satisfies tangency constraints
    for i in 1..=aNbSolutions {
        let aSol: Circle2 = aTangentSolver.this_solution(i).unwrap();

        // Basic validity checks
        t.verifySolutionValidity(&aSol, i, [-1000.0, 2000.0, 1000.0, 3000.0, 10000.0]);

        // Tangency constraint verification
        t.verifyTangencyConstraints(&aSol, [&aInputCirc1, &aInputCirc2, &aInputCirc3], i, 1e-6);
    }

    // Verify specific known solutions for BUC60622
    if aNbSolutions >= 3 {
        // Solution 1: center(500, 1900), radius=400 - this is actually one of the input circles
        let aSol1: Circle2 = aTangentSolver.this_solution(1).unwrap();
        assert!((aSol1.location().x() - 500.0).abs() <= 1.0);
        assert!((aSol1.location().y() - 1900.0).abs() <= 1.0);
        assert!((aSol1.radius() - 400.0).abs() <= 1.0);
    }
}

// Migrated from tests/lowalgos/2dgcc/buc60622_1.  The original DRAW case
// checked that this degenerate configuration can be passed to cirtang without
// raising an exception.
#[test]
fn Geom2dGcc_Circ2d3TanTest__BUC60622_1_DegenerateThreeCircleCase() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    let aQual1 = t.createQualifiedCircle(2900.0, 800.0, 500.0);
    let aQual2 = t.createQualifiedCircle(2900.0, 900.0, 400.0);
    let aQual3 = t.createQualifiedCircle(2900.0, 1000.0, 300.0);

    // EXPECT_NO_THROW: the constructor returns Ok.
    let aSolver =
        Geom2dGcc_Circ2d3Tan::new(&aQual1, &aQual2, &aQual3, t.myTolerance, [0.0, 0.0, 0.0]);
    assert!(aSolver.is_ok(), "{:?}", aSolver.err());
}

// Migrated from tests/lowalgos/2dgcc/buc60622_2.  Keep the three explicit
// solution-length checks in addition to the general regression validation.
#[test]
fn Geom2dGcc_Circ2d3TanTest__BUC60622_2_AllKnownSolutionLengths() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    let aQual1 = t.createQualifiedCircle(500.0, 1800.0, 500.0);
    let aQual2 = t.createQualifiedCircle(500.0, 1900.0, 400.0);
    let aQual3 = t.createQualifiedCircle(700.0, 1900.0, 200.0);
    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 3);

    let anExpectedLengths = [2513.2741228703289, 837.75804095727631, 279.2526803190928];
    checkLengths(&aSolver, &anExpectedLengths);
}

// Migrated from tests/lowalgos/2dgcc/buc60622_3.  The original DRAW test
// checks the lengths of all five returned tangent circles.
#[test]
fn Geom2dGcc_Circ2d3TanTest__BUC60622_3_AllTangentCircleLengths() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    let aQual1 = t.createQualifiedCircle(625.0, 3000.0, 125.0);
    let aQual2 = t.createQualifiedCircle(375.0, 3000.0, 125.0);
    let aQual3 = t.createQualifiedCircle(500.0, 3000.0, 250.0);
    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 5);

    let anExpectedLengths = [
        1570.7963267948965,
        785.39816339744868,
        785.39816339744823,
        523.59877559829806,
        523.59877559829806,
    ];
    checkLengths(&aSolver, &anExpectedLengths);
}

// Migrated from tests/lowalgos/2dgcc/buc60618.  This preserves the eight
// solution-length checks that guard the solver's solution filtering.
#[test]
fn Geom2dGcc_Circ2d3TanTest__BUC60618_FilteringKeepsAllSolutions() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    let aQual1 = t.createQualifiedCircle(0.0, 0.0, 100.0);
    let aQual2 = t.createQualifiedCircle(500.0, -400.0, 300.0);
    let aQual3 = t.createQualifiedCircle(300.0, 200.0, 200.0);
    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(aSolver.is_done());
    assert_eq!(aSolver.nb_solutions(), 8);

    let anExpectedLengths = [
        3568.9587306060739,
        9964.7111736227271,
        2082.9371204466097,
        32696.069757517998,
        1681.9298517645602,
        3097.3731840080009,
        1545.2428620020787,
        772.26646380656518,
    ];
    checkLengths(&aSolver, &anExpectedLengths);
}

// Test with different tolerance values to demonstrate the filtering issue
#[test]
fn Geom2dGcc_Circ2d3TanTest__ToleranceImpact_Analysis() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // Same BUC60622 configuration
    let aQual1 = t.createQualifiedCircle(500.0, 1800.0, 500.0);
    let aQual2 = t.createQualifiedCircle(500.0, 1900.0, 400.0);
    let aQual3 = t.createQualifiedCircle(700.0, 1900.0, 200.0);

    // Test with different tolerance values to ensure stability
    let aTestTolerances = [
        precision::confusion(), // Default ~1e-15
        1e-12,                  // Slightly larger
        1e-10,                  // Moderate
        1e-8,                   // Large
    ];

    let mut aDefaultSolCount: i32 = 0;

    for aTol in aTestTolerances {
        let aSolver = t.solve([&aQual1, &aQual2, &aQual3], aTol);

        assert!(
            aSolver.is_done(),
            "Algorithm should succeed with tolerance {aTol}"
        );

        let aNbSol: i32 = aSolver.nb_solutions();

        if aTol == precision::confusion() {
            aDefaultSolCount = aNbSol;
            assert!(
                aNbSol >= 1,
                "Should find at least 1 solution with default tolerance"
            );
        } else {
            // Solution count may vary with tolerance due to numerical precision effects
            // We allow some variation but expect reasonable bounds
            assert!(
                aNbSol >= 1,
                "Should find at least 1 solution with tolerance {aTol}"
            );
            assert!(
                aNbSol <= aDefaultSolCount + 2,
                "Solution count shouldn't increase dramatically with tolerance {aTol}"
            );
        }

        // Verify all solutions are valid regardless of tolerance
        for i in 1..=aNbSol {
            let aSol: Circle2 = aSolver.this_solution(i).unwrap();
            t.verifySolutionValidity(&aSol, i, [-1000.0, 2000.0, 1000.0, 3000.0, 10000.0]);
        }
    }
}

// Test a simpler case that should definitely work
#[test]
fn Geom2dGcc_Circ2d3TanTest__Simple_ThreeCircle_Case() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // Simple case: three well-separated circles
    let aQual1 = t.createQualifiedCircle(0.0, 0.0, 2.0);
    let aQual2 = t.createQualifiedCircle(10.0, 0.0, 2.0);
    let aQual3 = t.createQualifiedCircle(5.0, 8.0, 2.0);

    // Input circles for verification
    let aInputCirc1 = gp_Circ2d(0.0, 0.0, 2.0);
    let aInputCirc2 = gp_Circ2d(10.0, 0.0, 2.0);
    let aInputCirc3 = gp_Circ2d(5.0, 8.0, 2.0);

    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(aSolver.is_done(), "Simple case should always work");

    let aNbSol: i32 = aSolver.nb_solutions();
    assert!(
        aNbSol >= 1,
        "Should find at least one solution for simple case"
    );

    // Verify all solutions are valid and satisfy tangency constraints
    for i in 1..=aNbSol {
        let aSol: Circle2 = aSolver.this_solution(i).unwrap();
        t.verifySolutionValidity(&aSol, i, DEFAULT_BOUNDS);
        t.verifyTangencyConstraints(&aSol, [&aInputCirc1, &aInputCirc2, &aInputCirc3], i, 1e-6);
    }
}

// Test with concentric circles to verify edge cases
#[test]
fn Geom2dGcc_Circ2d3TanTest__Concentric_Circles_EdgeCase() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // Concentric circles: geometrically, this specific configuration with
    // concentric circles at (0,0) with radii 1,3 and separate circle at (10,0) radius 2
    // has no solutions - this is mathematically correct
    let aQual1 = t.createQualifiedCircle(0.0, 0.0, 1.0);
    let aQual2 = t.createQualifiedCircle(0.0, 0.0, 3.0);
    let aQual3 = t.createQualifiedCircle(10.0, 0.0, 2.0);

    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    // For this specific concentric configuration, the algorithm succeeds but finds no solutions
    assert!(
        aSolver.is_done(),
        "Algorithm should complete successfully even when no solutions exist"
    );

    let aNbSol: i32 = aSolver.nb_solutions();
    // This specific geometric configuration has no solutions - this is correct
    assert_eq!(
        aNbSol, 0,
        "This concentric configuration geometrically has no solutions"
    );
}

// Test with very small circles
#[test]
fn Geom2dGcc_Circ2d3TanTest__SmallCircles_PrecisionTest() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // Very small circles to test precision handling
    let aQual1 = t.createQualifiedCircle(0.0, 0.0, 0.01);
    let aQual2 = t.createQualifiedCircle(0.1, 0.0, 0.01);
    let aQual3 = t.createQualifiedCircle(0.05, 0.08, 0.01);

    // Input circles for verification
    let aInputCirc1 = gp_Circ2d(0.0, 0.0, 0.01);
    let aInputCirc2 = gp_Circ2d(0.1, 0.0, 0.01);
    let aInputCirc3 = gp_Circ2d(0.05, 0.08, 0.01);

    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(
        aSolver.is_done(),
        "Small circles should be handled correctly"
    );

    let aNbSol: i32 = aSolver.nb_solutions();
    assert!(
        aNbSol >= 1,
        "Should find at least one solution for small circles"
    );

    for i in 1..=aNbSol {
        let aSol: Circle2 = aSolver.this_solution(i).unwrap();
        t.verifySolutionValidity(&aSol, i, [-10.0, 10.0, -10.0, 10.0, 10.0]);
        // Relaxed tolerance for small circles
        t.verifyTangencyConstraints(&aSol, [&aInputCirc1, &aInputCirc2, &aInputCirc3], i, 1e-3);
    }
}

// Test with large circles
#[test]
fn Geom2dGcc_Circ2d3TanTest__LargeCircles_ScalingTest() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // Large circles to test numerical scaling
    let aQual1 = t.createQualifiedCircle(0.0, 0.0, 1000.0);
    let aQual2 = t.createQualifiedCircle(5000.0, 0.0, 1500.0);
    let aQual3 = t.createQualifiedCircle(2500.0, 4000.0, 800.0);

    // Input circles for verification
    let aInputCirc1 = gp_Circ2d(0.0, 0.0, 1000.0);
    let aInputCirc2 = gp_Circ2d(5000.0, 0.0, 1500.0);
    let aInputCirc3 = gp_Circ2d(2500.0, 4000.0, 800.0);

    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(
        aSolver.is_done(),
        "Large circles should be handled correctly"
    );

    let aNbSol: i32 = aSolver.nb_solutions();
    assert!(
        aNbSol >= 1,
        "Should find at least one solution for large circles"
    );

    for i in 1..=aNbSol {
        let aSol: Circle2 = aSolver.this_solution(i).unwrap();
        t.verifySolutionValidity(&aSol, i, [-10000.0, 10000.0, -10000.0, 10000.0, 50000.0]);
        // Relaxed tolerance for large circles
        t.verifyTangencyConstraints(&aSol, [&aInputCirc1, &aInputCirc2, &aInputCirc3], i, 1.0);
    }
}

// Test with circles in a line configuration
#[test]
fn Geom2dGcc_Circ2d3TanTest__LinearConfiguration_GeometricTest() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // Three circles aligned on X-axis
    let aQual1 = t.createQualifiedCircle(0.0, 0.0, 1.0);
    let aQual2 = t.createQualifiedCircle(5.0, 0.0, 1.5);
    let aQual3 = t.createQualifiedCircle(10.0, 0.0, 1.2);

    // Input circles for verification
    let aInputCirc1 = gp_Circ2d(0.0, 0.0, 1.0);
    let aInputCirc2 = gp_Circ2d(5.0, 0.0, 1.5);
    let aInputCirc3 = gp_Circ2d(10.0, 0.0, 1.2);

    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    assert!(aSolver.is_done(), "Linear configuration should be solvable");

    let aNbSol: i32 = aSolver.nb_solutions();
    assert!(
        aNbSol >= 1,
        "Should find at least one solution for linear configuration"
    );

    for i in 1..=aNbSol {
        let aSol: Circle2 = aSolver.this_solution(i).unwrap();
        t.verifySolutionValidity(&aSol, i, DEFAULT_BOUNDS);
        t.verifyTangencyConstraints(&aSol, [&aInputCirc1, &aInputCirc2, &aInputCirc3], i, 1e-6);
    }
}

// Test with touching circles
#[test]
fn Geom2dGcc_Circ2d3TanTest__TouchingCircles_DegenerateCase() {
    let t = Geom2dGcc_Circ2d3TanTest::SetUp();
    // Two circles that touch each other externally plus a third separate circle
    // This configuration should have solutions geometrically
    let aQual1 = t.createQualifiedCircle(0.0, 0.0, 2.0);
    let aQual2 = t.createQualifiedCircle(4.0, 0.0, 2.0); // Touch at (2,0)
    let aQual3 = t.createQualifiedCircle(2.0, 5.0, 1.5);

    // Input circles for verification
    let aInputCirc1 = gp_Circ2d(0.0, 0.0, 2.0);
    let aInputCirc2 = gp_Circ2d(4.0, 0.0, 2.0);
    let aInputCirc3 = gp_Circ2d(2.0, 5.0, 1.5);

    let aSolver = t.solve([&aQual1, &aQual2, &aQual3], t.myTolerance);

    // For this specific touching configuration, the algorithm should succeed
    assert!(
        aSolver.is_done(),
        "Touching circles configuration should be solvable"
    );

    let aNbSol: i32 = aSolver.nb_solutions();
    assert!(
        aNbSol >= 1,
        "Should find at least one solution for touching circles case"
    );

    // Verify all solutions are valid and satisfy tangency
    for i in 1..=aNbSol {
        let aSol: Circle2 = aSolver.this_solution(i).unwrap();
        t.verifySolutionValidity(&aSol, i, DEFAULT_BOUNDS);
        t.verifyTangencyConstraints(&aSol, [&aInputCirc1, &aInputCirc2, &aInputCirc3], i, 1e-6);
    }
}
