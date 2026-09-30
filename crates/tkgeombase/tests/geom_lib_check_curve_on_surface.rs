//! Direct translation of GeomLib_CheckCurveOnSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dEllipse, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSurface,
    GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::geom_lib::CheckCurveOnSurface as GeomLib_CheckCurveOnSurface;
use rustycad_tkmath::gp::{Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};
use std::f64::consts::PI as M_PI;

fn frame_z(origin: Point3) -> Axis2 {
    Axis2::with_x_direction(
        origin,
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap()
}

fn frame3_z() -> Axis3 {
    Axis3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap()
}

fn c3d(curve: Handle<dyn GeomCurve>, first: f64, last: f64) -> Handle<dyn Adaptor3dCurve> {
    Handle::new(GeomAdaptorCurve::with_range(&curve, first, last).unwrap()).cast()
}

fn curve_on_surface(
    pcurve: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
    surface: Handle<dyn GeomSurface>,
) -> Handle<Adaptor3dCurveOnSurface> {
    let c2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(&pcurve, first, last).unwrap()).cast();
    let surf: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&surface).unwrap()).cast();
    Handle::new(Adaptor3dCurveOnSurface::with_curve(&c2d, &surf).unwrap())
}

fn checked(
    curve: &Handle<dyn Adaptor3dCurve>,
    cos: &Handle<Adaptor3dCurveOnSurface>,
) -> GeomLib_CheckCurveOnSurface {
    let mut aCheck = GeomLib_CheckCurveOnSurface::new();
    aCheck.init(curve, precision::p_confusion());
    aCheck.perform(cos);
    aCheck
}

// Circle of radius theRadius at height theHeight on a cylinder of radius
// theCylRadius around OZ; the pcurve is the corresponding iso line in UV.
fn makeCircleOnCylinderCheck(
    theCircRadius: f64,
    theCylRadius: f64,
    theHeight: f64,
) -> GeomLib_CheckCurveOnSurface {
    let (aFirst, aLast) = (0.0, 2.0 * M_PI);

    let aCirc: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(frame_z(Point3::new(0.0, 0.0, theHeight)), theCircRadius).unwrap(),
    )
    .cast();
    let aC3d = c3d(aCirc, aFirst, aLast);

    let aCyl: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(frame3_z(), theCylRadius).unwrap()).cast();
    let aPLine: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, theHeight),
        Direction2::new(1.0, 0.0).unwrap(),
    ))
    .cast();

    let aCoS = curve_on_surface(aPLine, aFirst, aLast, aCyl);
    checked(&aC3d, &aCoS)
}

fn makeLineOnPlaneCheck(
    theCurveDirection: Direction3,
    thePCurveDirection: Direction2,
    theFirst: f64,
    theLast: f64,
) -> GeomLib_CheckCurveOnSurface {
    let aLine: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        theCurveDirection,
    ))
    .cast();
    let aC3d = c3d(aLine, theFirst, theLast);

    let aPlane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame3_z())).cast();
    let aPLine: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        thePCurveDirection,
    ))
    .cast();
    let aCoS = curve_on_surface(aPLine, theFirst, theLast, aPlane);
    checked(&aC3d, &aCoS)
}

fn makeCircleAndEllipseCheck(
    theMajorRadius: f64,
    theMinorRadius: f64,
    theFirst: f64,
    theLast: f64,
) -> GeomLib_CheckCurveOnSurface {
    let aCircle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(frame_z(Point3::new(0.0, 0.0, 0.0)), 2.0).unwrap()).cast();
    let aC3d = c3d(aCircle, theFirst, theLast);

    let anEllipse: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dEllipse::from_axis(
            Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap()),
            theMajorRadius,
            theMinorRadius,
            true,
        )
        .unwrap(),
    )
    .cast();
    let aPlane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame3_z())).cast();
    let aCoS = curve_on_surface(anEllipse, theFirst, theLast, aPlane);
    checked(&aC3d, &aCoS)
}

#[test]
fn GeomLib_CheckCurveOnSurfaceTest__AnalyticCoincident_ReportsZeroDeviation() {
    let aCheck = makeCircleOnCylinderCheck(2.0, 2.0, 5.0);

    assert!(aCheck.is_done());
    assert!(aCheck.max_distance() <= precision::confusion());
}

#[test]
fn GeomLib_CheckCurveOnSurfaceTest__AnalyticDeviating_ReportsActualDeviation() {
    // The 3D circle is 0.05 smaller than the cylinder carrying the pcurve: the
    // deviation is constant and equal to the radius difference. This guards the
    // constant-distance shortcut against swallowing real deviations.
    let aCheck = makeCircleOnCylinderCheck(1.95, 2.0, 5.0);

    assert!(aCheck.is_done());
    EXPECT_NEAR(aCheck.max_distance(), 0.05, 1.0e-9);
}

#[test]
fn GeomLib_CheckCurveOnSurfaceTest__LineOnPlane_ReportsZeroDeviation() {
    let aCheck = makeLineOnPlaneCheck(
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
        Direction2::new(1.0, 0.0).unwrap(),
        -10.0,
        10.0,
    );

    assert!(aCheck.is_done());
    assert!(aCheck.max_distance() <= precision::confusion());
}

#[test]
fn GeomLib_CheckCurveOnSurfaceTest__DeviatingLines_ReportsEndpointMaximum() {
    let aCheck = makeLineOnPlaneCheck(
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
        Direction2::new(0.0, 1.0).unwrap(),
        -2.0,
        3.0,
    );

    assert!(aCheck.is_done());
    EXPECT_NEAR(
        aCheck.max_distance(),
        18.0f64.sqrt(),
        precision::confusion(),
    );
    EXPECT_DOUBLE_EQ(aCheck.max_parameter(), 3.0);
}

#[test]
fn GeomLib_CheckCurveOnSurfaceTest__CircleAndEllipse_ReportsAnalyticMaximum() {
    let aCheck = makeCircleAndEllipseCheck(4.0, 1.0, 0.0, 2.0 * M_PI);

    assert!(aCheck.is_done());
    EXPECT_NEAR(aCheck.max_distance(), 2.0, precision::confusion());
}

#[test]
fn GeomLib_CheckCurveOnSurfaceTest__NegativeTrim_ReportsInteriorMaximum() {
    let aCheck = makeCircleAndEllipseCheck(2.0, 1.0, -2.0, -1.0);

    assert!(aCheck.is_done());
    EXPECT_NEAR(aCheck.max_distance(), 1.0, precision::confusion());
    EXPECT_NEAR(
        aCheck.max_parameter(),
        -0.5 * M_PI,
        precision::p_confusion(),
    );
}

#[test]
fn GeomLib_CheckCurveOnSurfaceTest__AnalyticAliasing_ReportsActualDeviation() {
    let (aFirst, aLast) = (0.0, 80.0 * M_PI);

    let aCircle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(frame_z(Point3::new(0.0, 0.0, 0.0)), 4.0).unwrap()).cast();
    let aC3d = c3d(aCircle, aFirst, aLast);

    let aTorus: Handle<dyn GeomSurface> =
        Handle::new(GeomToroidalSurface::new(frame3_z(), 3.0, 1.0).unwrap()).cast();
    let aPLine: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        Direction2::new(3.0, 4.0).unwrap(),
    ))
    .cast();
    let aCoS = curve_on_surface(aPLine, aFirst, aLast, aTorus);

    let mut aCheck = GeomLib_CheckCurveOnSurface::new();
    aCheck.init(&aC3d, precision::p_confusion());
    aCheck.perform(&aCoS);

    assert!(aCheck.is_done());
    EXPECT_NEAR(aCheck.max_distance(), 8.0, precision::confusion());
}
