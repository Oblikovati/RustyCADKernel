//! Direct translation of Convert_CylinderToBSplineSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkmath::bsplslib::{self, SurfaceSpan};
use rustycad_tkmath::convert::ElementarySurfaceToBSplineSurface;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Direction3, Point3};
use std::f64::consts::PI;

// gp_Cylinder(gp_Ax3(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)), radius).
fn cylinder(radius: f64) -> Cylinder3 {
    let frame = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Cylinder3::new(frame, radius).unwrap()
}

#[test]
fn Convert_CylinderToBSplineSurfaceTest__FullCylinder() {
    let aCyl = cylinder(3.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_cylinder(&aCyl, 0.0, 10.0).unwrap();

    assert_eq!(aConv.u_degree(), 2);
    assert_eq!(aConv.v_degree(), 1);
    assert!(aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
    assert!(aConv.nb_u_poles() > 0);
    assert!(aConv.nb_v_poles() > 0);
}

#[test]
fn Convert_CylinderToBSplineSurfaceTest__TrimmedCylinder() {
    let aCyl = cylinder(2.0);
    let (aU1, aU2) = (0.0, PI);
    let (aV1, aV2) = (-5.0, 5.0);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_cylinder_patch(&aCyl, aU1, aU2, aV1, aV2).unwrap();

    assert!(!aConv.is_u_periodic());
    assert!(!aConv.is_v_periodic());
    assert!(aConv.nb_u_poles() > 0);
    assert!(aConv.nb_v_poles() > 0);
}

#[test]
fn Convert_CylinderToBSplineSurfaceTest__WeightsArePositive() {
    let aCyl = cylinder(1.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_cylinder(&aCyl, 0.0, 1.0).unwrap();

    let aWeights = aConv.weights();
    for i in 1..=aConv.nb_u_poles() {
        for j in 1..=aConv.nb_v_poles() {
            assert!(aWeights[(i, j)] > 0.0);
        }
    }
}

#[test]
fn Convert_CylinderToBSplineSurfaceTest__KnotsAreMonotonic() {
    let aCyl = cylinder(1.0);
    let aConv = ElementarySurfaceToBSplineSurface::from_cylinder(&aCyl, 0.0, 1.0).unwrap();

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
fn Convert_CylinderToBSplineSurfaceTest__GeometricVerification() {
    let aRadius: f64 = 3.0;
    let (aV1, aV2) = (-5.0, 5.0);
    let aCyl = cylinder(aRadius);
    let aConv =
        ElementarySurfaceToBSplineSurface::from_cylinder_patch(&aCyl, 0.0, PI, aV1, aV2).unwrap();

    let aTol: f64 = 1.0e-10;
    // Sample 5x5 grid of BSpline parameters within the domain
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
                v_rat: false,
                u_per: false,
                v_per: false,
            };
            let aPnt: Point3 = bsplslib::d0(&aSpan, aConv.poles(), Some(aConv.weights())).unwrap();

            // Verify the point lies on the cylinder: distance from Z axis = R
            let aDistFromAxis: f64 = (aPnt.x() * aPnt.x() + aPnt.y() * aPnt.y()).sqrt();
            assert!(
                (aDistFromAxis - aRadius).abs() <= aTol,
                "Point not on cylinder at U={aU} V={aV}"
            );
            // Verify Z is within the V range
            assert!(aPnt.z() >= aV1 - aTol);
            assert!(aPnt.z() <= aV2 + aTol);
        }
    }
}
