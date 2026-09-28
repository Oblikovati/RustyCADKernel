// ============================================================================
// Plane Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__Plane_CopiedPlanes_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPlane1 = plane(anAxis);
    let aPlane2: Handle<GeomPlane> = copied(&aPlane1);

    assert_eq!(myHasher.hash(&surface(&aPlane1)), myHasher.hash(&surface(&aPlane2)));
    assert!(myHasher.equal(&surface(&aPlane1), &surface(&aPlane2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__Plane_DifferentPlanes_DifferentHash() {
    let myHasher = SetUp();
    let aPlane1 = plane(ax3(pnt(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)));
    let aPlane2 = plane(ax3(pnt(0.0, 0.0, 1.0), dir(0.0, 0.0, 1.0)));

    assert_ne!(myHasher.hash(&surface(&aPlane1)), myHasher.hash(&surface(&aPlane2)));
    assert!(!myHasher.equal(&surface(&aPlane1), &surface(&aPlane2)));
}

// ============================================================================
// Cylindrical Surface Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__CylindricalSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCyl1 = Handle::new(GeomCylindricalSurface::new(anAxis, 5.0).unwrap());
    let aCyl2: Handle<GeomCylindricalSurface> = copied(&aCyl1);

    assert_eq!(myHasher.hash(&surface(&aCyl1)), myHasher.hash(&surface(&aCyl2)));
    assert!(myHasher.equal(&surface(&aCyl1), &surface(&aCyl2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__CylindricalSurface_DifferentRadius_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCyl1 = Handle::new(GeomCylindricalSurface::new(anAxis, 5.0).unwrap());
    let aCyl2 = Handle::new(GeomCylindricalSurface::new(anAxis, 10.0).unwrap());

    assert_ne!(myHasher.hash(&surface(&aCyl1)), myHasher.hash(&surface(&aCyl2)));
    assert!(!myHasher.equal(&surface(&aCyl1), &surface(&aCyl2)));
}

// ============================================================================
// Conical Surface Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__ConicalSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCone1 = Handle::new(GeomConicalSurface::new(anAxis, PI / 6.0, 5.0).unwrap());
    let aCone2: Handle<GeomConicalSurface> = copied(&aCone1);

    assert_eq!(myHasher.hash(&surface(&aCone1)), myHasher.hash(&surface(&aCone2)));
    assert!(myHasher.equal(&surface(&aCone1), &surface(&aCone2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__ConicalSurface_DifferentAngle_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aCone1 = Handle::new(GeomConicalSurface::new(anAxis, PI / 6.0, 5.0).unwrap());
    let aCone2 = Handle::new(GeomConicalSurface::new(anAxis, PI / 4.0, 5.0).unwrap());

    assert_ne!(myHasher.hash(&surface(&aCone1)), myHasher.hash(&surface(&aCone2)));
    assert!(!myHasher.equal(&surface(&aCone1), &surface(&aCone2)));
}

// ============================================================================
// Spherical Surface Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__SphericalSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aSphere1 = Handle::new(GeomSphericalSurface::new(anAxis, 5.0).unwrap());
    let aSphere2: Handle<GeomSphericalSurface> = copied(&aSphere1);

    assert_eq!(myHasher.hash(&surface(&aSphere1)), myHasher.hash(&surface(&aSphere2)));
    assert!(myHasher.equal(&surface(&aSphere1), &surface(&aSphere2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__SphericalSurface_DifferentRadius_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aSphere1 = Handle::new(GeomSphericalSurface::new(anAxis, 5.0).unwrap());
    let aSphere2 = Handle::new(GeomSphericalSurface::new(anAxis, 10.0).unwrap());

    assert_ne!(myHasher.hash(&surface(&aSphere1)), myHasher.hash(&surface(&aSphere2)));
    assert!(!myHasher.equal(&surface(&aSphere1), &surface(&aSphere2)));
}

// ============================================================================
// Toroidal Surface Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__ToroidalSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aTorus1 = Handle::new(GeomToroidalSurface::new(anAxis, 10.0, 3.0).unwrap());
    let aTorus2: Handle<GeomToroidalSurface> = copied(&aTorus1);

    assert_eq!(myHasher.hash(&surface(&aTorus1)), myHasher.hash(&surface(&aTorus2)));
    assert!(myHasher.equal(&surface(&aTorus1), &surface(&aTorus2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__ToroidalSurface_DifferentRadii_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aTorus1 = Handle::new(GeomToroidalSurface::new(anAxis, 10.0, 3.0).unwrap());
    let aTorus2 = Handle::new(GeomToroidalSurface::new(anAxis, 10.0, 5.0).unwrap());

    assert_ne!(myHasher.hash(&surface(&aTorus1)), myHasher.hash(&surface(&aTorus2)));
    assert!(!myHasher.equal(&surface(&aTorus1), &surface(&aTorus2)));
}

// ============================================================================
// BezierSurface Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__BezierSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let aPoles = square_poles(1.0);

    let aSurf1 = Handle::new(GeomBezierSurface::new(&aPoles).unwrap());
    let aSurf2: Handle<GeomBezierSurface> = copied(&aSurf1);

    assert_eq!(myHasher.hash(&surface(&aSurf1)), myHasher.hash(&surface(&aSurf2)));
    assert!(myHasher.equal(&surface(&aSurf1), &surface(&aSurf2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__BezierSurface_DifferentPoles_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles1 = square_poles(1.0);
    // The (2, 2) pole differs.
    let aPoles2 = square_poles(2.0);

    let aSurf1 = Handle::new(GeomBezierSurface::new(&aPoles1).unwrap());
    let aSurf2 = Handle::new(GeomBezierSurface::new(&aPoles2).unwrap());

    // Hash may be same (metadata only), but comparison should differ
    assert!(!myHasher.equal(&surface(&aSurf1), &surface(&aSurf2)));
}

// ============================================================================
// BSplineSurface Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__BSplineSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let aPoles = square_poles(1.0);
    let (aUKnots, aVKnots) = (reals(&[0.0, 1.0]), reals(&[0.0, 1.0]));
    let (aUMults, aVMults) = (integers(&[2, 2]), integers(&[2, 2]));

    let aSurf1 = Handle::new(
        GeomBSplineSurface::new(
            &aPoles,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    );
    let aSurf2: Handle<GeomBSplineSurface> = copied(&aSurf1);

    assert_eq!(myHasher.hash(&surface(&aSurf1)), myHasher.hash(&surface(&aSurf2)));
    assert!(myHasher.equal(&surface(&aSurf1), &surface(&aSurf2)));
}

// ============================================================================
// SurfaceOfRevolution Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__SurfaceOfRevolution_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let aLine = line(pnt(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let anAxis = Axis1::new(pnt(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let aRev1 = Handle::new(GeomSurfaceOfRevolution::new(&curve(&aLine), anAxis).unwrap());
    let aRev2: Handle<GeomSurfaceOfRevolution> = copied(&aRev1);

    assert_eq!(myHasher.hash(&surface(&aRev1)), myHasher.hash(&surface(&aRev2)));
    assert!(myHasher.equal(&surface(&aRev1), &surface(&aRev2)));
}

// ============================================================================
// SurfaceOfLinearExtrusion Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__SurfaceOfLinearExtrusion_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let aLine = line(pnt(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0));
    let aDir = dir(0.0, 0.0, 1.0);
    let anExt1 = Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve(&aLine), aDir).unwrap());
    let anExt2: Handle<GeomSurfaceOfLinearExtrusion> = copied(&anExt1);

    assert_eq!(myHasher.hash(&surface(&anExt1)), myHasher.hash(&surface(&anExt2)));
    assert!(myHasher.equal(&surface(&anExt1), &surface(&anExt2)));
}

// ============================================================================
// RectangularTrimmedSurface Tests
// ============================================================================

// `new Geom_RectangularTrimmedSurface(S, U1, U2, V1, V2)` with both senses true.
fn trimmed(
    basis: &Handle<dyn GeomSurface>,
    u: (f64, f64),
    v: (f64, f64),
) -> Handle<GeomRectangularTrimmedSurface> {
    Handle::new(GeomRectangularTrimmedSurface::new(basis, u, v, (true, true)).unwrap())
}

#[test]
fn GeomHash_SurfaceHasherTest__RectangularTrimmedSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPlane = plane(anAxis);
    let aTrimmed1 = trimmed(&surface(&aPlane), (0.0, 10.0), (0.0, 10.0));
    let aTrimmed2: Handle<GeomRectangularTrimmedSurface> = copied(&aTrimmed1);

    assert_eq!(myHasher.hash(&surface(&aTrimmed1)), myHasher.hash(&surface(&aTrimmed2)));
    assert!(myHasher.equal(&surface(&aTrimmed1), &surface(&aTrimmed2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__RectangularTrimmedSurface_DifferentBounds_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPlane = plane(anAxis);
    let aTrimmed1 = trimmed(&surface(&aPlane), (0.0, 10.0), (0.0, 10.0));
    let aTrimmed2 = trimmed(&surface(&aPlane), (0.0, 20.0), (0.0, 10.0));

    assert_ne!(myHasher.hash(&surface(&aTrimmed1)), myHasher.hash(&surface(&aTrimmed2)));
    assert!(!myHasher.equal(&surface(&aTrimmed1), &surface(&aTrimmed2)));
}

// ============================================================================
// OffsetSurface Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__OffsetSurface_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPlane = plane(anAxis);
    let anOffset1 = Handle::new(GeomOffsetSurface::new(&surface(&aPlane), 5.0, false).unwrap());
    let anOffset2: Handle<GeomOffsetSurface> = copied(&anOffset1);

    assert_eq!(myHasher.hash(&surface(&anOffset1)), myHasher.hash(&surface(&anOffset2)));
    assert!(myHasher.equal(&surface(&anOffset1), &surface(&anOffset2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__OffsetSurface_DifferentOffset_DifferentHash() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPlane = plane(anAxis);
    let anOffset1 = Handle::new(GeomOffsetSurface::new(&surface(&aPlane), 5.0, false).unwrap());
    let anOffset2 = Handle::new(GeomOffsetSurface::new(&surface(&aPlane), 10.0, false).unwrap());

    assert_ne!(myHasher.hash(&surface(&anOffset1)), myHasher.hash(&surface(&anOffset2)));
    assert!(!myHasher.equal(&surface(&anOffset1), &surface(&anOffset2)));
}

// ============================================================================
// Cross-type Tests
// ============================================================================

#[test]
fn GeomHash_SurfaceHasherTest__DifferentTypes_DifferentComparison() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPlane = plane(anAxis);
    let aSphere = Handle::new(GeomSphericalSurface::new(anAxis, 5.0).unwrap());

    assert!(!myHasher.equal(&surface(&aPlane), &surface(&aSphere)));
}

#[test]
fn GeomHash_SurfaceHasherTest__NullSurfaces_HandledCorrectly() {
    let myHasher = SetUp();
    let aNullSurface: Handle<dyn GeomSurface> = Handle::default();
    let anAxis = standard_axis();
    let aPlane = plane(anAxis);

    assert_eq!(myHasher.hash(&aNullSurface), 0usize);
    assert!(myHasher.equal(&aNullSurface, &aNullSurface));
    assert!(!myHasher.equal(&surface(&aPlane), &aNullSurface));
}

#[test]
fn GeomHash_SurfaceHasherTest__SameObject_Equal() {
    let myHasher = SetUp();
    let anAxis = standard_axis();
    let aPlane = plane(anAxis);
    assert!(myHasher.equal(&surface(&aPlane), &surface(&aPlane)));
}

// ============================================================================
// Weighted BSpline Surface Tests
// ============================================================================

// `new Geom_BSplineSurface(Poles, Weights, UKnots, VKnots, UMults, VMults, 1, 1)` on
// the single-span bilinear knots.
fn weighted_bilinear(
    aPoles: &Array2<'_, Point3>,
    aWeights: &Array2<'_, f64>,
    aUKnots: &Array1<'_, f64>,
) -> Handle<GeomBSplineSurface> {
    let aVKnots = reals(&[0.0, 1.0]);
    let (aUMults, aVMults) = (integers(&[2, 2]), integers(&[2, 2]));
    Handle::new(
        GeomBSplineSurface::with_weights(
            aPoles,
            aWeights,
            (aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    )
}

#[test]
fn GeomHash_SurfaceHasherTest__BSplineSurface_Weighted_CopiedSurfaces_SameHash() {
    let myHasher = SetUp();
    let aPoles = square_poles(1.0);
    let aWeights = net(&[&[1.0, 1.0], &[1.0, 2.0]]);

    let aSurf1 = weighted_bilinear(&aPoles, &aWeights, &reals(&[0.0, 1.0]));
    let aSurf2: Handle<GeomBSplineSurface> = copied(&aSurf1);

    assert_eq!(myHasher.hash(&surface(&aSurf1)), myHasher.hash(&surface(&aSurf2)));
    assert!(myHasher.equal(&surface(&aSurf1), &surface(&aSurf2)));
}

#[test]
fn GeomHash_SurfaceHasherTest__BSplineSurface_DifferentWeights_DifferentComparison() {
    let myHasher = SetUp();
    let aPoles = square_poles(1.0);
    let aWeights1 = net(&[&[1.0, 1.0], &[1.0, 2.0]]);
    // The (2, 2) weight differs.
    let aWeights2 = net(&[&[1.0, 1.0], &[1.0, 3.0]]);

    let aSurf1 = weighted_bilinear(&aPoles, &aWeights1, &reals(&[0.0, 1.0]));
    let aSurf2 = weighted_bilinear(&aPoles, &aWeights2, &reals(&[0.0, 1.0]));

    assert!(!myHasher.equal(&surface(&aSurf1), &surface(&aSurf2)));
}
