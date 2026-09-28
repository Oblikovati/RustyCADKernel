//=================================================================================================
// Sphere Tests
//=================================================================================================

#[test]
fn BndLibTest__Sphere_Full() {
    // Sphere of radius 7 centered at origin
    let aSphere = Sphere3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 7.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_sphere(&aSphere, aTol, &mut aBox);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -7., precision::confusion());
    EXPECT_NEAR(aXmax, 7., precision::confusion());
    EXPECT_NEAR(aYmin, -7., precision::confusion());
    EXPECT_NEAR(aYmax, 7., precision::confusion());
    EXPECT_NEAR(aZmin, -7., precision::confusion());
    EXPECT_NEAR(aZmax, 7., precision::confusion());
}

#[test]
fn BndLibTest__Sphere_Patch() {
    // Sphere patch: upper hemisphere (V from 0 to PI/2)
    let aSphere = Sphere3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 5.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U range, V from 0 to PI/2 (upper hemisphere)
    bnd_lib::add_sphere_patch(&aSphere, [0., 2. * PI, 0., PI / 2.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -5., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 5., precision::confusion());
}

#[test]
fn BndLibTest__Sphere_Translated() {
    // Sphere centered at (10, 20, 30) with radius 3
    let aSphere = Sphere3::new(ax3(pnt(10., 20., 30.), dir(0., 0., 1.)), 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_sphere(&aSphere, aTol, &mut aBox);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, 7., precision::confusion());
    EXPECT_NEAR(aXmax, 13., precision::confusion());
    EXPECT_NEAR(aYmin, 17., precision::confusion());
    EXPECT_NEAR(aYmax, 23., precision::confusion());
    EXPECT_NEAR(aZmin, 27., precision::confusion());
    EXPECT_NEAR(aZmax, 33., precision::confusion());
}

//=================================================================================================
// Cylinder Tests
//=================================================================================================

#[test]
fn BndLibTest__Cylinder_FinitePatch() {
    // Cylinder of radius 4, axis along Z
    let aCyl = Cylinder3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 4.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U range, V from 0 to 10
    bnd_lib::add_cylinder_patch(&aCyl, [0., 2. * PI, 0., 10.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -4., precision::confusion());
    EXPECT_NEAR(aXmax, 4., precision::confusion());
    EXPECT_NEAR(aYmin, -4., precision::confusion());
    EXPECT_NEAR(aYmax, 4., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 10., precision::confusion());
}

#[test]
fn BndLibTest__Cylinder_InfiniteV() {
    // Cylinder with infinite V parameter
    let aCyl = Cylinder3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 4.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_cylinder(&aCyl, 0., f64::INFINITY, aTol, &mut aBox).unwrap();

    assert!(aBox.is_open_zmax(), "Zmax should be open for cylinder with VMax=+Inf");
    assert!(!aBox.is_open_zmin(), "Zmin should not be open");
}

#[test]
fn BndLibTest__Cylinder_PartialU() {
    // Cylinder with partial U range (quarter cylinder)
    let aCyl = Cylinder3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 5.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // U from 0 to PI/2 (first quadrant), V from 0 to 10
    bnd_lib::add_cylinder_patch(&aCyl, [0., PI / 2., 0., 10.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, 0., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, 0., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 10., precision::confusion());
}

//=================================================================================================
// Cone Tests
//=================================================================================================

#[test]
fn BndLibTest__Cone_FinitePatch() {
    // Cone with semi-angle PI/4 (45 deg), reference radius 2
    // Cone parameterization: P(u,v) = Apex + (RefRadius + v*sin(SemiAngle)) * (cos(u)*XDir +
    // sin(u)*YDir)
    //                                      + v*cos(SemiAngle) * Axis
    let aCone = Cone3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), PI / 4., 2.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U range, V from 0 to 5
    bnd_lib::add_cone_patch(&aCone, [0., 2. * PI, 0., 5.], aTol, &mut aBox).unwrap();

    let [_aXmin, _aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    // V=0: radius = RefRadius = 2, Z = 0
    // V=5: radius = 2 + 5*sin(PI/4) = 2 + 5*sqrt(2)/2, Z = 5*cos(PI/4) = 5*sqrt(2)/2
    let aMaxRadius = 2. + 5. * (PI / 4.).sin();
    let aMaxZ = 5. * (PI / 4.).cos();
    EXPECT_NEAR(aXmax, aMaxRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aMaxRadius, precision::confusion());
    EXPECT_NEAR(aZmax, aMaxZ, precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
}

#[test]
fn BndLibTest__Cone_NegativeV() {
    // Cone with negative V range (cone extending in opposite direction)
    // Cone parameterization: Z = v*cos(SemiAngle)
    let aCone = Cone3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), PI / 6., 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U range, V from -5 to 0
    bnd_lib::add_cone_patch(&aCone, [0., 2. * PI, -5., 0.], aTol, &mut aBox).unwrap();

    let [_aXmin, _aYmin, aZmin, _aXmax, _aYmax, aZmax] = get(&aBox);

    // At V=0: radius = 3, Z = 0
    // At V=-5: radius = 3 + (-5)*sin(PI/6) = 3 - 2.5 = 0.5, Z = -5*cos(PI/6)
    let aExpectedZmin = -5. * (PI / 6.).cos();
    EXPECT_NEAR(aZmin, aExpectedZmin, precision::confusion());
    EXPECT_NEAR(aZmax, 0., precision::confusion());
}

#[test]
fn BndLibTest__Cone_InfiniteV() {
    // Cone with infinite V parameter
    // Note: BndLib::Add for cone with U,V parameters opens Z but not X/Y
    // since the cone's bounding in X/Y is computed from the circular cross-sections
    let aCone = Cone3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), PI / 4., 2.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_cone(&aCone, 0., f64::INFINITY, aTol, &mut aBox).unwrap();

    // For cone with infinite V, the Z direction should be open
    assert!(aBox.is_open_zmax(), "Zmax should be open for cone with VMax=+Inf");
}

//=================================================================================================
// Torus Tests
//=================================================================================================

#[test]
fn BndLibTest__Torus_Full() {
    // Torus with major radius 10, minor radius 3
    let aTorus = Torus3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 10., 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_torus(&aTorus, aTol, &mut aBox);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    // Torus extends from -(R+r) to +(R+r) in X and Y
    // and from -r to +r in Z
    EXPECT_NEAR(aXmin, -13., precision::confusion());
    EXPECT_NEAR(aXmax, 13., precision::confusion());
    EXPECT_NEAR(aYmin, -13., precision::confusion());
    EXPECT_NEAR(aYmax, 13., precision::confusion());
    EXPECT_NEAR(aZmin, -3., precision::confusion());
    EXPECT_NEAR(aZmax, 3., precision::confusion());
}

#[test]
fn BndLibTest__Torus_Patch() {
    // Torus patch: quarter in U, half in V
    // Torus parameterization: P(u,v) = Center + (R + r*cos(v)) * (cos(u)*XDir + sin(u)*YDir)
    //                                         + r*sin(v) * Axis
    let aTorus = Torus3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 10., 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // U from 0 to PI/2, V from 0 to PI
    bnd_lib::add_torus_patch(&aTorus, [0., PI / 2., 0., PI], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, _aXmax, _aYmax, aZmax] = get(&aBox);

    // Box should be valid and contain reasonable bounds
    assert!(!aBox.is_void(), "Box should not be void for torus patch");

    // For U in [0, PI/2], X and Y should be non-negative at minimum
    // The bounding box algorithm produces conservative bounds that may slightly
    // exceed R+r due to sampling/approximation
    assert!(aXmin >= -precision::confusion(), "Xmin should be >= 0 for first quadrant");
    assert!(aYmin >= -precision::confusion(), "Ymin should be >= 0 for first quadrant");

    // Z max should be approximately r (minor radius)
    assert!(aZmax <= 3. + precision::confusion(), "Zmax should not exceed r");
}

#[test]
fn BndLibTest__Torus_NegativeVParameter() {
    // Torus patch with negative V parameters.
    // This tests the fix for negative modulo: ((i % 8) + 8) % 8
    // The main purpose is to verify no crash occurs with negative V.
    let aTorus = Torus3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 10., 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U, V from -PI to 0
    bnd_lib::add_torus_patch(&aTorus, [0., 2. * PI, -PI, 0.], aTol, &mut aBox).unwrap();

    let [_aXmin, _aYmin, aZmin, _aXmax, _aYmax, _aZmax] = get(&aBox);

    // Should produce valid bounds without crash (main test for the fix)
    assert!(!aBox.is_void(), "Box should not be void for negative V parameters");

    // For V in [-PI, 0], Z = r*sin(v) ranges from -r (at v=-PI/2) to 0
    // The bounding box should contain this range
    assert!(aZmin <= 0., "Zmin should be <= 0 for V in [-PI, 0]");
    assert!(aZmin >= -3. - precision::confusion(), "Zmin should be >= -r");
}

#[test]
fn BndLibTest__Torus_LargeNegativeVParameter() {
    // Torus with large negative V parameter that would cause
    // out-of-bounds array access with incorrect modulo.
    // This is the primary test for the negative modulo fix.
    let aTorus = Torus3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 10., 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // V from -3*PI to -2*PI covers one full revolution
    bnd_lib::add_torus_patch(&aTorus, [0., 2. * PI, -3. * PI, -2. * PI], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    // Should produce valid bounds without crash (main test for the fix)
    assert!(!aBox.is_void(), "Box should not be void for large negative V parameters");

    // Full revolution in U gives full X/Y extent, the box may be slightly larger
    // due to conservative bounding box algorithm
    assert!(aXmin <= -13. + precision::confusion(), "Xmin should be at least -(R+r)");
    assert!(aXmax >= 13. - precision::confusion(), "Xmax should be at least R+r");
    assert!(aYmin <= -13. + precision::confusion(), "Ymin should be at least -(R+r)");
    assert!(aYmax >= 13. - precision::confusion(), "Ymax should be at least R+r");
}

#[test]
fn BndLibTest__Torus_Translated() {
    // Torus centered at (5, 5, 5)
    let aTorus = Torus3::new(ax3(pnt(5., 5., 5.), dir(0., 0., 1.)), 8., 2.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    bnd_lib::add_torus(&aTorus, aTol, &mut aBox);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, 5. - 10., precision::confusion());
    EXPECT_NEAR(aXmax, 5. + 10., precision::confusion());
    EXPECT_NEAR(aYmin, 5. - 10., precision::confusion());
    EXPECT_NEAR(aYmax, 5. + 10., precision::confusion());
    EXPECT_NEAR(aZmin, 5. - 2., precision::confusion());
    EXPECT_NEAR(aZmax, 5. + 2., precision::confusion());
}

//=================================================================================================
// 2D Tests
//=================================================================================================

#[test]
fn BndLibTest__Line2d_FiniteSegment() {
    let aLine = Line2::new(pnt2d(0., 0.), dir2d(1., 1.));
    let mut aBox = BoundingBox2::default();
    let aTol = 0.;

    bnd_lib::add_lin2d(&aLine, 0., 10., aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aXmax, aYmax] = get2d(&aBox);

    // Line in direction (1,1) normalized, so at t=10:
    // position = (10/sqrt(2), 10/sqrt(2))
    let aExpected = 10. / 2.0_f64.sqrt();
    EXPECT_NEAR(aXmin, 0., precision::confusion());
    EXPECT_NEAR(aXmax, aExpected, precision::confusion());
    EXPECT_NEAR(aYmin, 0., precision::confusion());
    EXPECT_NEAR(aYmax, aExpected, precision::confusion());
}

#[test]
fn BndLibTest__Circle2d_Full() {
    let aCirc = Circle2::from_x_axis(ax2d(0., 0., 1., 0.), 5., true).unwrap();
    let mut aBox = BoundingBox2::default();
    let aTol = 0.;

    bnd_lib::add_circ2d(&aCirc, aTol, &mut aBox);

    let [aXmin, aYmin, aXmax, aYmax] = get2d(&aBox);

    EXPECT_NEAR(aXmin, -5., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
}

#[test]
fn BndLibTest__Ellipse2d_Full() {
    let aElips = Ellipse2::from_x_axis(ax2d(0., 0., 1., 0.), 8., 4., true).unwrap();
    let mut aBox = BoundingBox2::default();
    let aTol = 0.;

    bnd_lib::add_elips2d(&aElips, aTol, &mut aBox);

    let [aXmin, aYmin, aXmax, aYmax] = get2d(&aBox);

    EXPECT_NEAR(aXmin, -8., precision::confusion());
    EXPECT_NEAR(aXmax, 8., precision::confusion());
    EXPECT_NEAR(aYmin, -4., precision::confusion());
    EXPECT_NEAR(aYmax, 4., precision::confusion());
}

#[test]
fn BndLibTest__Hyperbola2d_SimpleArc() {
    let aHypr = Hyperbola2::from_x_axis(ax2d(0., 0., 1., 0.), 3., 2., true).unwrap();
    let mut aBox = BoundingBox2::default();
    let aTol = 0.;

    bnd_lib::add_hypr2d(&aHypr, -1., 1., aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aXmax, aYmax] = get2d(&aBox);

    EXPECT_NEAR(aXmin, 3., precision::confusion());
    EXPECT_NEAR(aXmax, 3. * f64::cosh(1.), precision::confusion());
    EXPECT_NEAR(aYmin, 2. * f64::sinh(-1.), precision::confusion());
    EXPECT_NEAR(aYmax, 2. * f64::sinh(1.), precision::confusion());
}

#[test]
fn BndLibTest__Parabola2d_FiniteArc() {
    let aParab = Parabola2::from_mirror_axis(ax2d(0., 0., 1., 0.), 2., true).unwrap();
    let mut aBox = BoundingBox2::default();
    let aTol = 0.;

    bnd_lib::add_parab2d(&aParab, -4., 4., aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aXmax, aYmax] = get2d(&aBox);

    EXPECT_NEAR(aXmin, 0., precision::confusion());
    EXPECT_NEAR(aXmax, 2., precision::confusion());
    EXPECT_NEAR(aYmin, -4., precision::confusion());
    EXPECT_NEAR(aYmax, 4., precision::confusion());
}

//=================================================================================================
// Tolerance Tests
//=================================================================================================

#[test]
fn BndLibTest__Circle_WithTolerance() {
    let aCirc = Circle3::new(ax2(pnt(0., 0., 0.), dir(0., 0., 1.)), 5.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 0.5;

    bnd_lib::add_circ(&aCirc, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    // Box should be enlarged by tolerance
    EXPECT_NEAR(aXmin, -5.5, precision::confusion());
    EXPECT_NEAR(aXmax, 5.5, precision::confusion());
    EXPECT_NEAR(aYmin, -5.5, precision::confusion());
    EXPECT_NEAR(aYmax, 5.5, precision::confusion());
    EXPECT_NEAR(aZmin, -0.5, precision::confusion());
    EXPECT_NEAR(aZmax, 0.5, precision::confusion());
}

#[test]
fn BndLibTest__Sphere_WithTolerance() {
    let aSphere = Sphere3::new(ax3(pnt(0., 0., 0.), dir(0., 0., 1.)), 3.).unwrap();
    let mut aBox = BoundingBox3::default();
    let aTol = 1.;

    bnd_lib::add_sphere(&aSphere, aTol, &mut aBox);

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -4., precision::confusion());
    EXPECT_NEAR(aXmax, 4., precision::confusion());
    EXPECT_NEAR(aYmin, -4., precision::confusion());
    EXPECT_NEAR(aYmax, 4., precision::confusion());
    EXPECT_NEAR(aZmin, -4., precision::confusion());
    EXPECT_NEAR(aZmax, 4., precision::confusion());
}
