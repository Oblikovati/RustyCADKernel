//=================================================================================================
// Tests for Surface Derivative Evaluation (D1, D2)
//=================================================================================================

#[test]
fn GeomGridEval_PlaneTest__DerivativeD1() {
    // XY plane at origin
    let aPlane = PlaneZ(0.0, 0.0, 0.0);
    let anEval = GeomGridEvalPlane::new(aPlane.clone());

    let aUParams = CreateUniformParams(0.0, 5.0, 6);
    let aVParams = CreateUniformParams(0.0, 3.0, 4);

    let aGrid: Array2<GeomGridEvalSurfD1> = anEval.evaluate_grid_d1(&aUParams, &aVParams);

    // For a plane, D1U and D1V are constant (the X and Y directions of the plane)
    let aRef = aPlane.d1(0.0, 0.0).unwrap();
    for iU in 1..=6 {
        for iV in 1..=4 {
            EXPECT_NEAR(
                (aGrid[(iU, iV)].d1u - aRef.d1u).magnitude(),
                0.0,
                THE_TOLERANCE,
            );
            EXPECT_NEAR(
                (aGrid[(iU, iV)].d1v - aRef.d1v).magnitude(),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_PlaneTest__DerivativeD2() {
    let aPlane = PlaneZ(0.0, 0.0, 0.0);
    let anEval = GeomGridEvalPlane::new(aPlane);

    let aUParams = CreateUniformParams(0.0, 5.0, 6);
    let aVParams = CreateUniformParams(0.0, 3.0, 4);

    let aGrid: Array2<GeomGridEvalSurfD2> = anEval.evaluate_grid_d2(&aUParams, &aVParams);

    // For a plane, all second derivatives are zero
    for iU in 1..=6 {
        for iV in 1..=4 {
            EXPECT_NEAR(aGrid[(iU, iV)].d2u.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aGrid[(iU, iV)].d2v.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(aGrid[(iU, iV)].d2uv.magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SphereTest__DerivativeD1() {
    let aSphere = SphereZ(0.0, 0.0, 0.0, 2.0);
    let anEval = GeomGridEvalSphere::new(aSphere.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(-PI / 2.0, PI / 2.0, 5);

    let aGrid: Array2<GeomGridEvalSurfD1> = anEval.evaluate_grid_d1(&aUParams, &aVParams).unwrap();

    // Verify D1 against direct evaluation
    CheckD1(&aGrid, (9, 5), |iU, iV| {
        aSphere.d1(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SphereTest__DerivativeD2() {
    let aSphere = SphereZ(0.0, 0.0, 0.0, 2.0);
    let anEval = GeomGridEvalSphere::new(aSphere.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(-PI / 2.0, PI / 2.0, 5);

    let aGrid: Array2<GeomGridEvalSurfD2> = anEval.evaluate_grid_d2(&aUParams, &aVParams).unwrap();

    // Verify D2 against direct evaluation
    CheckD2(&aGrid, (9, 5), |iU, iV| {
        aSphere.d2(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__UnifiedDerivativeD1() {
    let aSphere = SphereZ(0.0, 0.0, 0.0, 2.0);
    let anAdaptor = Adaptor(&aSphere);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(-PI / 2.0, PI / 2.0, 5);

    let aGrid: Array2<GeomGridEvalSurfD1> = anEval.evaluate_grid_d1(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckD1(&aGrid, (9, 5), |iU, iV| {
        aSphere.d1(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__UnifiedDerivativeD2() {
    let aSurf = CreateSimpleBSplineSurface();
    let anAdaptor = Adaptor(&aSurf);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();

    let aUParams = CreateUniformParams(0.0, 1.0, 5);
    let aVParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array2<GeomGridEvalSurfD2> = anEval.evaluate_grid_d2(&aUParams, &aVParams).unwrap();

    // Verify against direct evaluation
    CheckD2(&aGrid, (5, 5), |iU, iV| {
        aSurf.d2(aUParams[iU], aVParams[iV]).unwrap()
    });
}

//=================================================================================================
// Tests for Surface Third Derivative Evaluation (D3)
//=================================================================================================

#[test]
fn GeomGridEval_PlaneTest__DerivativeD3() {
    let aPlane = PlaneZ(0.0, 0.0, 0.0);
    let anEval = GeomGridEvalPlane::new(aPlane);

    let aUParams = CreateUniformParams(0.0, 5.0, 6);
    let aVParams = CreateUniformParams(0.0, 3.0, 4);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams);

    // For a plane, all derivatives >= 2 are zero
    for iU in 1..=6 {
        for iV in 1..=4 {
            let g = &aGrid[(iU, iV)];
            EXPECT_NEAR(g.d2u.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(g.d2v.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(g.d2uv.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(g.d3u.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(g.d3v.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(g.d3uuv.magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR(g.d3uvv.magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_SphereTest__DerivativeD3() {
    let aSphere = SphereZ(0.0, 0.0, 0.0, 2.0);
    let anEval = GeomGridEvalSphere::new(aSphere.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(-PI / 2.0 + 0.1, PI / 2.0 - 0.1, 5); // Avoid poles

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    // Verify D3 against direct evaluation using GeomAdaptor
    let anAdaptor = Adaptor(&aSphere);
    CheckD3(&aGrid, (9, 5), true, |iU, iV| {
        anAdaptor.d3(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_CylinderTest__DerivativeD3() {
    let aCyl = CylinderOfRadius2();
    let anEval = GeomGridEvalCylinder::new(aCyl.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    let anAdaptor = Adaptor(&aCyl);
    CheckD3(&aGrid, (9, 6), false, |iU, iV| {
        anAdaptor.d3(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_ConeTest__DerivativeD3() {
    let aCone = ConeQuarterPi();
    let anEval = GeomGridEvalCone::new(aCone.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 5.0, 6);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    let anAdaptor = Adaptor(&aCone);
    CheckD3(&aGrid, (9, 6), false, |iU, iV| {
        anAdaptor.d3(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_TorusTest__DerivativeD3() {
    let aTorus = Torus4x1();
    let anEval = GeomGridEvalTorus::new(aTorus.clone());

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    let anAdaptor = Adaptor(&aTorus);
    CheckD3(&aGrid, (9, 9), false, |iU, iV| {
        anAdaptor.d3(aUParams[iU], aVParams[iV]).unwrap()
    });
}

#[test]
fn GeomGridEval_SurfaceTest__UnifiedDerivativeD3() {
    let aTorus = Torus4x1();
    let anAdaptor = Adaptor(&aTorus);

    let anEval = GeomGridEvalSurface::from_adaptor(&anAdaptor).unwrap();

    let aUParams = CreateUniformParams(0.0, 2.0 * PI, 9);
    let aVParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    CheckD3(&aGrid, (9, 9), false, |iU, iV| {
        aTorus.d3(aUParams[iU], aVParams[iV]).unwrap()
    });
}
