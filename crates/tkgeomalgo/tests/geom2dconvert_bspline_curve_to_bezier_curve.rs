//! Direct translation of Geom2dConvert_BSplineCurveToBezierCurve_Test.cxx (TKGeomAlgo
//! GTests) at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_api::Interpolate;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkgeombase::geom2d_convert::BSplineCurveToBezierCurve;
use rustycad_tkmath::gp::Point2;

#[test]
fn Geom2dConvert_BSplineCurveToBezierCurveTest__OCC7372_PeriodicBSplineToBeziersAfterIncreaseDegree()
 {
    let aPoints: Handle<HArray1<Point2>> = Handle::new(HArray1::with_bounds(1, 5).unwrap());
    {
        let mut values = aPoints.get().unwrap().change_array1();
        values.set_value(1, Point2::new(100.0, 0.0)).unwrap();
        values.set_value(2, Point2::new(100.0, 100.0)).unwrap();
        values.set_value(3, Point2::new(0.0, 100.0)).unwrap();
        values.set_value(4, Point2::new(0.0, 0.0)).unwrap();
        values.set_value(5, Point2::new(50.0, -50.0)).unwrap();
    }

    let mut anInterp = Interpolate::new(&aPoints, true, 1e-6).unwrap();
    anInterp.perform().unwrap();
    assert!(anInterp.is_done());

    let aBSpline = anInterp.curve().unwrap();
    assert!(!aBSpline.is_null());

    aBSpline.get().unwrap().increase_degree(8).unwrap();

    let aConverter = BSplineCurveToBezierCurve::new(&aBSpline).unwrap();
    let aNbArcs = aConverter.nb_arcs().unwrap();
    assert_eq!(aNbArcs, 5);

    let anArc5 = aConverter.arc(5).unwrap();
    assert!(!anArc5.is_null());

    let anArc5Curve: Handle<dyn Geom2dCurve> = anArc5.cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&anArc5Curve).unwrap();
    let aLen = AbscissaPoint::length(&anAdaptor).unwrap();
    EXPECT_NEAR(aLen, 73.3203, 0.01);
}
