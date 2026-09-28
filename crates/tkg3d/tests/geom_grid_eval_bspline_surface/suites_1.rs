//=================================================================================================
// Derivative Tests
//=================================================================================================

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeD1() {
    let aSurf = CreateSimpleBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());

    let aUParams = CreateUniformParams(0.0, 1.0, 5);
    let aVParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array2<GeomGridEvalSurfD1> = anEval.evaluate_grid_d1(&aUParams, &aVParams).unwrap();

    for iU in 1..=5 {
        for iV in 1..=5 {
            let r = aSurf.d1(aUParams[iU], aVParams[iV]).unwrap();
            let g = &aGrid[(iU, iV)];
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeD2() {
    let aSurf = CreateSimpleBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());

    let aUParams = CreateUniformParams(0.0, 1.0, 5);
    let aVParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array2<GeomGridEvalSurfD2> = anEval.evaluate_grid_d2(&aUParams, &aVParams).unwrap();

    for iU in 1..=5 {
        for iV in 1..=5 {
            let r = aSurf.d2(aUParams[iU], aVParams[iV]).unwrap();
            let g = &aGrid[(iU, iV)];
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeD3() {
    let aSurf = CreateMultiSpanBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());

    let aUParams = CreateUniformParams(0.0, 1.0, 11);
    let aVParams = CreateUniformParams(0.0, 1.0, 11);

    let aGrid: Array2<GeomGridEvalSurfD3> = anEval.evaluate_grid_d3(&aUParams, &aVParams).unwrap();

    let anAdaptor = GeomAdaptorSurface::with_surface(&aSurf.cast()).unwrap();

    for iU in 1..=11 {
        for iV in 1..=11 {
            let r = anAdaptor.d3(aUParams[iU], aVParams[iV]).unwrap();
            let g = &aGrid[(iU, iV)];
            EXPECT_NEAR(g.point.distance(&r.point), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1u - r.d1u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d1v - r.d1v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2u - r.d2u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2v - r.d2v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d2uv - r.d2uv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3u - r.d3u).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3v - r.d3v).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uuv - r.d3uuv).magnitude(), 0.0, THE_TOLERANCE);
            EXPECT_NEAR((g.d3uvv - r.d3uvv).magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

//=================================================================================================
// DN (Arbitrary Order Derivative) Tests
//=================================================================================================

// The DN comparison loop: the grid DN of order `(theNU, theNV)` against
// `Geom_BSplineSurface::DN` on `theNb` x `theNb` uniform parameters in [0, 1].
fn CheckGridDN(
    theSurf: &Handle<GeomBSplineSurface>,
    theEval: &GeomGridEvalBSplineSurface,
    theNb: i32,
    theOrders: (i32, i32),
) {
    let (aNU, aNV) = theOrders;
    let aUParams = CreateUniformParams(0.0, 1.0, theNb);
    let aVParams = CreateUniformParams(0.0, 1.0, theNb);
    let aGrid: Array2<Vector3> = theEval
        .evaluate_grid_dn(&aUParams, &aVParams, aNU, aNV)
        .unwrap();
    for iU in 1..=theNb {
        for iV in 1..=theNb {
            let aExpected: Vector3 = theSurf.dn(aUParams[iU], aVParams[iV], aNU, aNV).unwrap();
            EXPECT_NEAR(
                (aGrid[(iU, iV)] - aExpected).magnitude(),
                0.0,
                THE_TOLERANCE,
            );
        }
    }
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeDN_U1V0() {
    let aSurf = CreateSimpleBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());
    CheckGridDN(&aSurf, &anEval, 5, (1, 0));
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeDN_U0V1() {
    let aSurf = CreateSimpleBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());
    CheckGridDN(&aSurf, &anEval, 5, (0, 1));
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeDN_U1V1() {
    let aSurf = CreateSimpleBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());
    CheckGridDN(&aSurf, &anEval, 5, (1, 1));
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeDN_BeyondDegree() {
    let aSurf = CreateSimpleBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf);

    let aUParams = CreateUniformParams(0.0, 1.0, 5);
    let aVParams = CreateUniformParams(0.0, 1.0, 5);

    let aGrid: Array2<Vector3> = anEval.evaluate_grid_dn(&aUParams, &aVParams, 2, 0).unwrap();

    for iU in 1..=5 {
        for iV in 1..=5 {
            EXPECT_NEAR(aGrid[(iU, iV)].magnitude(), 0.0, THE_TOLERANCE);
        }
    }
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeDN_MultiSpan() {
    let aSurf = CreateMultiSpanBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());

    let aTestCases: [[i32; 2]; 7] = [[1, 0], [0, 1], [1, 1], [2, 0], [0, 2], [2, 1], [1, 2]];
    for aCase in aTestCases {
        CheckGridDN(&aSurf, &anEval, 11, (aCase[0], aCase[1]));
    }
}

#[test]
fn GeomGridEval_BSplineSurfaceTest__DerivativeDN_RationalSurface() {
    let aSurf = CreateRationalBSplineSurface();
    let anEval = GeomGridEvalBSplineSurface::new(aSurf.clone());

    for aNU in 0..=2 {
        for aNV in 0..=2 {
            if aNU + aNV == 0 {
                continue;
            }
            CheckGridDN(&aSurf, &anEval, 7, (aNU, aNV));
        }
    }
}
