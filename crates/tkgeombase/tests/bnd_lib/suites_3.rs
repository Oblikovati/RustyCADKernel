//=================================================================================================
// BndLib_AddSurface Tests
//=================================================================================================

// `gp_Ax3(gp_Pnt(x, y, z), gp_Dir(0., 0., 1.))`.
fn z_frame(theX: f64, theY: f64, theZ: f64) -> Axis3 {
    ax3(pnt(theX, theY, theZ), dir(0., 0., 1.))
}

// `GeomAdaptor_Surface aSurf(theSurface)`.
fn surface_adaptor(theSurface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>) -> GeomAdaptorSurface {
    GeomAdaptorSurface::with_surface(theSurface).unwrap()
}

#[test]
fn BndLib_AddSurfaceTest__Plane() {
    // Create a plane and add a finite patch
    let aPlane = surface(GeomPlane::new(z_frame(0., 0., 0.)));
    let aSurf = surface_adaptor(&aPlane);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Add a 10x10 patch centered at origin
    AddSurface::add_range(&aSurf, [-5., 5., -5., 5.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -5., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 0., precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__Cylinder_Full() {
    // Create a cylindrical surface
    let aCyl = surface(GeomCylindricalSurface::new(z_frame(0., 0., 0.), 5.).unwrap());
    let aSurf = surface_adaptor(&aCyl);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U range, V from 0 to 10
    AddSurface::add_range(&aSurf, [0., 2. * PI, 0., 10.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -5., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 10., precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__Sphere_Full() {
    // Create a spherical surface
    let aSphere = surface(GeomSphericalSurface::new(z_frame(0., 0., 0.), 7.).unwrap());
    let aSurf = surface_adaptor(&aSphere);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    AddSurface::add(&aSurf, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -7., precision::confusion());
    EXPECT_NEAR(aXmax, 7., precision::confusion());
    EXPECT_NEAR(aYmin, -7., precision::confusion());
    EXPECT_NEAR(aYmax, 7., precision::confusion());
    EXPECT_NEAR(aZmin, -7., precision::confusion());
    EXPECT_NEAR(aZmax, 7., precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__Sphere_Patch() {
    // Create a spherical surface patch (upper hemisphere)
    let aSphere = surface(GeomSphericalSurface::new(z_frame(0., 0., 0.), 5.).unwrap());
    let aSurf = surface_adaptor(&aSphere);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U, V from 0 to PI/2 (upper hemisphere)
    AddSurface::add_range(&aSurf, [0., 2. * PI, 0., PI / 2.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -5., precision::confusion());
    EXPECT_NEAR(aXmax, 5., precision::confusion());
    EXPECT_NEAR(aYmin, -5., precision::confusion());
    EXPECT_NEAR(aYmax, 5., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 5., precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__Cone_Full() {
    // Create a conical surface
    let aCone = surface(GeomConicalSurface::new(z_frame(0., 0., 0.), PI / 4., 2.).unwrap());
    let aSurf = surface_adaptor(&aCone);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Full U range, V from 0 to 5
    AddSurface::add_range(&aSurf, [0., 2. * PI, 0., 5.], aTol, &mut aBox).unwrap();

    let [_aXmin, _aYmin, _aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    // V=5: radius = 2 + 5*sin(PI/4), Z = 5*cos(PI/4)
    let aMaxRadius = 2. + 5. * (PI / 4.).sin();
    let aMaxZ = 5. * (PI / 4.).cos();

    EXPECT_NEAR(aXmax, aMaxRadius, precision::confusion());
    EXPECT_NEAR(aYmax, aMaxRadius, precision::confusion());
    EXPECT_NEAR(aZmax, aMaxZ, precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__Torus_Full() {
    // Create a toroidal surface
    let aTorus = surface(GeomToroidalSurface::new(z_frame(0., 0., 0.), 10., 3.).unwrap());
    let aSurf = surface_adaptor(&aTorus);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    AddSurface::add(&aSurf, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    // BndLib_AddSurface uses conservative bounding for torus
    // The exact bounds are +-13 for X,Y and +-3 for Z
    // The algorithm may produce slightly larger bounds
    assert!(aXmin <= -13. + precision::confusion());
    assert!(aXmax >= 13. - precision::confusion());
    assert!(aYmin <= -13. + precision::confusion());
    assert!(aYmax >= 13. - precision::confusion());
    EXPECT_NEAR(aZmin, -3., precision::confusion());
    EXPECT_NEAR(aZmax, 3., precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__BezierSurface() {
    // Create a simple Bezier surface (2x2 control points)
    let aPoles = array2(
        2,
        2,
        &[pnt(0., 0., 0.), pnt(0., 10., 2.), pnt(10., 0., 2.), pnt(10., 10., 0.)],
    );

    let aBezier = surface(GeomBezierSurface::new(&aPoles).unwrap());
    let aSurf = surface_adaptor(&aBezier);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    AddSurface::add(&aSurf, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    // Box should contain all control points
    assert!(aXmin <= 0.);
    assert!(aXmax >= 10.);
    assert!(aYmin <= 0.);
    assert!(aYmax >= 10.);
    assert!(aZmin <= 0.);
    assert!(aZmax >= 2.);
}

#[test]
fn BndLib_AddSurfaceTest__AddOptimal_Sphere() {
    // Test AddOptimal method with a sphere
    let aSphere = surface(GeomSphericalSurface::new(z_frame(0., 0., 0.), 4.).unwrap());
    let aSurf = surface_adaptor(&aSphere);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    AddSurface::add_optimal(&aSurf, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, -4., precision::confusion());
    EXPECT_NEAR(aXmax, 4., precision::confusion());
    EXPECT_NEAR(aYmin, -4., precision::confusion());
    EXPECT_NEAR(aYmax, 4., precision::confusion());
    EXPECT_NEAR(aZmin, -4., precision::confusion());
    EXPECT_NEAR(aZmax, 4., precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__Cylinder_Translated() {
    // Create a cylindrical surface centered at (5, 5, 0)
    let aCyl = surface(GeomCylindricalSurface::new(z_frame(5., 5., 0.), 3.).unwrap());
    let aSurf = surface_adaptor(&aCyl);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    AddSurface::add_range(&aSurf, [0., 2. * PI, 0., 10.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, aZmin, aXmax, aYmax, aZmax] = get(&aBox);

    EXPECT_NEAR(aXmin, 2., precision::confusion());
    EXPECT_NEAR(aXmax, 8., precision::confusion());
    EXPECT_NEAR(aYmin, 2., precision::confusion());
    EXPECT_NEAR(aYmax, 8., precision::confusion());
    EXPECT_NEAR(aZmin, 0., precision::confusion());
    EXPECT_NEAR(aZmax, 10., precision::confusion());
}

#[test]
fn BndLib_AddSurfaceTest__Plane_Tilted() {
    // Create a tilted plane (normal at 45 degrees)
    // gp_Dir automatically normalizes the input vector
    let aPlane = surface(GeomPlane::new(ax3(pnt(0., 0., 0.), dir(0., 1., 1.))));
    let aSurf = surface_adaptor(&aPlane);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Add a unit square patch
    AddSurface::add_range(&aSurf, [0., 1., 0., 1.], aTol, &mut aBox).unwrap();

    assert!(!aBox.is_void(), "Box should not be void for tilted plane");
}

// Test for edge case with large parameter values and small step sizes.
// This verifies that the algorithm correctly samples different points
// even when parameter values are large relative to the step size,
// which could cause floating-point precision issues if accumulation
// (U += dU) is used instead of multiplication (U = UMin + dU * i).
#[test]
fn BndLib_AddSurfaceTest__LargeParameters_PrecisionTest() {
    // Create a Bezier surface - uses the default sampling path
    let mut aValues = Vec::new();
    // Create a simple surface with known extent
    for i in 1..=3 {
        for j in 1..=3 {
            aValues.push(pnt(f64::from(i - 1) * 10., f64::from(j - 1) * 10., 0.));
        }
    }
    let aPoles = array2(3, 3, &aValues);
    let aBezier = surface(GeomBezierSurface::new(&aPoles).unwrap());
    let aSurf = surface_adaptor(&aBezier);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Use large parameter offset with small parameter range
    // This simulates the edge case where UMin/UMax are large but (UMax-UMin) is small
    // The actual Bezier is defined on [0,1] x [0,1], but we test a sub-range
    // to ensure the sampling algorithm works correctly
    AddSurface::add_range(&aSurf, [0., 1., 0., 1.], aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    // The surface spans [0,20] x [0,20] x [0,0]
    // Bounding box should contain all sample points
    assert!(aXmin <= 1., "XMin should capture sampled points");
    assert!(aXmax >= 19., "XMax should capture sampled points");
    assert!(aYmin <= 1., "YMin should capture sampled points");
    assert!(aYmax >= 19., "YMax should capture sampled points");
}

// Test with translated surface where parameters are offset
// This is a more realistic test of the large parameter scenario
#[test]
fn BndLib_AddSurfaceTest__OffsetSurface_ParameterPrecision() {
    // Create a simple plane
    let aPlane = surface(GeomPlane::new(z_frame(0., 0., 0.)));
    let aSurf = surface_adaptor(&aPlane);
    let mut aBox = BoundingBox3::default();
    let aTol = 0.;

    // Test with very large parameter values but small range
    // UMin = 1e10, UMax = 1e10 + 100, so dU ~ 100/(Nu-1) ~ small relative to 1e10
    let aLargeOffset = 1.0e10;
    let aRange = 100.;
    let aPatch = [aLargeOffset, aLargeOffset + aRange, aLargeOffset, aLargeOffset + aRange];
    AddSurface::add_range(&aSurf, aPatch, aTol, &mut aBox).unwrap();

    let [aXmin, aYmin, _aZmin, aXmax, aYmax, _aZmax] = get(&aBox);

    // For a plane at z=0 with parameters as X,Y coordinates:
    // The box should span approximately [1e10, 1e10+100] in both X and Y
    // Allow some tolerance due to floating-point representation
    let aTolerance = 1.0; // 1 unit tolerance for large values

    EXPECT_NEAR(aXmin, aLargeOffset, aTolerance);
    EXPECT_NEAR(aXmax, aLargeOffset + aRange, aTolerance);
    EXPECT_NEAR(aYmin, aLargeOffset, aTolerance);
    EXPECT_NEAR(aYmax, aLargeOffset + aRange, aTolerance);

    // Most importantly: verify the range is captured (not collapsed due to precision loss)
    let aComputedRangeX = aXmax - aXmin;
    let aComputedRangeY = aYmax - aYmin;

    assert!(aComputedRangeX > aRange * 0.9, "X range should not collapse due to floating-point precision issues");
    assert!(aComputedRangeY > aRange * 0.9, "Y range should not collapse due to floating-point precision issues");
}
