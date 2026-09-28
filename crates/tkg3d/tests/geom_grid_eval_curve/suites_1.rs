//=================================================================================================
// Additional tests for B-spline curves (from BSplCLib_GridEvaluator_Test)
//=================================================================================================

#[test]
fn GeomGridEval_BSplineCurveTest__RationalBSpline() {
    // Create a rational B-spline (NURBS circle approximation)
    let aCurve = CreateRationalBSpline();

    let anEval = GeomGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 21);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();
    assert_eq!(aGrid.len(), 21);

    // Verify against direct evaluation
    CheckPoints(&aGrid, 21, |i| aCurve.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_BSplineCurveTest__MultiSpanBSpline() {
    // Create a B-spline with multiple spans (degree 3, 6 control points, 1 internal knot)
    // Formula: n_poles = sum(multiplicities) - degree - 1 = (4 + 2 + 4) - 3 - 1 = 6
    let aCurve = CreateMultiSpanBSpline();

    let anEval = GeomGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 31);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation - should match exactly
    CheckPoints(&aGrid, 31, |i| aCurve.value(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_BSplineCurveTest__HighDegree() {
    // Create a higher degree B-spline (degree 5)
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 6).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 3.0, 0.0);
    aPoles[3] = Point3::new(2.0, 1.0, 0.0);
    aPoles[4] = Point3::new(3.0, 4.0, 0.0);
    aPoles[5] = Point3::new(4.0, 2.0, 0.0);
    aPoles[6] = Point3::new(5.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;
    aMults[1] = 6;
    aMults[2] = 6;

    let aCurve = Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 5, false).unwrap());

    let anEval = GeomGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 51);

    let aGrid: Array1<Point3> = anEval.evaluate_grid(&aParams).unwrap();

    // Verify against direct evaluation
    CheckPoints(&aGrid, 51, |i| aCurve.value(aParams[i]).unwrap());
}

//=================================================================================================
// Tests for Curve Derivative Evaluation (D1, D2, D3)
//=================================================================================================

#[test]
fn GeomGridEval_LineTest__DerivativeD1() {
    // Line along (1, 2, 3) normalized direction
    let aGeomLine = Line(0.0, 0.0, 0.0, 1.0, 2.0, 3.0);
    let anEval = GeomGridEvalLine::new(aGeomLine);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array1<GeomGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams);
    assert_eq!(aGrid.len(), 6);

    // For a line, D1 is constant (the direction vector)
    let aDir = Dir(1.0, 2.0, 3.0);
    for i in 1..=6 {
        // Verify D1 is the direction
        EXPECT_NEAR(aGrid[i].d1.x(), aDir.x(), THE_TOLERANCE);
        EXPECT_NEAR(aGrid[i].d1.y(), aDir.y(), THE_TOLERANCE);
        EXPECT_NEAR(aGrid[i].d1.z(), aDir.z(), THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_LineTest__DerivativeD2D3() {
    let aGeomLine = Line(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let anEval = GeomGridEvalLine::new(aGeomLine);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGridD2: Array1<GeomGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams);
    let aGridD3: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams);

    // For a line, D2 and D3 are zero
    for i in 1..=6 {
        EXPECT_NEAR(aGridD2[i].d2.magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGridD3[i].d2.magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR(aGridD3[i].d3.magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_CircleTest__DerivativeD1() {
    let aGeomCircle = CircleOfRadius2();
    let anEval = GeomGridEvalCircle::new(aGeomCircle.clone());

    let aParams = QuarterTurnParams();

    let aGrid: Array1<GeomGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    // Verify D1 against direct evaluation
    CheckD1(&aGrid, 5, |i| aGeomCircle.d1(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CircleTest__DerivativeD2() {
    let aGeomCircle = CircleOfRadius2();
    let anEval = GeomGridEvalCircle::new(aGeomCircle.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<GeomGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    // Verify D2 against direct evaluation
    CheckD2(&aGrid, 9, |i| aGeomCircle.d2(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeD1() {
    let aCurve = CreateSimpleBSpline();
    let anEval = GeomGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<GeomGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    // Verify against direct evaluation
    CheckD1(&aGrid, 11, |i| aCurve.d1(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeD2() {
    let aCurve = CreateSimpleBSpline();
    let anEval = GeomGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<GeomGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    // Verify against direct evaluation
    CheckD2(&aGrid, 11, |i| aCurve.d2(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeD3() {
    let aCurve = CreateSimpleBSpline();
    let anEval = GeomGridEvalBSplineCurve::new(aCurve.clone());

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    // Verify against direct evaluation
    CheckD3(&aGrid, 11, |i| aCurve.d3(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__UnifiedDerivativeD1() {
    let aGeomCircle = CircleOfRadius2();
    let anAdaptor = Adaptor(&aGeomCircle);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<GeomGridEvalCurveD1> = anEval.evaluate_grid_d1(&aParams).unwrap();

    // Verify against direct evaluation
    CheckD1(&aGrid, 9, |i| aGeomCircle.d1(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__UnifiedDerivativeD2() {
    let aCurve = CreateSimpleBSpline();
    let anAdaptor = Adaptor(&aCurve);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<GeomGridEvalCurveD2> = anEval.evaluate_grid_d2(&aParams).unwrap();

    // Verify against direct evaluation
    CheckD2(&aGrid, 11, |i| aCurve.d2(aParams[i]).unwrap());
}

//=================================================================================================
// Tests for Curve Third Derivative Evaluation (D3)
//=================================================================================================

#[test]
fn GeomGridEval_CircleTest__DerivativeD3() {
    let aGeomCircle = CircleOfRadius2();
    let anEval = GeomGridEvalCircle::new(aGeomCircle.clone());

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    // Verify D3 against direct evaluation
    CheckD3(&aGrid, 9, |i| aGeomCircle.d3(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_OffsetCurveTest__DerivativeD3() {
    // Offset curve from a circle
    let aCircle = CircleOfRadius2();
    let anOffset = OffsetDZ(&aCircle, 0.5);
    let anAdaptor = Adaptor(&anOffset);

    let anEval = GeomGridEvalOtherCurve::new(&anAdaptor);

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    // Verify D3 against adaptor evaluation
    CheckD3(&aGrid, 9, |i| anAdaptor.d3(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__OffsetCurveDerivativeD3() {
    // Offset curve via unified dispatcher
    let aLine = Line(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let anOffset = OffsetDZ(&aLine, 1.0);
    let anAdaptor = Adaptor(&anOffset);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(anEval.get_type(), CurveType::OffsetCurve);

    let aParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    // Verify D3 against adaptor evaluation
    CheckD3(&aGrid, 6, |i| anAdaptor.d3(aParams[i]).unwrap());
}

#[test]
fn GeomGridEval_CurveTest__UnifiedDerivativeD3() {
    let aCurve = CreateSimpleBSpline();
    let anAdaptor = Adaptor(&aCurve);

    let anEval = GeomGridEvalCurve::from_adaptor(&anAdaptor).unwrap();

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<GeomGridEvalCurveD3> = anEval.evaluate_grid_d3(&aParams).unwrap();

    // Verify against direct evaluation
    CheckD3(&aGrid, 11, |i| aCurve.d3(aParams[i]).unwrap());
}

//=================================================================================================
// Tests for BSpline Curve DN (Arbitrary Order Derivative)
//=================================================================================================

// The DN loop: the grid DN of order `theOrder` on `theNb` uniform parameters in [0, 1]
// against `Geom_BSplineCurve::DN`.
fn CheckGridDN(theCurve: &Handle<GeomBSplineCurve>, theNb: i32, theOrder: i32) {
    let anEval = GeomGridEvalBSplineCurve::new(theCurve.clone());
    let aParams = CreateUniformParams(0.0, 1.0, theNb);
    let aGrid: Array1<Vector3> = anEval.evaluate_grid_dn(&aParams, theOrder).unwrap();
    for i in 1..=theNb {
        let aExpected: Vector3 = theCurve.dn(aParams[i], theOrder).unwrap();
        EXPECT_NEAR((aGrid[i] - aExpected).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeDN_Order1() {
    CheckGridDN(&CreateSimpleBSpline(), 11, 1);
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeDN_Order2() {
    CheckGridDN(&CreateSimpleBSpline(), 11, 2);
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeDN_Order3() {
    CheckGridDN(&CreateSimpleBSpline(), 11, 3);
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeDN_BeyondDegree() {
    // Cubic B-spline (degree 3), 4th derivative should be zero
    let aCurve = CreateSimpleBSpline();
    let anEval = GeomGridEvalBSplineCurve::new(aCurve);

    let aParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array1<Vector3> = anEval.evaluate_grid_dn(&aParams, 4).unwrap();

    for i in 1..=11 {
        EXPECT_NEAR(aGrid[i].magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeDN_RationalCurve() {
    // Create a rational B-spline (NURBS)
    let aCurve = CreateRationalBSpline();

    // Test DN for orders 1, 2, 3
    for aOrder in 1..=3 {
        CheckGridDN(&aCurve, 21, aOrder);
    }
}

#[test]
fn GeomGridEval_BSplineCurveTest__DerivativeDN_MultiSpan() {
    // Create a multi-span B-spline (degree 3, with internal knot)
    let aCurve = CreateMultiSpanBSpline();

    // Test DN for orders 1, 2, 3
    for aOrder in 1..=3 {
        CheckGridDN(&aCurve, 31, aOrder);
    }
}
