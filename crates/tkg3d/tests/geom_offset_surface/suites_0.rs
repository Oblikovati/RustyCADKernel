// Geom_OffsetSurface_Test.cxx, the fixture tests and Geom_OffsetSurface_EquivalentSurface.

#[test]
fn Geom_OffsetSurface_Test__CopyConstructorBasicProperties() {
    let myOriginalSurface = SetUp();
    // Test copy constructor
    let aCopiedSurface = copied(&myOriginalSurface);

    // Verify basic properties are identical
    EXPECT_DOUBLE_EQ(myOriginalSurface.offset(), aCopiedSurface.offset());
    assert_eq!(myOriginalSurface.is_u_periodic(), aCopiedSurface.is_u_periodic());
    assert_eq!(myOriginalSurface.is_v_periodic(), aCopiedSurface.is_v_periodic());
    assert_eq!(
        myOriginalSurface.is_u_closed().unwrap(),
        aCopiedSurface.is_u_closed().unwrap()
    );
    assert_eq!(
        myOriginalSurface.is_v_closed().unwrap(),
        aCopiedSurface.is_v_closed().unwrap()
    );
}

#[test]
fn Geom_OffsetSurface_Test__CopyConstructorBasisSurface() {
    let myOriginalSurface = SetUp();
    let aCopiedSurface = copied(&myOriginalSurface);

    // Verify basis surfaces are equivalent but independent
    let anOrigBasis: Handle<dyn GeomSurface> = myOriginalSurface.basis_surface();
    let aCopyBasis: Handle<dyn GeomSurface> = aCopiedSurface.basis_surface();

    // They should be different objects
    assert!(anOrigBasis != aCopyBasis);

    // But functionally equivalent
    let (anUFirst, anULast, aVFirst, aVLast) = anOrigBasis.bounds();

    let (anUFirstCopy, anULastCopy, aVFirstCopy, aVLastCopy) = aCopyBasis.bounds();

    EXPECT_DOUBLE_EQ(anUFirst, anUFirstCopy);
    EXPECT_DOUBLE_EQ(anULast, anULastCopy);
    EXPECT_DOUBLE_EQ(aVFirst, aVFirstCopy);
    EXPECT_DOUBLE_EQ(aVLast, aVLastCopy);
}

#[test]
fn Geom_OffsetSurface_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalSurface = SetUp();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn GeomGeometry> = myOriginalSurface.copy().unwrap();
    let aCopiedSurface: Handle<GeomOffsetSurface> = aCopiedGeom.cast();

    assert!(!aCopiedSurface.is_null());

    // Verify the copy is functionally identical
    EXPECT_DOUBLE_EQ(myOriginalSurface.offset(), aCopiedSurface.offset());

    // Test evaluation at several points
    for u in steps(-5.0, 2.5, |x| x <= 5.0) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            let anOrigPnt = myOriginalSurface.value(u, v).unwrap();
            let aCopyPnt = aCopiedSurface.value(u, v).unwrap();
            assert!(anOrigPnt.is_equal(&aCopyPnt, 1e-10));
        }
    }
}

#[test]
fn Geom_OffsetSurface_Test__CopyIndependence() {
    let myOriginalSurface = SetUp();
    let aCopiedSurface = copied(&myOriginalSurface);

    // Store original offset value
    let anOrigOffset: f64 = aCopiedSurface.offset();

    // Modify the original surface
    myOriginalSurface.set_offset_value(15.0).unwrap();

    // Verify the copied surface is not affected
    EXPECT_DOUBLE_EQ(aCopiedSurface.offset(), anOrigOffset);
    assert_ne!(aCopiedSurface.offset(), myOriginalSurface.offset());
}

// Tests for Surface() method - equivalent surface computation for canonical surfaces.

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Plane_ReturnsTranslatedPlane() {
    // Create a plane at origin with Z-normal.
    let anAxis = z_frame(1.0, 2.0, 3.0);
    let aPlane: Handle<GeomPlane> = Handle::new(GeomPlane::new(anAxis));

    let anOffset: f64 = 5.0;
    let anOffsetSurf = offset_of(aPlane.clone().cast(), anOffset);

    // Get equivalent surface.
    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    // Should be a plane.
    let anEquivPlane: Handle<GeomPlane> = anEquiv.cast();
    assert!(!anEquivPlane.is_null());

    // Verify the plane is translated by offset along normal.
    let anOrigLoc: Point3 = aPlane.location();
    let anEquivLoc: Point3 = anEquivPlane.location();
    EXPECT_NEAR(anEquivLoc.z() - anOrigLoc.z(), anOffset, precision::confusion());
    EXPECT_NEAR(anEquivLoc.x(), anOrigLoc.x(), precision::confusion());
    EXPECT_NEAR(anEquivLoc.y(), anOrigLoc.y(), precision::confusion());
}

// `new Geom_CylindricalSurface(gp_Ax3(origin, Z), radius)` offset by `offset`, and its
// equivalent surface as a cylinder (both ASSERT_FALSE(IsNull) checks included).
fn equivalent_cylinder(radius: f64, offset: f64) -> Handle<GeomCylindricalSurface> {
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aCyl: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(anAxis, radius).unwrap()).cast();
    let anOffsetSurf = offset_of(aCyl, offset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    let anEquivCyl: Handle<GeomCylindricalSurface> = anEquiv.cast();
    assert!(!anEquivCyl.is_null());
    anEquivCyl
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Cylinder_PositiveOffset_ReturnsLargerCylinder() {
    // Create a cylinder with radius 10.
    let anEquivCyl = equivalent_cylinder(10.0, 3.0);

    // Radius should increase by offset.
    EXPECT_NEAR(anEquivCyl.radius(), 13.0, precision::confusion());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Cylinder_NegativeOffset_ReturnsSmallerCylinder() {
    let anEquivCyl = equivalent_cylinder(10.0, -3.0);

    EXPECT_NEAR(anEquivCyl.radius(), 7.0, precision::confusion());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Cylinder_NegativeRadiusFlip_ReturnsFlippedCylinder() {
    // Offset that makes radius negative should flip the surface.
    let anEquivCyl = equivalent_cylinder(5.0, -8.0); // Results in radius = -3

    // Radius should be absolute value.
    EXPECT_NEAR(anEquivCyl.radius(), 3.0, precision::confusion());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Sphere_PositiveOffset_ReturnsLargerSphere() {
    let anAxis = z_frame(1.0, 2.0, 3.0);
    let aSphere: Handle<GeomSphericalSurface> =
        Handle::new(GeomSphericalSurface::new(anAxis, 10.0).unwrap());

    let anOffset: f64 = 5.0;
    let anOffsetSurf = offset_of(aSphere.clone().cast(), anOffset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    let anEquivSphere: Handle<GeomSphericalSurface> = anEquiv.cast();
    assert!(!anEquivSphere.is_null());

    EXPECT_NEAR(anEquivSphere.radius(), 15.0, precision::confusion());
    // Center should remain the same.
    assert!(
        anEquivSphere
            .location()
            .is_equal(&aSphere.location(), precision::confusion())
    );
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Sphere_NegativeRadiusFlip_ReturnsFlippedSphere() {
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aSphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(anAxis, 5.0).unwrap()).cast();

    let anOffset: f64 = -8.0; // Results in radius = -3
    let anOffsetSurf = offset_of(aSphere, anOffset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    let anEquivSphere: Handle<GeomSphericalSurface> = anEquiv.cast();
    assert!(!anEquivSphere.is_null());

    EXPECT_NEAR(anEquivSphere.radius(), 3.0, precision::confusion());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Cone_PositiveOffset_ReturnsOffsetCone() {
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aSemiAngle: f64 = PI / 6.0; // 30 degrees
    let aRefRadius: f64 = 10.0;
    let aCone: Handle<dyn GeomSurface> =
        Handle::new(GeomConicalSurface::new(anAxis, aSemiAngle, aRefRadius).unwrap()).cast();

    let anOffset: f64 = 2.0;
    let anOffsetSurf = offset_of(aCone, anOffset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    let anEquivCone: Handle<GeomConicalSurface> = anEquiv.cast();
    assert!(!anEquivCone.is_null());

    // Semi-angle should remain the same.
    EXPECT_NEAR(anEquivCone.semi_angle(), aSemiAngle, precision::confusion());

    // Reference radius should change by offset * cos(alpha).
    let anExpectedRadius: f64 = aRefRadius + anOffset * aSemiAngle.cos();
    EXPECT_NEAR(anEquivCone.ref_radius(), anExpectedRadius, precision::confusion());
}

// `new Geom_ToroidalSurface(gp_Ax3(origin, Z), major, minor)` offset by `offset`, and
// its equivalent surface as a torus (both ASSERT_FALSE(IsNull) checks included).
fn equivalent_torus(major: f64, minor: f64, offset: f64) -> Handle<GeomToroidalSurface> {
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aTorus: Handle<dyn GeomSurface> =
        Handle::new(GeomToroidalSurface::new(anAxis, major, minor).unwrap()).cast();
    let anOffsetSurf = offset_of(aTorus, offset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    let anEquivTorus: Handle<GeomToroidalSurface> = anEquiv.cast();
    assert!(!anEquivTorus.is_null());
    anEquivTorus
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Torus_PositiveOffset_ReturnsLargerTorus() {
    let aMajorRadius: f64 = 20.0;
    let aMinorRadius: f64 = 5.0;
    let anOffset: f64 = 2.0;
    let anEquivTorus = equivalent_torus(aMajorRadius, aMinorRadius, anOffset);

    // Major radius should remain the same.
    EXPECT_NEAR(anEquivTorus.major_radius(), aMajorRadius, precision::confusion());
    // Minor radius should increase by offset.
    EXPECT_NEAR(
        anEquivTorus.minor_radius(),
        aMinorRadius + anOffset,
        precision::confusion(),
    );
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Torus_NegativeOffset_ReturnsSmallerTorus() {
    let aMajorRadius: f64 = 20.0;
    let aMinorRadius: f64 = 5.0;
    let anOffset: f64 = -2.0;
    let anEquivTorus = equivalent_torus(aMajorRadius, aMinorRadius, anOffset);

    EXPECT_NEAR(anEquivTorus.major_radius(), aMajorRadius, precision::confusion());
    EXPECT_NEAR(
        anEquivTorus.minor_radius(),
        aMinorRadius - 2.0,
        precision::confusion(),
    );
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__Torus_NegativeMinorRadiusFlip_ReturnsFlippedTorus() {
    let aMajorRadius: f64 = 20.0;
    let aMinorRadius: f64 = 3.0;
    let anOffset: f64 = -5.0; // Results in minor radius = -2
    let anEquivTorus = equivalent_torus(aMajorRadius, aMinorRadius, anOffset);

    EXPECT_NEAR(anEquivTorus.major_radius(), aMajorRadius, precision::confusion());
    EXPECT_NEAR(anEquivTorus.minor_radius(), 2.0, precision::confusion());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__ZeroOffset_ReturnsBasisSurface() {
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aCyl: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(anAxis, 10.0).unwrap()).cast();

    let anOffsetSurf = offset_of(aCyl, 0.0);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    // Should return the internal basis surface (which is a copy of the original).
    assert!(anEquiv == anOffsetSurf.basis_surface());

    // The equivalent should be geometrically identical to the original.
    let anEquivCyl: Handle<GeomCylindricalSurface> = anEquiv.cast();
    assert!(!anEquivCyl.is_null());
    EXPECT_NEAR(anEquivCyl.radius(), 10.0, precision::confusion());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__TrimmedPlane_ReturnsTrimmedTranslatedPlane() {
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aPlane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(anAxis)).cast();

    // Create trimmed surface.
    let aTrimmed: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&aPlane, (0.0, 10.0), (0.0, 5.0), (true, true))
            .unwrap(),
    )
    .cast();

    let anOffset: f64 = 3.0;
    let anOffsetSurf = offset_of(aTrimmed, anOffset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    // Should be a trimmed surface.
    let anEquivTrimmed: Handle<GeomRectangularTrimmedSurface> = anEquiv.cast();
    assert!(!anEquivTrimmed.is_null());

    // Verify bounds are preserved.
    let (U1, U2, V1, V2) = anEquivTrimmed.bounds();
    EXPECT_NEAR(U1, 0.0, precision::confusion());
    EXPECT_NEAR(U2, 10.0, precision::confusion());
    EXPECT_NEAR(V1, 0.0, precision::confusion());
    EXPECT_NEAR(V2, 5.0, precision::confusion());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__DegenerateCylinder_ReturnsNull() {
    // Offset that makes radius too small (degenerate case).
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aCyl: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(anAxis, 1.0).unwrap()).cast();

    // Offset close to -1 should result in degenerate surface.
    let anOffset: f64 = -1.0 + 0.5 * precision::confusion();
    let anOffsetSurf = offset_of(aCyl, anOffset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    // Degenerate surface should return null.
    assert!(anEquiv.is_null());
}

#[test]
fn Geom_OffsetSurface_EquivalentSurface__EquivalentSurface_EvaluationConsistency() {
    // Verify that evaluation on offset surface matches evaluation on equivalent surface.
    let anAxis = z_frame(0.0, 0.0, 0.0);
    let aCyl: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(anAxis, 10.0).unwrap()).cast();

    let anOffset: f64 = 3.0;
    let anOffsetSurf = offset_of(aCyl, anOffset);

    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());

    // Test evaluation at several points.
    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            let anOffsetPnt = anOffsetSurf.value(u, v).unwrap();
            let anEquivPnt = anEquiv.value(u, v).unwrap();
            assert!(
                anOffsetPnt.is_equal(&anEquivPnt, precision::confusion()),
                "Mismatch at u={u}, v={v}"
            );
        }
    }
}
