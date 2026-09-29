// =========================================================================
// Plane
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__Plane_FinitePatch() {
    let aPlane = surface(GeomPlane::new(xoy()));
    let aSurf = Surface::from_surface(&aPlane).unwrap();
    assert_eq!(aSurf.get_type(), SurfaceType::Plane);

    let mut aBox = BoundingBox3::default();
    aSurf
        .add_range([-5.0, 5.0, -3.0, 3.0], 0.0, &mut aBox)
        .unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    EXPECT_NEAR(aXmin, -5.0, precision::confusion());
    EXPECT_NEAR(aYmin, -3.0, precision::confusion());
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aXmax, 5.0, precision::confusion());
    EXPECT_NEAR(aYmax, 3.0, precision::confusion());
    EXPECT_NEAR(aZmax, 0.0, precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__Plane_CompareWithBndLib() {
    let aPlane = surface(GeomPlane::new(xoy()));
    let aPatch = Some([-5.0, 5.0, -3.0, 3.0]);
    CompareBoxes(
        &new_box(&aPlane, aPatch),
        &old_box(&adaptor(&aPlane), aPatch),
        precision::confusion(),
    );
}

// =========================================================================
// Cylinder
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__Cylinder_Patch() {
    let aRadius = 3.0;
    let aCyl = surface(GeomCylindricalSurface::new(xoy(), aRadius).unwrap());
    let aSurf = Surface::from_surface(&aCyl).unwrap();
    assert_eq!(aSurf.get_type(), SurfaceType::Cylinder);

    let mut aBox = BoundingBox3::default();
    aSurf
        .add_range(
            [0.0, 2.0 * PI, 0.0, 10.0],
            precision::confusion(),
            &mut aBox,
        )
        .unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    EXPECT_NEAR(aXmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aYmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aZmin, 0.0, precision::confusion());
    EXPECT_NEAR(aXmax, aRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aRadius, precision::confusion());
    EXPECT_NEAR(aZmax, 10.0, precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__Cylinder_CompareWithBndLib() {
    let aCyl = surface(GeomCylindricalSurface::new(xoy(), 4.0).unwrap());
    let aPatch = Some([0.0, 2.0 * PI, -5.0, 5.0]);
    CompareBoxes(
        &new_box(&aCyl, aPatch),
        &old_box(&adaptor(&aCyl), aPatch),
        precision::confusion(),
    );
}

// =========================================================================
// Cone
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__Cone_Patch() {
    let aRefRadius = 2.0;
    let aSemiAngle = PI / 4.0;
    let aCone = Cone3::new(xoy(), aSemiAngle, aRefRadius).unwrap();
    let aConeSurf = surface(GeomConicalSurface::from_cone(aCone));
    let aSurf = Surface::from_surface(&aConeSurf).unwrap();
    assert_eq!(aSurf.get_type(), SurfaceType::Cone);

    let mut aBox = BoundingBox3::default();
    aSurf
        .add_range([0.0, 2.0 * PI, 0.0, 5.0], precision::confusion(), &mut aBox)
        .unwrap();

    let [aXmin, aXmax, aYmin, aYmax, _aZmin, _aZmax] = get3(&aBox);

    // At V=5, radius = aRefRadius + 5*sin(PI/4)
    let aMaxR = aRefRadius + 5.0 * aSemiAngle.sin();

    assert!(aXmax >= aMaxR - precision::confusion());
    assert!(aYmax >= aMaxR - precision::confusion());
    assert!(aXmin <= -aMaxR + precision::confusion());
    assert!(aYmin <= -aMaxR + precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__Cone_CompareWithBndLib() {
    let aCone = Cone3::new(xoy(), PI / 6.0, 3.0).unwrap();
    let aConeSurf = surface(GeomConicalSurface::from_cone(aCone));
    let aPatch = Some([0.0, 2.0 * PI, 0.0, 5.0]);
    CompareBoxes(
        &new_box(&aConeSurf, aPatch),
        &old_box(&adaptor(&aConeSurf), aPatch),
        precision::confusion(),
    );
}

// =========================================================================
// Sphere
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__Sphere_Full() {
    let aRadius = 5.0;
    let aSphere = sphere(aRadius);
    let aSurf = Surface::from_surface(&aSphere).unwrap();
    assert_eq!(aSurf.get_type(), SurfaceType::Sphere);

    let mut aBox = BoundingBox3::default();
    aSurf.add(0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    EXPECT_NEAR(aXmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aYmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aZmin, -aRadius, precision::confusion());
    EXPECT_NEAR(aXmax, aRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aRadius, precision::confusion());
    EXPECT_NEAR(aZmax, aRadius, precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__Sphere_Patch() {
    let aRadius = 5.0;
    let aSphere = sphere(aRadius);
    let aSurf = Surface::from_surface(&aSphere).unwrap();

    let mut aBox = BoundingBox3::default();
    // U in [0, PI/2], V in [0, PI/2] (first octant patch)
    aSurf
        .add_range(
            [0.0, PI / 2.0, 0.0, PI / 2.0],
            precision::confusion(),
            &mut aBox,
        )
        .unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    // Patch should be in the positive octant
    assert!(aXmin >= -precision::confusion());
    assert!(aYmin >= -precision::confusion());
    assert!(aZmin >= -precision::confusion());
    assert!(aXmax <= aRadius + precision::confusion());
    assert!(aYmax <= aRadius + precision::confusion());
    assert!(aZmax <= aRadius + precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__Sphere_CompareWithBndLib() {
    let aSphere = sphere(7.0);
    CompareBoxes(
        &new_box(&aSphere, None),
        &old_box(&adaptor(&aSphere), None),
        precision::confusion(),
    );
}

#[test]
fn GeomBndLib_SurfaceTest__Sphere_Patch_CompareWithBndLib() {
    let aSphere = sphere(7.0);
    let aPatch = Some([0.0, PI, -PI / 4.0, PI / 4.0]);
    CompareBoxes(
        &new_box(&aSphere, aPatch),
        &old_box(&adaptor(&aSphere), aPatch),
        precision::confusion(),
    );
}

// =========================================================================
// Torus
// =========================================================================

#[test]
fn GeomBndLib_SurfaceTest__Torus_Full() {
    let aMajR = 10.0;
    let aMinR = 3.0;
    let aTorus = torus();
    let aSurf = Surface::from_surface(&aTorus).unwrap();
    assert_eq!(aSurf.get_type(), SurfaceType::Torus);

    let mut aBox = BoundingBox3::default();
    aSurf.add(0.0, &mut aBox).unwrap();

    let [aXmin, aXmax, aYmin, aYmax, aZmin, aZmax] = get3(&aBox);

    let aOuterR = aMajR + aMinR;
    EXPECT_NEAR(aXmin, -aOuterR, precision::confusion());
    EXPECT_NEAR(aYmin, -aOuterR, precision::confusion());
    EXPECT_NEAR(aZmin, -aMinR, precision::confusion());
    EXPECT_NEAR(aXmax, aOuterR, precision::confusion());
    EXPECT_NEAR(aYmax, aOuterR, precision::confusion());
    EXPECT_NEAR(aZmax, aMinR, precision::confusion());
}

#[test]
fn GeomBndLib_SurfaceTest__Torus_CompareWithBndLib() {
    let aTorus = torus();
    // New implementation produces a tighter (better) box than old BndLib for full torus.
    // Verify new box is no larger than old box.
    expect_no_larger(&new_box(&aTorus, None), &old_box(&adaptor(&aTorus), None));
}
