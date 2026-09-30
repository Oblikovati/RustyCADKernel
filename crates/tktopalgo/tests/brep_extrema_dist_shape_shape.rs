//! 1:1 translation of src/ModelingAlgorithms/TKTopAlgo/GTests/BRepExtrema_DistShapeShape_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use rustycad_tktopalgo::brep_extrema::BRepExtremaDistShapeShape;

/// `BRepExtrema_DistShapeShape(Shape1, Shape2, theDeflection)` with the default flag,
/// algorithm and progress range.
fn dist_shape_shape(
    shape1: &rustycad_tkbrep::topods::Shape,
    shape2: &rustycad_tkbrep::topods::Shape,
    deflection: f64,
) -> BRepExtremaDistShapeShape {
    let options = (ExtFlag::MinMax, ExtAlgo::Grad);
    BRepExtremaDistShapeShape::with_deflection(
        shape1,
        shape2,
        deflection,
        options,
        &ProgressRange::default(),
    )
    .unwrap()
}

// Bug BUC60870: BRepExtrema_DistShapeShape should correctly compute the minimum distance
// between a linear edge and a vertex.
// The edge runs from (0,0,0) to (0,1,0); the vertex is at (0,0.3,1).
// The theoretical minimum distance is 1.0 (perpendicular from (0,0.3,1) to (0,0.3,0)).
#[test]
fn BRepExtrema_DistShapeShapeTest__BUC60870_EdgeToVertexMinimumDistance() {
    let an_edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0))
            .unwrap()
            .edge()
            .unwrap();
    let a_vertex = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 0.3, 1.0))
        .unwrap()
        .vertex()
        .unwrap();

    let a_dist = dist_shape_shape(&an_edge, &a_vertex, 2.0);

    assert!(
        a_dist.is_done(),
        "BRepExtrema_DistShapeShape failed to compute"
    );
    assert!(a_dist.nb_solution() >= 1, "Expected at least one solution");

    let a_theor_dist = 1.0;
    let a_percent_max = 0.01; // 1% relative tolerance
    assert!(
        (a_dist.value().unwrap() - a_theor_dist).abs() <= a_theor_dist * a_percent_max,
        "Minimum distance deviates from expected value 1.0"
    );
}

// Verify that BRepExtrema_DistShapeShape does not crash when computing the distance
// between two edges and one of them has no 3D curve (null Geom_Curve from BRep_Tool::Curve).
// This exercises the null-check in the PERFORM_C0 helper.
#[test]
fn BRepExtrema_DistShapeShapeTest__EdgeEdge_Null3DCurve_NoCrash() {
    let a_edge1 =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
            .unwrap()
            .edge()
            .unwrap();

    let a_circle: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(
            Axis2::new(
                Point3::new(5.0, 0.0, 0.0),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
            )
            .unwrap(),
            1.0,
        )
        .unwrap(),
    )
    .cast();
    let a_edge2 = BRepBuilderAPIMakeEdge::from_curve(&a_circle)
        .unwrap()
        .edge()
        .unwrap();

    let a_builder = BRepBuilder::default();
    a_builder
        .update_edge_curve(&a_edge2, &Handle::default(), 1e-7)
        .unwrap();

    let a_dist = dist_shape_shape(&a_edge1, &a_edge2, 10.0);
    let _ = a_dist;
}

// Verify that BRepExtrema_DistShapeShape does not crash when computing the distance
// between an edge with no 3D curve and a face.
// This exercises the null-check in Perform(Edge, Face).
#[test]
fn BRepExtrema_DistShapeShapeTest__EdgeFace_Null3DCurve_NoCrash() {
    let a_circle: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(
            Axis2::new(
                Point3::new(5.0, 0.0, 0.0),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
            )
            .unwrap(),
            1.0,
        )
        .unwrap(),
    )
    .cast();
    let a_edge = BRepBuilderAPIMakeEdge::from_curve(&a_circle)
        .unwrap()
        .edge()
        .unwrap();

    let a_builder = BRepBuilder::default();
    a_builder
        .update_edge_curve(&a_edge, &Handle::default(), 1e-7)
        .unwrap();

    let a_face = BRepBuilderAPIMakeFace::from_plane_bounds(
        Plane3::new(
            Point3::new(0.0, 0.0, 5.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
        .unwrap(),
        (-10.0, 10.0, -10.0, 10.0),
    )
    .unwrap()
    .face()
    .unwrap();

    let a_dist = dist_shape_shape(&a_edge, &a_face, 10.0);
    let _ = a_dist;
}
