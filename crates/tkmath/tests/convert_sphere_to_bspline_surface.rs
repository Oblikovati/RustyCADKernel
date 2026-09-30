//! Direct translation of Convert_SphereToBSplineSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkmath::bsplslib::{self, SurfaceSpan};
use rustycad_tkmath::convert::ElementarySurfaceToBSplineSurface;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Sphere3};
use std::f64::consts::PI;

// gp_Sphere(gp_Ax3(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), radius).
fn sphere(radius: f64) -> Sphere3 {
    let frame = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Sphere3::new(frame, radius).unwrap()
}

#[test]
fn Convert_SphereToBSplineSurfaceTest__FullSphere() {
    let aSphere = sphere(5.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_sphere(&aSphere).unwrap();

    assert_eq!(aConv.u_degree(), 2);
    assert_eq!(aConv.v_degree(), 2);
    assert!(aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
    assert!(aConv.nb_u_poles() > 0);
    assert!(aConv.nb_v_poles() > 0);
    assert!(aConv.nb_u_knots() > 0);
    assert!(aConv.nb_v_knots() > 0);

    // Check all weights are non-negative (sphere poles may have zero weights)
    let aWeights = aConv.weights();
    for i in 1..=aConv.nb_u_poles() {
        for j in 1..=aConv.nb_v_poles() {
            assert!(aWeights[(i, j)] >= 0.0);
        }
    }
}

#[test]
fn Convert_SphereToBSplineSurfaceTest__TrimmedUV() {
    let aSphere = sphere(3.0);
    let (aU1, aU2) = (0.0, PI);
    let (aV1, aV2) = (-PI / 4.0, PI / 4.0);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_sphere_patch(&aSphere, aU1, aU2, aV1, aV2).unwrap();

    assert!(!aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
    assert!(aConv.nb_u_poles() > 0);
    assert!(aConv.nb_v_poles() > 0);
}

#[test]
fn Convert_SphereToBSplineSurfaceTest__UTrimmed() {
    let aSphere = sphere(2.0);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_sphere_band(&aSphere, 0.0, PI, true).unwrap();

    assert!(!aConv.is_u_periodic());
}

#[test]
fn Convert_SphereToBSplineSurfaceTest__VTrimmed() {
    let aSphere = sphere(2.0);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_sphere_band(&aSphere, -PI / 4.0, PI / 4.0, false)
            .unwrap();

    assert!(aConv.is_u_periodic());
}

#[test]
fn Convert_SphereToBSplineSurfaceTest__KnotsAreMonotonic() {
    let aSphere = sphere(1.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_sphere(&aSphere).unwrap();

    let aUKnots = aConv.u_knots();
    for i in 2..=aConv.nb_u_knots() {
        assert!(aUKnots[i] > aUKnots[i - 1]);
    }
    let aVKnots = aConv.v_knots();
    for i in 2..=aConv.nb_v_knots() {
        assert!(aVKnots[i] > aVKnots[i - 1]);
    }
}

#[test]
fn Convert_SphereToBSplineSurfaceTest__GeometricVerification() {
    let aRadius: f64 = 5.0;
    let aSphere = sphere(aRadius);
    // Use trimmed sphere to avoid degenerate poles
    let aConv = ElementarySurfaceToBSplineSurface::from_sphere_patch(
        &aSphere,
        0.0,
        PI,
        -PI / 4.0,
        PI / 4.0,
    )
    .unwrap();

    let aTol: f64 = 1.0e-10;
    let aUK = aConv.u_knots();
    let aVK = aConv.v_knots();
    let (aUMin, aUMax) = (aUK[aUK.lower()], aUK[aUK.upper()]);
    let (aVMin, aVMax) = (aVK[aVK.lower()], aVK[aVK.upper()]);

    for i in 0..=4 {
        let aU: f64 = aUMin + f64::from(i) * (aUMax - aUMin) / 4.0;
        for j in 0..=4 {
            let aV: f64 = aVMin + f64::from(j) * (aVMax - aVMin) / 4.0;

            let aSpan = SurfaceSpan {
                u: aU,
                v: aV,
                u_index: 0,
                v_index: 0,
                u_knots: aConv.u_knots(),
                v_knots: aConv.v_knots(),
                u_mults: Some(aConv.u_multiplicities()),
                v_mults: Some(aConv.v_multiplicities()),
                u_degree: aConv.u_degree(),
                v_degree: aConv.v_degree(),
                u_rat: true,
                v_rat: true,
                u_per: false,
                v_per: false,
            };
            let aPnt: Point3 = bsplslib::d0(&aSpan, aConv.poles(), Some(aConv.weights())).unwrap();

            // Verify the point lies on the sphere: x^2 + y^2 + z^2 = R^2
            let aDistFromOrigin: f64 = aPnt.distance(&Point3::new(0.0, 0.0, 0.0));
            assert!(
                (aDistFromOrigin - aRadius).abs() <= aTol,
                "Point not on sphere at U={aU} V={aV}"
            );
        }
    }
}
