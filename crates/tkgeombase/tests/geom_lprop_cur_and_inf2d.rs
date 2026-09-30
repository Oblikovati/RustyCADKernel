//! Direct translation of GeomLProp_CurAndInf2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dParabola,
};
use rustycad_tkgeombase::geom_lprop::CurAndInf2d;
use rustycad_tkgeombase::lprop::CIType;
use rustycad_tkmath::gp::{Axis2d, Circle2, Direction2, Ellipse2, Hyperbola2, Parabola2, Point2};
use std::f64::consts::PI;

/// The GeomLProp_CurAndInf2dTest fixture (SetUp).
struct Fixture {
    myCircle: Handle<dyn Geom2dCurve>,
    myEllipse: Handle<dyn Geom2dCurve>,
    myHyperbola: Handle<dyn Geom2dCurve>,
    myParabola: Handle<dyn Geom2dCurve>,
}

fn SetUp() -> Fixture {
    let anAxis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    // Circle centered at origin, radius 5
    let aCirc = Circle2::from_x_axis(anAxis, 5.0, true).unwrap();
    // Ellipse centered at origin, major radius 10, minor radius 3
    let anElips = Ellipse2::from_x_axis(anAxis, 10.0, 3.0, true).unwrap();
    let aHypr = Hyperbola2::from_x_axis(anAxis, 6.0, 3.0, true).unwrap();
    let aParab = Parabola2::from_mirror_axis(anAxis, 2.0, true).unwrap();
    Fixture {
        myCircle: Handle::new(Geom2dCircle::from_circ2d(aCirc)).cast(),
        myEllipse: Handle::new(Geom2dEllipse::from_elips2d(anElips)).cast(),
        myHyperbola: Handle::new(Geom2dHyperbola::from_hypr2d(aHypr)).cast(),
        myParabola: Handle::new(Geom2dParabola::from_parab2d(aParab)).cast(),
    }
}

#[test]
fn GeomLProp_CurAndInf2dTest__Circle_Perform_NoInflections() {
    let f = SetUp();
    // A circle has constant curvature: no inflections or extrema
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform(&f.myCircle).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 0);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Circle_PerformInf_NoInflections() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_inf(&f.myCircle).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 0);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Circle_PerformCurExt_NoExtrema() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myCircle).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 0);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Ellipse_PerformCurExt_HasExtrema() {
    let f = SetUp();
    // An ellipse has 4 curvature extrema:
    // 2 maxima (at ends of minor axis) and 2 minima (at ends of major axis)
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 4);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Ellipse_PerformCurExt_Types() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 4);

    // Count min and max curvature points
    let mut aNbMin: i32 = 0;
    let mut aNbMax: i32 = 0;
    for i in 1..=aAnalyzer.nb_points() {
        let aType: CIType = aAnalyzer.type_of(i).unwrap();
        if aType == CIType::MinCur {
            aNbMin += 1;
        } else if aType == CIType::MaxCur {
            aNbMax += 1;
        }
    }
    assert_eq!(aNbMin, 2);
    assert_eq!(aNbMax, 2);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Ellipse_PerformCurExt_Parameters() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 4);

    // Parameters should be sorted in increasing order
    for i in 1..aAnalyzer.nb_points() {
        assert!(aAnalyzer.parameter(i).unwrap() < aAnalyzer.parameter(i + 1).unwrap());
    }
}

#[test]
fn GeomLProp_CurAndInf2dTest__Ellipse_PerformInf_NoInflections() {
    let f = SetUp();
    // An ellipse is a convex curve, so it has no inflection points
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_inf(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 0);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Ellipse_Perform_CombinedResult() {
    let f = SetUp();
    // Perform computes both inflections and curvature extrema
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    // Ellipse: 4 extrema, 0 inflections
    assert_eq!(aAnalyzer.nb_points(), 4);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Ellipse_CurvatureExtremaAtExpectedParameters() {
    let f = SetUp();
    // For an ellipse with major axis along X:
    // Curvature extrema at U = 0, PI/2, PI, 3*PI/2
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 4);

    let anExpectedParams: [f64; 4] = [0.0, PI / 2.0, PI, 3.0 * PI / 2.0];
    for i in 1..=4 {
        EXPECT_NEAR(
            aAnalyzer.parameter(i).unwrap(),
            anExpectedParams[i as usize - 1],
            1e-6,
        );
    }
}

#[test]
fn GeomLProp_CurAndInf2dTest__Hyperbola_PerformCurExt_VertexOnly() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myHyperbola).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 1);
    EXPECT_NEAR(
        aAnalyzer.parameter(1).unwrap(),
        0.0,
        precision::p_confusion(),
    );
    assert_eq!(aAnalyzer.type_of(1).unwrap(), CIType::MinCur);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Hyperbola_PerformInf_NoInflections() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_inf(&f.myHyperbola).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 0);
}

#[test]
fn GeomLProp_CurAndInf2dTest__Parabola_PerformCurExt_VertexOnly() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myParabola).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 1);
    EXPECT_NEAR(
        aAnalyzer.parameter(1).unwrap(),
        0.0,
        precision::p_confusion(),
    );
    assert_eq!(aAnalyzer.type_of(1).unwrap(), CIType::MinCur);
}

#[test]
fn GeomLProp_CurAndInf2dTest__PerformInf_ClearsPreviousExtrema() {
    let f = SetUp();
    let mut aAnalyzer = CurAndInf2d::new();
    aAnalyzer.perform_cur_ext(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 4);

    aAnalyzer.perform_inf(&f.myEllipse).unwrap();

    assert!(aAnalyzer.is_done());
    assert_eq!(aAnalyzer.nb_points(), 0);
}
