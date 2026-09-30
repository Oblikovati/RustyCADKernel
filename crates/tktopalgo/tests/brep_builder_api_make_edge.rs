//! 1:1 translation of src/ModelingAlgorithms/TKTopAlgo/GTests/BRepBuilderAPI_MakeEdge_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

/// `BRepGProp::LinearProperties(anEdge, aProps)` (default flags), then `Mass()`.
fn edge_length(edge: &rustycad_tkbrep::topods::Edge) -> f64 {
    let mut a_props = GProps::new();
    BRepGProp::linear_properties(edge, &mut a_props, false, false).unwrap();
    a_props.mass()
}

/// `new Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 5.0)`.
fn circle_of_radius_5() -> Handle<dyn GeomCurve> {
    let an_axis = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Handle::new(GeomCircle::new(an_axis, 5.0).unwrap()).cast()
}

#[test]
fn BRepBuilderAPI_MakeEdgeTest__LinearEdge_TwoPoints() {
    let a_p1 = Point3::new(0.0, 0.0, 0.0);
    let a_p2 = Point3::new(10.0, 0.0, 0.0);

    let a_make_edge = BRepBuilderAPIMakeEdge::from_points(a_p1, a_p2).unwrap();
    assert!(a_make_edge.is_done(), "Linear edge creation failed");

    let an_edge = a_make_edge.edge().unwrap();
    assert!(!an_edge.is_null(), "Resulting edge is null");

    // Verify edge length
    let mass = edge_length(&an_edge);
    assert!(
        (mass - 10.0).abs() <= precision::confusion(),
        "Edge length should be 10"
    );
}

#[test]
fn BRepBuilderAPI_MakeEdgeTest__CircularEdge_Full() {
    let a_circle = circle_of_radius_5();

    let a_make_edge = BRepBuilderAPIMakeEdge::from_curve(&a_circle).unwrap();
    assert!(a_make_edge.is_done(), "Full circular edge creation failed");

    let an_edge = a_make_edge.edge().unwrap();
    assert!(!an_edge.is_null());

    // Full circle length = 2*PI*R
    let mass = edge_length(&an_edge);
    assert!((mass - 2.0 * PI * 5.0).abs() <= precision::confusion());
}

#[test]
fn BRepBuilderAPI_MakeEdgeTest__CircularEdge_Trimmed() {
    let a_circle = circle_of_radius_5();

    // Half circle: 0 to PI
    let a_make_edge = BRepBuilderAPIMakeEdge::from_curve_params(&a_circle, 0.0, PI).unwrap();
    assert!(
        a_make_edge.is_done(),
        "Trimmed circular edge creation failed"
    );

    let an_edge = a_make_edge.edge().unwrap();
    assert!(!an_edge.is_null());

    // Half circle length = PI*R
    let mass = edge_length(&an_edge);
    assert!((mass - PI * 5.0).abs() <= precision::confusion());
}

#[test]
fn BRepBuilderAPI_MakeEdgeTest__EdgeFromLine_WithBounds() {
    let a_line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    ))
    .cast();

    let a_make_edge = BRepBuilderAPIMakeEdge::from_curve_params(&a_line, 0.0, 7.0).unwrap();
    assert!(a_make_edge.is_done(), "Edge from line with bounds failed");

    let an_edge = a_make_edge.edge().unwrap();
    assert!(!an_edge.is_null());

    let mass = edge_length(&an_edge);
    assert!((mass - 7.0).abs() <= precision::confusion());
}

#[test]
fn BRepBuilderAPI_MakeEdgeTest__VertexExtraction() {
    let a_p1 = Point3::new(0.0, 0.0, 0.0);
    let a_p2 = Point3::new(10.0, 0.0, 0.0);

    let a_make_edge = BRepBuilderAPIMakeEdge::from_points(a_p1, a_p2).unwrap();
    assert!(a_make_edge.is_done());

    let an_edge = a_make_edge.edge().unwrap();
    let (a_v1, a_v2) = topexp::vertices(&an_edge, false);

    assert!(!a_v1.is_null(), "First vertex is null");
    assert!(!a_v2.is_null(), "Last vertex is null");

    let a_first_pnt = BRepTool::pnt(&a_v1).unwrap();
    let a_last_pnt = BRepTool::pnt(&a_v2).unwrap();

    assert!(a_first_pnt.is_equal(&a_p1, precision::confusion()));
    assert!(a_last_pnt.is_equal(&a_p2, precision::confusion()));
}

#[test]
fn BRepBuilderAPI_MakeEdgeTest__ToleranceCheck() {
    let a_p1 = Point3::new(0.0, 0.0, 0.0);
    let a_p2 = Point3::new(10.0, 0.0, 0.0);

    let a_make_edge = BRepBuilderAPIMakeEdge::from_points(a_p1, a_p2).unwrap();
    assert!(a_make_edge.is_done());

    let an_edge = a_make_edge.edge().unwrap();
    let a_tol = BRepTool::edge_tolerance(&an_edge).unwrap();
    assert!(a_tol > 0.0, "Edge tolerance should be positive");
    assert!(
        a_tol <= precision::confusion(),
        "Edge tolerance should not exceed confusion"
    );
}
