//! Direct translation of BRep_Tool_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! BRepPrimAPI_MakeBox and BRepBuilderAPI_MakeEdge are the BRep fakes of `support`
//! (TKPrim and TKTopAlgo are not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCircle, GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{self, Axis2, Axis3, Point3};
use rustycad_tkmath::toploc::Location;
use support::{
    expect_double_eq, make_box, make_edge, make_edge_on_curve, make_edge_on_whole_curve,
};

fn gp_Pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// `BRepPrimAPI_MakeBox aBoxMaker(10.0, 20.0, 30.0); aBoxMaker.Shape()`; the fake is
/// always done.
fn box_shape() -> Shape {
    make_box(10.0, 20.0, 30.0).into_shape()
}

/// The first sub-shape of `kind` found by `TopExp_Explorer`.
fn first_of(shape: &Shape, kind: ShapeEnum) -> Shape {
    let anExp = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    assert!(anExp.more());
    anExp.current().clone()
}

/// `new Geom_Circle(gp_Ax2(Origin, DZ), radius)`.
fn circle(radius: f64) -> Handle<dyn GeomCurve> {
    let anAx2 = Axis2::new(gp::origin(), gp::dz()).unwrap();
    Handle::new(GeomCircle::new(anAx2, radius).unwrap()).into_cast()
}

/// `new Geom_Plane(gp::XOY())`.
fn xoy_plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::from_axis2(gp::xoy()))).into_cast()
}

#[test]
fn BRep_Tool_Test__Pnt_FromVertex() {
    let aBox = box_shape();

    let aVertex = topods::vertex(&first_of(&aBox, ShapeEnum::Vertex)).unwrap();
    let aPnt = BRepTool::pnt(&aVertex).unwrap();

    // The point coordinates should be within the box bounds [0,10] x [0,20] x [0,30]
    assert!(aPnt.x() >= -precision::confusion());
    assert!(aPnt.x() <= 10.0 + precision::confusion());
    assert!(aPnt.y() >= -precision::confusion());
    assert!(aPnt.y() <= 20.0 + precision::confusion());
    assert!(aPnt.z() >= -precision::confusion());
    assert!(aPnt.z() <= 30.0 + precision::confusion());
}

#[test]
fn BRep_Tool_Test__Tolerance_Vertex() {
    let aBox = box_shape();

    let aVertex = topods::vertex(&first_of(&aBox, ShapeEnum::Vertex)).unwrap();
    let aTol = BRepTool::vertex_tolerance(&aVertex).unwrap();
    assert!(aTol >= 0.0);
}

#[test]
fn BRep_Tool_Test__Tolerance_Edge() {
    let aBox = box_shape();

    let anEdge = topods::edge(&first_of(&aBox, ShapeEnum::Edge)).unwrap();
    let aTol = BRepTool::edge_tolerance(&anEdge).unwrap();
    assert!(aTol >= 0.0);
}

#[test]
fn BRep_Tool_Test__Tolerance_Face() {
    let aBox = box_shape();

    let aFace = topods::face(&first_of(&aBox, ShapeEnum::Face)).unwrap();
    let aTol = BRepTool::face_tolerance(&aFace).unwrap();
    assert!(aTol >= 0.0);
}

#[test]
fn BRep_Tool_Test__Curve_FromEdge() {
    let aBox = box_shape();

    let anEdge = topods::edge(&first_of(&aBox, ShapeEnum::Edge)).unwrap();
    let (aCurve, aFirst, aLast) = BRepTool::curve(&anEdge).unwrap();

    assert!(
        !aCurve.is_null(),
        "Curve from a box edge should not be null"
    );
    assert!(
        aFirst < aLast,
        "First parameter should be less than last parameter"
    );
}

#[test]
fn BRep_Tool_Test__Surface_FromFace() {
    let aBox = box_shape();

    let aFace = topods::face(&first_of(&aBox, ShapeEnum::Face)).unwrap();
    let aSurface = BRepTool::surface(&aFace).unwrap();

    assert!(
        !aSurface.is_null(),
        "Surface from a box face should not be null"
    );
}

#[test]
fn BRep_Tool_Test__IsClosed_CircleEdge() {
    let aCircle = circle(5.0);
    let anEdge = make_edge_on_whole_curve(&aCircle);

    assert!(
        BRepTool::is_closed(&anEdge).unwrap(),
        "A full circle edge should be closed"
    );
}

#[test]
fn BRep_Tool_Test__IsClosed_LineEdge() {
    let anEdge = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(10.0, 0.0, 0.0));

    assert!(
        !BRepTool::is_closed(&anEdge).unwrap(),
        "A line segment edge should not be closed"
    );
}

#[test]
fn BRep_Tool_Test__Degenerated() {
    let aBox = box_shape();

    let anEdge = topods::edge(&first_of(&aBox, ShapeEnum::Edge)).unwrap();
    assert!(
        !BRepTool::degenerated(&anEdge).unwrap(),
        "Box edges should not be degenerated"
    );
}

#[test]
fn BRep_Tool_Test__CurveOnPlane_RejectsEdgeRangeOutsideBoundedCurve() {
    let aPolesBuffer = [
        gp_Pnt(0.0, 0.0, 0.0),
        gp_Pnt(0.5, 0.25, 0.0),
        gp_Pnt(1.0, 0.0, 0.0),
    ];
    let aPoles = Array1::from_shared(&aPolesBuffer, 1);
    let aCurve: Handle<dyn GeomCurve> =
        Handle::new(GeomBezierCurve::new(&aPoles).unwrap()).into_cast();

    let anEdge = make_edge_on_whole_curve(&aCurve);

    const THE_EDGE_FIRST: f64 = -0.000002673149646;
    const THE_EDGE_LAST: f64 = 0.999997326850354;
    BRepBuilder::default()
        .range(&anEdge, THE_EDGE_FIRST, THE_EDGE_LAST, true)
        .unwrap();

    let aPlane = xoy_plane();
    let aResult = BRepTool::curve_on_surface(&anEdge, &aPlane, &Location::default());
    assert!(aResult.is_ok(), "CurveOnSurface should not throw");
    let aResult = aResult.unwrap();

    assert!(aResult.pcurve.is_null());
    assert!(!aResult.is_stored);
    expect_double_eq(aResult.first, THE_EDGE_FIRST);
    expect_double_eq(aResult.last, THE_EDGE_LAST);
}

#[test]
fn BRep_Tool_Test__CurveOnPlane_ProjectsShiftedPeriodicRange() {
    let aCircle = circle(1.0);
    let anEdge = make_edge_on_whole_curve(&aCircle);

    let aPeriod = aCircle.get().unwrap().period().unwrap();
    let aRangeFirst = aPeriod;
    let aRangeLast = 2.0 * aPeriod;
    BRepBuilder::default()
        .range(&anEdge, aRangeFirst, aRangeLast, true)
        .unwrap();

    let aPlane = xoy_plane();
    let aResult = BRepTool::curve_on_plane(&anEdge, &aPlane, &Location::default());
    assert!(aResult.is_ok(), "CurveOnPlane should not throw");
    let (aPCurve, aFirst, aLast) = aResult.unwrap();

    assert!(!aPCurve.is_null());
    expect_double_eq(aFirst, aRangeFirst);
    expect_double_eq(aLast, aRangeLast);
}

#[test]
fn BRep_Tool_Test__CurveOnPlane_ProjectsPeriodicEdgeBuiltFromEqualParameters() {
    let aCircle = circle(1.0);
    let anEdge = make_edge_on_curve(&aCircle, 0.0, 0.0);

    let (aEdgeFirst, aEdgeLast) = BRepTool::range(&anEdge).unwrap();
    let aPeriod = aCircle.get().unwrap().period().unwrap();
    assert!(((aEdgeLast - aEdgeFirst) - aPeriod).abs() <= precision::p_confusion());

    let aPlane = xoy_plane();
    let aResult = BRepTool::curve_on_plane(&anEdge, &aPlane, &Location::default());
    assert!(aResult.is_ok(), "CurveOnPlane should not throw");
    let (aPCurve, aFirst, aLast) = aResult.unwrap();

    assert!(!aPCurve.is_null());
    expect_double_eq(aFirst, aEdgeFirst);
    expect_double_eq(aLast, aEdgeLast);
}

#[test]
fn BRep_Tool_Test__CurveOnPlane_RejectsEqualPeriodicRange() {
    let aCircle = circle(1.0);
    let anEdge = make_edge_on_whole_curve(&aCircle);
    BRepBuilder::default()
        .range(&anEdge, 0.0, 0.0, true)
        .unwrap();

    let aPlane = xoy_plane();
    let aResult = BRepTool::curve_on_plane(&anEdge, &aPlane, &Location::default());
    assert!(aResult.is_ok(), "CurveOnPlane should not throw");
    let (aPCurve, aFirst, aLast) = aResult.unwrap();

    assert!(aPCurve.is_null());
    expect_double_eq(aFirst, 0.0);
    expect_double_eq(aLast, 0.0);
}
