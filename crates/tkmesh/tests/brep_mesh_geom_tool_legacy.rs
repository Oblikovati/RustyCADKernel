//! 1:1 original BRepMesh_GeomTool_Test declaration at OCCT 3d097a03.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point2, Point3, Xy};
use rustycad_tkmesh::brep_mesh::{GeomTool, IntersectionFlag};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
#[test]
fn BRepMesh_GeomTool_Test__OCC25547_StaticMethodsExportAndFunctionality() {
    // OCC25547: static class methods must remain exported and functional.
    discretize_original_arc();
    original_surface_normal();
    original_intersections();
}
fn discretize_original_arc() {
    let first = 0.0;
    let last = std::f64::consts::PI;
    let circle: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(Axis2::new(Point3::default(), Direction3::Z).unwrap(), 10.0).unwrap(),
    )
    .cast();
    let half: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&circle, first, last, true, true).unwrap()).cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve(&half)
        .unwrap()
        .edge()
        .unwrap();
    let adaptor = BRepAdaptorCurve::with_edge(&edge).unwrap();
    let tool = GeomTool::new(&adaptor, (first, last), 0.1, 0.5, 2, precision::confusion()).unwrap();
    assert!(
        tool.nb_points() > 0,
        "BRepMesh_GeomTool failed to discretize an arc"
    );
}
fn original_surface_normal() {
    let face =
        BRepBuilderAPIMakeFace::from_plane(Plane3::new(Point3::default(), Direction3::Z).unwrap())
            .unwrap()
            .face()
            .unwrap();
    let surface = Handle::new(BRepAdaptorSurface::with_face(&face, true).unwrap());
    let mut point = Point3::default();
    let mut normal = Direction3::default();
    let computed = GeomTool::normal(&surface, 10.0, 10.0, &mut point, &mut normal).unwrap();
    assert!(
        computed,
        "BRepMesh_GeomTool failed to compute a normal of surface"
    );
}
fn original_intersections() {
    let points = [
        Xy::new(-10.0, -10.0),
        Xy::new(10.0, 10.0),
        Xy::new(-10.0, 10.0),
        Xy::new(10.0, -10.0),
    ];
    let mut intersection = Xy::default();
    let mut parameters = [0.0; 2];
    let flag = GeomTool::int_lin_lin(
        points[0],
        points[1],
        points[2],
        points[3],
        &mut intersection,
        &mut parameters,
    );
    assert_eq!(
        flag,
        IntersectionFlag::Cross,
        "BRepMesh_GeomTool::IntLinLin should return Cross flag"
    );
    let mut intersection = Point2::new(intersection.x(), intersection.y());
    assert!(
        intersection.distance(Point2::default()) <= precision::p_confusion(),
        "BRepMesh_GeomTool failed to intersect two lines at origin"
    );
    let flag = GeomTool::int_seg_seg(
        points[0],
        points[1],
        points[2],
        points[3],
        false,
        false,
        &mut intersection,
    );
    assert_eq!(
        flag,
        IntersectionFlag::Cross,
        "BRepMesh_GeomTool::IntSegSeg should return Cross flag"
    );
    assert!(
        intersection.distance(Point2::default()) <= precision::p_confusion(),
        "BRepMesh_GeomTool failed to intersect two segments at origin"
    );
}
