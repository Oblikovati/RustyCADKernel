#[test]
fn GeomBndLib_SurfaceTest__Torus_Patch_CompareWithBndLib() {
    let aTorus = torus();
    let aPatch = Some([0.0, PI, 0.0, PI]);
    // New implementation produces a tighter (better) box than old BndLib for torus patches.
    // Verify new box is no larger than old box.
    expect_no_larger(
        &new_box(&aTorus, aPatch),
        &old_box(&adaptor(&aTorus), aPatch),
    );
}

// =========================================================================
// BezierSurface
// =========================================================================

fn bezier() -> Handle<dyn GeomSurface> {
    let aPoles = array2(
        2,
        2,
        &[
            pnt(0.0, 0.0, 0.0),
            pnt(10.0, 0.0, 0.0),
            pnt(0.0, 10.0, 0.0),
            pnt(10.0, 10.0, 5.0),
        ],
    );
    surface(GeomBezierSurface::new(&aPoles).unwrap())
}

#[test]
fn GeomBndLib_SurfaceTest__BezierSurface_Simple() {
    let aBezSurf = bezier();
    let aSurf = Surface::from_surface(&aBezSurf).unwrap();
    assert_eq!(aSurf.get_type(), SurfaceType::BezierSurface);

    let mut aBox = BoundingBox3::default();
    aSurf.add(precision::confusion(), &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    assert!(aXmin <= precision::confusion());
    assert!(aXmax >= 10.0 - precision::confusion());
    assert!(aYmin <= precision::confusion());
    assert!(aYmax >= 10.0 - precision::confusion());
    assert!(aZmin <= precision::confusion());
    assert!(aZmax >= 5.0 - precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__BezierSurface_CompareWithBndLib() {
    let aBezSurf = bezier();
    CompareBoxes(
        &new_box(&aBezSurf, None),
        &old_box(&adaptor(&aBezSurf), None),
        precision::confusion(),
    );
}

// =========================================================================
// BSplineSurface
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__BSplineSurface_CompareWithBndLib() {
    let aPoles = array2(
        3,
        3,
        &[
            pnt(0.0, 0.0, 0.0),
            pnt(5.0, 0.0, 2.0),
            pnt(10.0, 0.0, 0.0),
            pnt(0.0, 5.0, 1.0),
            pnt(5.0, 5.0, 5.0),
            pnt(10.0, 5.0, 1.0),
            pnt(0.0, 10.0, 0.0),
            pnt(5.0, 10.0, 2.0),
            pnt(10.0, 10.0, 0.0),
        ],
    );
    let aUKnots = array1(&[0.0, 1.0]);
    let aVKnots = array1(&[0.0, 1.0]);
    let aUMults = array1(&[3, 3]);
    let aVMults = array1(&[3, 3]);

    let aBSplSurf = surface(
        GeomBSplineSurface::new(
            &aPoles,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (2, 2),
            (false, false),
        )
        .unwrap(),
    );

    CompareBoxes(
        &new_box(&aBSplSurf, None),
        &old_box(&adaptor(&aBSplSurf), None),
        precision::confusion(),
    );
}

// =========================================================================
// Adaptor constructor
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__AdaptorConstructor_Sphere() {
    let aSphere = sphere(5.0);

    // Construct from handle
    let aBoxH = new_box(&aSphere, None);

    // Construct from adaptor
    let anAdaptor = adaptor(&aSphere);
    let aSurfA = Surface::from_adaptor(&anAdaptor).unwrap();
    assert_eq!(aSurfA.get_type(), SurfaceType::Sphere);

    let mut aBoxA = BoundingBox3::default();
    aSurfA.add(precision::confusion(), &mut aBoxA).unwrap();

    CompareBoxes(&aBoxH, &aBoxA, precision::confusion());
}

// =========================================================================
// AddOptimal
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__AddOptimal_Sphere() {
    let aSphere = sphere(5.0);
    let aSurf = Surface::from_surface(&aSphere).unwrap();

    let mut aBox = BoundingBox3::default();
    aSurf.add(precision::confusion(), &mut aBox).unwrap();

    let mut aBoxOpt = BoundingBox3::default();
    aSurf
        .add_optimal(precision::confusion(), &mut aBoxOpt)
        .unwrap();

    // AddOptimal should produce a box no larger than Add
    expect_no_larger(&aBoxOpt, &aBox);
}

#[test]
fn GeomBndLib_SurfaceTest__AddOptimal_CompareWithBndLib() {
    let aSphere = sphere(7.0);
    let anAdaptor = adaptor(&aSphere);

    let mut aNewBox = BoundingBox3::default();
    Surface::from_surface(&aSphere)
        .unwrap()
        .add_optimal(precision::confusion(), &mut aNewBox)
        .unwrap();

    let mut anOldBox = BoundingBox3::default();
    AddSurface::add_optimal(&anAdaptor, precision::confusion(), &mut anOldBox).unwrap();

    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__AddOptimal_Torus_TiltedBetterThanFast() {
    // Tilted torus: axis is (1,0,1)/sqrt(2), Ra=8, Rmi=3.
    // For a tilted torus the PI/4 circle-sampling used by Box() misses the true
    // extremum, so BoxOptimal (PSO) must give a tighter result.
    // Analytic: for axis n=(1/sqrt(2), 0, 1/sqrt(2)),
    //   Xmax = Zmax = Ra/sqrt(2) + Rmi = 4*sqrt(2)+3 ~= 8.657
    //   Ymax = Ra + Rmi = 11  (ny=0, so full equatorial radius applies)
    let aTiltedAx = Axis3::from_normal(gp::origin(), dir(1.0, 0.0, 1.0)).unwrap();
    let aTorus = surface(GeomToroidalSurface::new(aTiltedAx, 8.0, 3.0).unwrap());

    let aFastBox = new_box(&aTorus, None);

    let mut aOptBox = BoundingBox3::default();
    Surface::from_surface(&aTorus)
        .unwrap()
        .add_optimal(precision::confusion(), &mut aOptBox)
        .unwrap();

    let [aXminO, aXmaxO, aYminO, aYmaxO, aZminO, aZmaxO] = get3(&aOptBox);

    // Optimal must be no larger than the fast (conservative) box.
    expect_no_larger(&aOptBox, &aFastBox);

    // PSO must converge to the analytic tight bound.
    let aTrueXZ = 8.0 / 2.0_f64.sqrt() + 3.0; // ~= 8.6569
    let aTrueY = 8.0 + 3.0; // = 11.0
    let aTol = 10.0 * precision::confusion(); // PSO converges to ~Confusion()
    EXPECT_NEAR(aXmaxO, aTrueXZ, aTol);
    EXPECT_NEAR(-aXminO, aTrueXZ, aTol);
    EXPECT_NEAR(aYmaxO, aTrueY, aTol);
    EXPECT_NEAR(-aYminO, aTrueY, aTol);
    EXPECT_NEAR(aZmaxO, aTrueXZ, aTol);
    EXPECT_NEAR(-aZminO, aTrueXZ, aTol);
}

#[test]
fn GeomBndLib_SurfaceTest__AdaptorConstructor_TrimmedRangeUsedForFullAdd() {
    let aSphere = sphere(7.0);
    let anAdaptor =
        GeomAdaptorSurface::with_bounds(&aSphere, (0.0, PI, -PI / 4.0, PI / 4.0)).unwrap();

    let mut aNewBox = BoundingBox3::default();
    Surface::from_adaptor(&anAdaptor)
        .unwrap()
        .add(precision::confusion(), &mut aNewBox)
        .unwrap();

    CompareBoxes(&aNewBox, &old_box(&anAdaptor, None), precision::confusion());
}

fn trimmed_sphere() -> Handle<dyn GeomSurface> {
    let aSphere = sphere(7.0);
    surface(
        GeomRectangularTrimmedSurface::new(
            &aSphere,
            (0.0, PI),
            (-PI / 4.0, PI / 4.0),
            (true, true),
        )
        .unwrap(),
    )
}

#[test]
fn GeomBndLib_SurfaceTest__TrimmedSurfaceHandle_Fallback_CompareWithBndLib() {
    let aTrim = trimmed_sphere();

    let aNewBox = new_box(&aTrim, None);
    let anOldBox = old_box(&adaptor(&aTrim), None);

    assert!(!aNewBox.is_void());
    CompareBoxes(&aNewBox, &anOldBox, precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__RectangularTrimmedSurface_UsesBasisSpecialization() {
    let aTrim = trimmed_sphere();

    let aSurf = Surface::from_surface(&aTrim).unwrap();
    assert_eq!(aSurf.get_type(), SurfaceType::Sphere);

    let mut aNewBox = BoundingBox3::default();
    aSurf.add(precision::confusion(), &mut aNewBox).unwrap();
    CompareBoxes(
        &aNewBox,
        &old_box(&adaptor(&aTrim), None),
        precision::confusion(),
    );
}

#[test]
fn GeomBndLib_SurfaceTest__Torus_NegativeVMin_NoUnderBounding() {
    let aTorus = torus();
    let aU1 = 0.0;
    let aU2 = 2.0 * PI;
    let aV1 = -PI / 8.0;
    let aV2 = PI / 8.0;

    let aNewBox = new_box(&aTorus, Some([aU1, aU2, aV1, aV2]));

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aNewBox);
    let aTol = precision::confusion();
    for i in 0..=72 {
        let aU = aU1 + (aU2 - aU1) * f64::from(i) / 72.0;
        for j in 0..=16 {
            let aV = aV1 + (aV2 - aV1) * f64::from(j) / 16.0;
            let aP = aTorus.get().unwrap().value(aU, aV).unwrap();
            assert!(aP.x() >= aXmin - aTol);
            assert!(aP.x() <= aXmax + aTol);
            assert!(aP.y() >= aYmin - aTol);
            assert!(aP.y() <= aYmax + aTol);
            assert!(aP.z() >= aZmin - aTol);
            assert!(aP.z() <= aZmax + aTol);
        }
    }
}
