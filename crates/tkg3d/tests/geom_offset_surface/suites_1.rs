// Geom_OffsetSurface_Test.cxx, Geom_OffsetSurface_Derivatives and Geom_OffsetSurface_ThreeWay.

// The offset of `basis` by `offset`, an adaptor on it and an adaptor on its equivalent
// surface (with the ASSERT_FALSE(anEquiv.IsNull()) of every test).
fn adaptors(
    basis: Handle<dyn GeomSurface>,
    offset: f64,
) -> (Handle<GeomOffsetSurface>, GeomAdaptorSurface, GeomAdaptorSurface) {
    let anOffsetSurf = offset_of(basis, offset);

    // Create adaptors for offset surface and equivalent surface.
    let anOffsetAdaptor = adaptor_of(&anOffsetSurf.clone().cast());
    let anEquiv: Handle<dyn GeomSurface> = anOffsetSurf.surface().unwrap();
    assert!(!anEquiv.is_null());
    let anEquivAdaptor = adaptor_of(&anEquiv);
    (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor)
}

fn plane_at_123() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(z_frame(1.0, 2.0, 3.0))).cast()
}
fn cylinder(radius: f64) -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(z_frame(0.0, 0.0, 0.0), radius).unwrap()).cast()
}
fn sphere_at_123() -> Handle<dyn GeomSurface> {
    Handle::new(GeomSphericalSurface::new(z_frame(1.0, 2.0, 3.0), 10.0).unwrap()).cast()
}
fn cone_30() -> Handle<dyn GeomSurface> {
    let aSemiAngle: f64 = PI / 6.0;
    let aRefRadius: f64 = 10.0;
    Handle::new(GeomConicalSurface::new(z_frame(0.0, 0.0, 0.0), aSemiAngle, aRefRadius).unwrap())
        .cast()
}
fn torus_20_5() -> Handle<dyn GeomSurface> {
    let aMajorRadius: f64 = 20.0;
    let aMinorRadius: f64 = 5.0;
    Handle::new(
        GeomToroidalSurface::new(z_frame(0.0, 0.0, 0.0), aMajorRadius, aMinorRadius).unwrap(),
    )
    .cast()
}

// Tests for D0, D1, D2, D3 consistency between offset surface and equivalent surface via adaptor.

#[test]
fn Geom_OffsetSurface_Derivatives__Plane_D0D1D2D3_Consistency() {
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(plane_at_123(), 5.0);
    let aTol: f64 = 1e-10;

    // Test at several points.
    for u in steps(-5.0, 2.5, |x| x <= 5.0) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            check_d0(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d1(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d2(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
        }
    }
}

#[test]
fn Geom_OffsetSurface_Derivatives__Cylinder_D0D1D2D3_Consistency() {
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(cylinder(10.0), 3.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            check_d0(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d1(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d2(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d3(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
        }
    }
}

#[test]
fn Geom_OffsetSurface_Derivatives__Sphere_D0D1D2D3_Consistency() {
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(sphere_at_123(), 5.0);
    let aTol: f64 = 1e-10;

    // Avoid poles at v = +/- PI/2.
    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(-PI / 3.0, PI / 6.0, |x| x <= PI / 3.0) {
            check_d0(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d1(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d2(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
        }
    }
}

#[test]
fn Geom_OffsetSurface_Derivatives__Cone_D0D1D2_Consistency() {
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(cone_30(), 2.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(1.0, 3.0, |x| x <= 10.0) {
            // Avoid apex at v=0.
            check_d0(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d1(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d2(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
        }
    }
}

#[test]
fn Geom_OffsetSurface_Derivatives__Torus_D0D1D2D3_Consistency() {
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(torus_20_5(), 2.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
            check_d0(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d1(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d2(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
            check_d3(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
        }
    }
}

#[test]
fn Geom_OffsetSurface_Derivatives__Cylinder_DN_Consistency() {
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(cylinder(10.0), 3.0);
    let aTol: f64 = 1e-10;
    let u: f64 = PI / 3.0;
    let v: f64 = 2.5;

    // Test various derivative orders.
    for Nu in 0..=3 {
        for Nv in 0..=3 {
            if Nu + Nv == 0 || Nu + Nv > 3 {
                continue;
            }
            let aDN1: Vector3 = anOffsetAdaptor.dn(u, v, Nu, Nv).unwrap();
            let aDN2: Vector3 = anEquivAdaptor.dn(u, v, Nu, Nv).unwrap();
            assert!(
                VectorsEqual(aDN1, aDN2, aTol),
                "DN({Nu},{Nv}) mismatch at u={u}, v={v}"
            );
        }
    }
}

#[test]
fn Geom_OffsetSurface_Derivatives__NegativeOffset_Cylinder_D0D1D2_Consistency() {
    // Test with negative offset (smaller cylinder).
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(cylinder(10.0), -3.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 3.0, |x| x < 2.0 * PI) {
        for v in steps(-3.0, 2.0, |x| x <= 3.0) {
            check_d1(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
        }
    }
}

#[test]
fn Geom_OffsetSurface_Derivatives__FlippedCylinder_D0D1_Consistency() {
    // Test with offset that causes cylinder flip (negative radius).
    // Results in flipped cylinder with radius 3.
    let (_s, anOffsetAdaptor, anEquivAdaptor) = adaptors(cylinder(5.0), -8.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 3.0, |x| x < 2.0 * PI) {
        for v in steps(-3.0, 2.0, |x| x <= 3.0) {
            check_d1(&anOffsetAdaptor, &anEquivAdaptor, (u, v), aTol);
        }
    }
}

// Three-way consistency tests: Direct Geom_OffsetSurface vs OffsetAdaptor vs EquivalentAdaptor.

#[test]
fn Geom_OffsetSurface_ThreeWay__Cylinder_D0_ThreeWayConsistency() {
    // Verify that direct Geom_OffsetSurface, adaptor from offset surface, and adaptor
    // from equivalent surface all return the same values.
    let (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor) = adaptors(cylinder(10.0), 3.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            // 1. Direct evaluation on Geom_OffsetSurface.
            let aDirectPnt = anOffsetSurf.value(u, v).unwrap();
            // 2. Evaluation via adaptor from offset surface.
            let aOffsetAdaptorPnt = anOffsetAdaptor.d0(u, v).unwrap();
            // 3. Evaluation via adaptor from equivalent surface.
            let aEquivAdaptorPnt = anEquivAdaptor.d0(u, v).unwrap();

            assert!(
                aDirectPnt.is_equal(&aOffsetAdaptorPnt, aTol),
                "Direct vs OffsetAdaptor D0 mismatch at u={u}, v={v}"
            );
            assert!(
                aDirectPnt.is_equal(&aEquivAdaptorPnt, aTol),
                "Direct vs EquivAdaptor D0 mismatch at u={u}, v={v}"
            );
            assert!(
                aOffsetAdaptorPnt.is_equal(&aEquivAdaptorPnt, aTol),
                "OffsetAdaptor vs EquivAdaptor D0 mismatch at u={u}, v={v}"
            );
        }
    }
}

#[test]
fn Geom_OffsetSurface_ThreeWay__Cylinder_D1_ThreeWayConsistency() {
    let (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor) = adaptors(cylinder(10.0), 3.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            // 1. Direct evaluation on Geom_OffsetSurface.
            let aDirect = anOffsetSurf.d1(u, v).unwrap();
            // 2. Evaluation via adaptor from offset surface.
            let aOffsetAdaptor = anOffsetAdaptor.d1(u, v).unwrap();
            // 3. Evaluation via adaptor from equivalent surface.
            let aEquivAdaptor = anEquivAdaptor.d1(u, v).unwrap();

            // Point consistency.
            assert!(
                aDirect.point.is_equal(&aOffsetAdaptor.point, aTol),
                "Direct vs OffsetAdaptor D1 point mismatch at u={u}, v={v}"
            );
            assert!(
                aDirect.point.is_equal(&aEquivAdaptor.point, aTol),
                "Direct vs EquivAdaptor D1 point mismatch at u={u}, v={v}"
            );
            check_three_way_d1(&aDirect, &aOffsetAdaptor, &aEquivAdaptor, (u, v), aTol, "");
        }
    }
}

// The D1U and D1V checks of the three-way tests, `prefix` naming the surface.
fn check_three_way_d1(
    aDirect: &rustycad_tkg3d::geom::GeomSurfaceD1,
    aOffset: &rustycad_tkg3d::geom::GeomSurfaceD1,
    aEquiv: &rustycad_tkg3d::geom::GeomSurfaceD1,
    (u, v): (f64, f64),
    aTol: f64,
    prefix: &str,
) {
    // D1U consistency.
    assert!(
        VectorsEqual(aDirect.d1u, aOffset.d1u, aTol),
        "{prefix}Direct vs OffsetAdaptor D1U mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(aDirect.d1u, aEquiv.d1u, aTol),
        "{prefix}Direct vs EquivAdaptor D1U mismatch at u={u}, v={v}"
    );
    // D1V consistency.
    assert!(
        VectorsEqual(aDirect.d1v, aOffset.d1v, aTol),
        "{prefix}Direct vs OffsetAdaptor D1V mismatch at u={u}, v={v}"
    );
    assert!(
        VectorsEqual(aDirect.d1v, aEquiv.d1v, aTol),
        "{prefix}Direct vs EquivAdaptor D1V mismatch at u={u}, v={v}"
    );
}

#[test]
fn Geom_OffsetSurface_ThreeWay__Cylinder_D2_ThreeWayConsistency() {
    let (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor) = adaptors(cylinder(10.0), 3.0);
    let aTol: f64 = 1e-10;

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            // 1. Direct evaluation on Geom_OffsetSurface.
            let aDirect = anOffsetSurf.d2(u, v).unwrap();
            // 2. Evaluation via adaptor from offset surface.
            let aOffset = anOffsetAdaptor.d2(u, v).unwrap();
            // 3. Evaluation via adaptor from equivalent surface.
            let aEquiv = anEquivAdaptor.d2(u, v).unwrap();

            // D2U consistency.
            assert!(
                VectorsEqual(aDirect.d2u, aOffset.d2u, aTol),
                "Direct vs OffsetAdaptor D2U mismatch at u={u}, v={v}"
            );
            assert!(
                VectorsEqual(aDirect.d2u, aEquiv.d2u, aTol),
                "Direct vs EquivAdaptor D2U mismatch at u={u}, v={v}"
            );
            // D2V consistency.
            assert!(
                VectorsEqual(aDirect.d2v, aOffset.d2v, aTol),
                "Direct vs OffsetAdaptor D2V mismatch at u={u}, v={v}"
            );
            assert!(
                VectorsEqual(aDirect.d2v, aEquiv.d2v, aTol),
                "Direct vs EquivAdaptor D2V mismatch at u={u}, v={v}"
            );
            // D2UV consistency.
            assert!(
                VectorsEqual(aDirect.d2uv, aOffset.d2uv, aTol),
                "Direct vs OffsetAdaptor D2UV mismatch at u={u}, v={v}"
            );
            assert!(
                VectorsEqual(aDirect.d2uv, aEquiv.d2uv, aTol),
                "Direct vs EquivAdaptor D2UV mismatch at u={u}, v={v}"
            );
        }
    }
}

// The D0 and D1 part shared by the sphere, plane, torus and cone three-way tests.
fn check_three_way_d0_d1(
    anOffsetSurf: &GeomOffsetSurface,
    anOffsetAdaptor: &GeomAdaptorSurface,
    anEquivAdaptor: &GeomAdaptorSurface,
    (u, v): (f64, f64),
    prefix: &str,
) {
    let aTol: f64 = 1e-10;
    // D0.
    let aDirectPnt = anOffsetSurf.value(u, v).unwrap();
    let aOffsetAdaptorPnt = anOffsetAdaptor.d0(u, v).unwrap();
    let aEquivAdaptorPnt = anEquivAdaptor.d0(u, v).unwrap();

    assert!(
        aDirectPnt.is_equal(&aOffsetAdaptorPnt, aTol),
        "{prefix}Direct vs OffsetAdaptor D0 mismatch at u={u}, v={v}"
    );
    assert!(
        aDirectPnt.is_equal(&aEquivAdaptorPnt, aTol),
        "{prefix}Direct vs EquivAdaptor D0 mismatch at u={u}, v={v}"
    );

    // D1.
    let aDirect = anOffsetSurf.d1(u, v).unwrap();
    let aOffset = anOffsetAdaptor.d1(u, v).unwrap();
    let aEquiv = anEquivAdaptor.d1(u, v).unwrap();
    check_three_way_d1(&aDirect, &aOffset, &aEquiv, (u, v), aTol, prefix);
}

#[test]
fn Geom_OffsetSurface_ThreeWay__Sphere_D0D1D2_ThreeWayConsistency() {
    let (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor) = adaptors(sphere_at_123(), 5.0);

    // Avoid poles at v = +/- PI/2.
    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(-PI / 3.0, PI / 6.0, |x| x <= PI / 3.0) {
            check_three_way_d0_d1(&anOffsetSurf, &anOffsetAdaptor, &anEquivAdaptor, (u, v), "Sphere ");
        }
    }
}

#[test]
fn Geom_OffsetSurface_ThreeWay__Plane_D0D1D2_ThreeWayConsistency() {
    let (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor) = adaptors(plane_at_123(), 5.0);
    let aTol: f64 = 1e-10;

    for u in steps(-5.0, 2.5, |x| x <= 5.0) {
        for v in steps(-5.0, 2.5, |x| x <= 5.0) {
            check_three_way_d0_d1(&anOffsetSurf, &anOffsetAdaptor, &anEquivAdaptor, (u, v), "Plane ");

            // D2.
            let aDirect = anOffsetSurf.d2(u, v).unwrap();
            let aOffset = anOffsetAdaptor.d2(u, v).unwrap();
            let aEquiv = anEquivAdaptor.d2(u, v).unwrap();

            assert!(
                VectorsEqual(aDirect.d2u, aOffset.d2u, aTol),
                "Plane Direct vs OffsetAdaptor D2U mismatch at u={u}, v={v}"
            );
            assert!(
                VectorsEqual(aDirect.d2u, aEquiv.d2u, aTol),
                "Plane Direct vs EquivAdaptor D2U mismatch at u={u}, v={v}"
            );
            assert!(
                VectorsEqual(aDirect.d2v, aOffset.d2v, aTol),
                "Plane Direct vs OffsetAdaptor D2V mismatch at u={u}, v={v}"
            );
            assert!(
                VectorsEqual(aDirect.d2v, aEquiv.d2v, aTol),
                "Plane Direct vs EquivAdaptor D2V mismatch at u={u}, v={v}"
            );
        }
    }
}

#[test]
fn Geom_OffsetSurface_ThreeWay__Torus_D0D1D2_ThreeWayConsistency() {
    let (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor) = adaptors(torus_20_5(), 2.0);

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
            check_three_way_d0_d1(&anOffsetSurf, &anOffsetAdaptor, &anEquivAdaptor, (u, v), "Torus ");
        }
    }
}

#[test]
fn Geom_OffsetSurface_ThreeWay__Cone_D0D1_ThreeWayConsistency() {
    let (anOffsetSurf, anOffsetAdaptor, anEquivAdaptor) = adaptors(cone_30(), 2.0);

    for u in steps(0.0, PI / 4.0, |x| x < 2.0 * PI) {
        for v in steps(1.0, 3.0, |x| x <= 10.0) {
            // Avoid apex at v=0.
            check_three_way_d0_d1(&anOffsetSurf, &anOffsetAdaptor, &anEquivAdaptor, (u, v), "Cone ");
        }
    }
}
