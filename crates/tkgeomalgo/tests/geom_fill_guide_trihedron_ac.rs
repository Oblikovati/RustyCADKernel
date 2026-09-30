//! Direct translation of GeomFill_GuideTrihedronAC_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillGuideTrihedronAC, GeomFillTrihedronLaw, TrihedronD0, TrihedronD1, TrihedronD2,
};
use rustycad_tkmath::gp::Point3;

// Helper: create a cubic BSpline curve suitable for C2 evaluation.
fn makeBSplineCurve(
    theP1: Point3,
    theP2: Point3,
    theP3: Point3,
    theP4: Point3,
) -> Handle<dyn Adaptor3dCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = theP1;
    aPoles[2] = theP2;
    aPoles[3] = theP3;
    aPoles[4] = theP4;

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    let aCurve: Handle<dyn GeomCurve> =
        Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap()).cast();
    Handle::new(GeomAdaptorCurve::with_curve(&aCurve).unwrap()).cast()
}

fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn trihedron(scale: f64) -> GeomFillGuideTrihedronAC {
    let aGuide = makeBSplineCurve(
        pnt(0.0, 0.0, scale),
        pnt(scale, 0.0, scale),
        pnt(scale, scale, scale),
        pnt(0.0, scale, scale),
    );
    let aPath = makeBSplineCurve(
        pnt(0.0, 0.0, 0.0),
        pnt(scale, 0.0, 0.0),
        pnt(scale, scale, 0.0),
        pnt(0.0, scale, 0.0),
    );
    let aTrihedron = GeomFillGuideTrihedronAC::new(&aGuide).unwrap();
    aTrihedron.set_curve(&aPath).unwrap();
    aTrihedron
}

#[test]
fn GeomFill_GuideTrihedronAC__D0_ReturnsTrue() {
    let aTrihedron = trihedron(1.0);

    let mut aFrame = TrihedronD0::default();
    let isOk = aTrihedron.d0(0.5, &mut aFrame).unwrap();
    assert!(isOk);
    assert!(aFrame.tangent.magnitude() > precision::confusion());
    assert!(aFrame.normal.magnitude() > precision::confusion());
    assert!(aFrame.binormal.magnitude() > precision::confusion());
}

#[test]
fn GeomFill_GuideTrihedronAC__D1_ReturnsTrue() {
    let aTrihedron = trihedron(1.0);

    let mut aFrame = TrihedronD1::default();
    let isOk = aTrihedron.d1(0.5, &mut aFrame).unwrap();
    assert!(isOk);
    assert!(aFrame.tangent.magnitude() > precision::confusion());
    assert!(aFrame.normal.magnitude() > precision::confusion());
    assert!(aFrame.binormal.magnitude() > precision::confusion());
}

// Regression test for bug #7: D2 was returning false despite completing computation.
#[test]
fn GeomFill_GuideTrihedronAC__D2_ReturnsTrue() {
    let aTrihedron = trihedron(1.0);

    let mut aFrame = TrihedronD2::default();
    let isOk = aTrihedron.d2(0.5, &mut aFrame).unwrap();
    assert!(isOk);

    // Verify frame vectors are non-degenerate.
    assert!(aFrame.tangent.magnitude() > precision::confusion());
    assert!(aFrame.normal.magnitude() > precision::confusion());
    assert!(aFrame.binormal.magnitude() > precision::confusion());
}

// Verify D2 produces consistent results at multiple parameters.
#[test]
fn GeomFill_GuideTrihedronAC__D2_ConsistentAtMultipleParams() {
    let aTrihedron = trihedron(2.0);

    let aParams = [0.2, 0.4, 0.6, 0.8];
    for aParam in aParams {
        let mut aFrame = TrihedronD2::default();
        let isOk = aTrihedron.d2(aParam, &mut aFrame).unwrap();
        assert!(isOk, "D2 failed at param={aParam}");
    }
}
