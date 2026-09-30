//! Behaviour tests with the cases of
//! src/ModelingAlgorithms/TKTopAlgo/GTests/BRepExtrema_ShapeProximity_Test.cxx.
//!
//! The upstream suite meshes its shapes with BRepMesh_IncrementalMesh of TKMesh, which
//! the port does not provide (and the shared native build does not hold), so it is not
//! mapped. Each case builds the same shapes with BRepBuilderAPI, meshes them with the
//! grid and tangential-deflection stand-in of `brep_extrema_support`, and checks the same
//! properties (the proximity value within the tolerance of the case, the statuses where
//! the case compares them, distinct proximity points).
#![allow(non_snake_case)]

mod brep_extrema_support;

use brep_extrema_support::shapes::{
    dir, make_bezier_edge, make_circular_arc, make_cylinder_patch, make_full_circular_edge,
    make_plane_patch, make_polygon_face, make_sphere_face, make_sphere_patch, make_split_sphere,
    make_trimmed_circular_arc,
};
use brep_extrema_support::{BREPMESH_LIKE, Density, FINE, MEDIUM, mesh_shape};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_extrema::{
    BRepExtremaDistShapeShape, BRepExtremaProxPntStatus as ProximityStatus,
    BRepExtremaShapeProximity,
};
use std::f64::consts::PI;

/// `expectProximityValue`: meshes both shapes, computes the proximity with an infinite
/// tolerance and checks the value, the statuses (when asked) and distinct points.
fn expect_proximity_value(
    (shape1, shape2): (&Shape, &Shape),
    expected_value: f64,
    tolerance: f64,
    (expected_status1, expected_status2): (ProximityStatus, ProximityStatus),
    (mesh_deflection, density): (f64, Density),
    check_statuses: bool,
) {
    mesh_shape(shape1, mesh_deflection, density).unwrap();
    mesh_shape(shape2, mesh_deflection, density).unwrap();

    let mut a_proximity = BRepExtremaShapeProximity::new(precision::infinite());
    assert!(a_proximity.load_shape1(shape1).unwrap());
    assert!(a_proximity.load_shape2(shape2).unwrap());
    a_proximity.perform().unwrap();

    assert!(a_proximity.is_done());
    assert!(
        (a_proximity.proximity() - expected_value).abs() <= tolerance,
        "proximity {} expected {expected_value} +- {tolerance}",
        a_proximity.proximity()
    );
    if check_statuses {
        assert_eq!(a_proximity.prox_pnt_status1(), expected_status1);
        assert_eq!(a_proximity.prox_pnt_status2(), expected_status2);
    }
    assert!(
        !a_proximity
            .proximity_point1()
            .is_equal(a_proximity.proximity_point2(), precision::confusion())
    );
}

const MIDDLE: ProximityStatus = ProximityStatus::Middle;
const BORDER: ProximityStatus = ProximityStatus::Border;
const ORIGIN: Point3 = Point3::new(0.0, 0.0, 0.0);

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_A1_OppositeCircularArcs() {
    let an_edge1 = make_circular_arc(ORIGIN, dir(1.0, 0.0, 0.0), 5.0, 0.5 * PI, PI);
    let an_edge2 = make_circular_arc(ORIGIN, dir(1.0, 0.0, 0.0), 5.0, -0.5 * PI, 0.0);
    expect_proximity_value(
        (&an_edge1, &an_edge2),
        10.0,
        1.0e-3,
        (MIDDLE, MIDDLE),
        (1.0e-3, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_B1_ParallelPlanePatches() {
    let a_face1 = make_plane_patch(ORIGIN, dir(0.0, 0.0, 1.0), (-1.0, 1.0, -1.0, 1.0));
    let a_face2 = make_plane_patch(
        Point3::new(0.0, 0.0, 1.0),
        dir(0.0, 0.0, 1.0),
        (-1.0, 1.0, -1.0, 1.0),
    );
    expect_proximity_value(
        (&a_face1, &a_face2),
        1.0,
        1.0e-3,
        (BORDER, BORDER),
        (0.1, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_C1_PlaneAndCylinder() {
    let a_plane = make_plane_patch(ORIGIN, dir(0.0, 0.0, 1.0), (-1.0, 1.0, -1.0, 1.0));
    let a_cylinder =
        make_cylinder_patch(ORIGIN, dir(0.0, 1.0, 0.0), 0.1, (0.0, 2.0 * PI), (0.0, 1.0));
    expect_proximity_value(
        (&a_plane, &a_cylinder),
        0.1,
        1.0e-3,
        (MIDDLE, BORDER),
        (1.0e-3, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_D1_OpposingSpherePatches() {
    let a_sphere1 = make_sphere_patch(Point3::new(0.0, 1.0, 0.0), 0.0, PI);
    let a_sphere2 = make_sphere_patch(Point3::new(0.0, -1.0, 0.0), -PI, 0.0);
    expect_proximity_value(
        (&a_sphere1, &a_sphere2),
        4.0,
        1.0e-2,
        (MIDDLE, MIDDLE),
        (1.0e-3, MEDIUM),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_A2_PerpendicularFullCircles() {
    let an_edge1 = make_full_circular_edge(ORIGIN, dir(1.0, 0.0, 0.0), 1.0);
    let an_edge2 = make_full_circular_edge(ORIGIN, dir(0.0, 1.0, 0.0), 1.0);
    expect_proximity_value(
        (&an_edge1, &an_edge2),
        2.0,
        1.0e-3,
        (MIDDLE, MIDDLE),
        (1.0e-3, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_A3_OffsetSemicircles() {
    let an_edge1 = make_trimmed_circular_arc(
        Point3::new(0.0, -1.0, 0.0),
        dir(1.0, 0.0, 0.0),
        1.0,
        -PI,
        0.0,
    );
    let an_edge2 =
        make_trimmed_circular_arc(Point3::new(0.0, 1.0, 0.0), dir(1.0, 0.0, 0.0), 1.0, 0.0, PI);
    let expected = 5.0_f64.sqrt() - 1.0;
    expect_proximity_value(
        (&an_edge1, &an_edge2),
        expected,
        1.0e-3,
        (MIDDLE, BORDER),
        (1.0e-3, FINE),
        false,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_A4_CoplanarCirclesWithDifferentAxes() {
    let an_edge1 = make_full_circular_edge(ORIGIN, dir(1.0, 0.0, 0.0), 1.0);
    let an_edge2 = make_full_circular_edge(ORIGIN, dir(-1.0, 0.5, 0.0), 1.0);
    expect_proximity_value(
        (&an_edge1, &an_edge2),
        2.0,
        1.0e-3,
        (MIDDLE, MIDDLE),
        (1.0e-3, FINE),
        false,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_A5_TrimmedCircularEdges() {
    let an_edge1 = make_trimmed_circular_arc(ORIGIN, dir(1.0, 0.0, 0.0), 1.0, -0.5 * PI, 0.5 * PI);
    let an_edge2 = make_trimmed_circular_arc(ORIGIN, dir(1.0, 0.5, 0.0), 1.0, 0.0, PI);
    let first1 = topexp::first_vertex(&rustycad_tkbrep::topods::edge(&an_edge1).unwrap(), false);
    let first2 = topexp::first_vertex(&rustycad_tkbrep::topods::edge(&an_edge2).unwrap(), false);
    assert!(!first1.is_null());
    assert!(!first2.is_null());
    let reference =
        BRepExtremaDistShapeShape::with_shapes(&first1, &first2, Default::default()).unwrap();
    assert!(reference.is_done());
    assert!(reference.value().unwrap() > 0.0);
    let expected = reference.value().unwrap();
    expect_proximity_value(
        (&an_edge1, &an_edge2),
        expected,
        2.0e-3,
        (MIDDLE, BORDER),
        (1.0e-3, FINE),
        false,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_B2_OrthogonalPlanePatches() {
    let a_face1 = make_plane_patch(ORIGIN, dir(0.0, 0.0, 1.0), (-1.0, 1.0, -1.0, 1.0));
    let a_face2 = make_plane_patch(
        Point3::new(0.0, 0.0, 1.0),
        dir(-1.0, 0.0, 0.0),
        (-1.0, 1.0, -1.0, 1.0),
    );
    expect_proximity_value(
        (&a_face1, &a_face2),
        1.0,
        1.0e-3,
        (BORDER, BORDER),
        (0.1, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_B3_IntersectingPlanePatches() {
    let a_face1 = make_plane_patch(ORIGIN, dir(0.0, 0.0, 1.0), (-1.0, 1.0, -1.0, 1.0));
    let a_face2 = make_plane_patch(ORIGIN, dir(-1.0, 0.0, 0.0), (-1.0, 1.0, -1.0, 1.0));
    expect_proximity_value(
        (&a_face1, &a_face2),
        1.0,
        1.0e-3,
        (BORDER, BORDER),
        (0.1, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_B4_OffsetIntersectingPlanePatches() {
    let a_face1 = make_plane_patch(ORIGIN, dir(0.0, 0.0, 1.0), (-1.0, 1.0, -1.0, 1.0));
    let a_face2 = make_plane_patch(
        Point3::new(0.0, 0.0, 0.5),
        dir(-1.0, 0.0, 0.0),
        (-1.0, 1.0, -1.0, 1.0),
    );
    expect_proximity_value(
        (&a_face1, &a_face2),
        1.0,
        1.0e-3,
        (BORDER, BORDER),
        (0.1, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_B5_SlopedPolygonPatch() {
    let a_face1 = make_plane_patch(ORIGIN, dir(0.0, 0.0, 1.0), (-1.0, 1.0, -1.0, 1.0));
    let a_face2 = make_polygon_face();
    let expected = 0.8 * 5.0_f64.sqrt();
    expect_proximity_value(
        (&a_face1, &a_face2),
        expected,
        1.0e-3,
        (BORDER, BORDER),
        (1.0e-3, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_C2_ObliqueCylinderAndPlane() {
    let a_plane = make_plane_patch(ORIGIN, dir(0.0, 0.0, 1.0), (-2.0, 2.0, -2.0, 2.0));
    let axis = dir(0.0, 0.5_f64.sqrt(), 0.5_f64.sqrt());
    let a_cylinder = make_cylinder_patch(
        Point3::new(0.0, 0.0, 1.0),
        axis,
        0.1,
        (0.0, 2.0 * PI),
        (0.0, 1.0),
    );
    let expected = 1.0 + 1.1 * 0.5_f64.sqrt();
    expect_proximity_value(
        (&a_plane, &a_cylinder),
        expected,
        1.0e-3,
        (MIDDLE, BORDER),
        (1.0e-3, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_C3_ParallelCylinderPatches() {
    let a_cylinder1 = make_cylinder_patch(
        Point3::new(0.0, 0.0, 1.0),
        dir(1.0, 0.0, 0.0),
        0.1,
        (0.0, 2.0 * PI),
        (0.0, 1.0),
    );
    let a_cylinder2 =
        make_cylinder_patch(ORIGIN, dir(1.0, 0.0, 0.0), 0.1, (0.0, 2.0 * PI), (0.0, 1.0));
    // The BRepMesh density: the finer grid holds facets of the second cylinder whose
    // perpendicular reaches the far side of the first one (a proximity of 1.19).
    let density = BREPMESH_LIKE;
    expect_proximity_value(
        (&a_cylinder1, &a_cylinder2),
        1.0,
        1.0e-3,
        (BORDER, BORDER),
        (1.0e-3, density),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_C4_TrimmedParallelCylinderPatches() {
    let a_cylinder1 = make_cylinder_patch(
        Point3::new(0.0, 0.0, 1.0),
        dir(1.0, 0.0, 0.0),
        0.1,
        (-0.5 * PI, 0.5 * PI),
        (0.0, 1.0),
    );
    let a_cylinder2 = make_cylinder_patch(
        ORIGIN,
        dir(1.0, 0.0, 0.0),
        0.1,
        (0.5 * PI, -0.5 * PI),
        (0.0, 1.0),
    );
    expect_proximity_value(
        (&a_cylinder1, &a_cylinder2),
        1.2,
        1.0e-3,
        (BORDER, BORDER),
        (1.0e-5, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_D2_FullOpposingSpheres() {
    let a_sphere1 = make_sphere_face(Point3::new(0.0, 1.0, 0.0));
    let a_sphere2 = make_sphere_face(Point3::new(0.0, -1.0, 0.0));
    expect_proximity_value(
        (&a_sphere1, &a_sphere2),
        4.0,
        1.0e-2,
        (MIDDLE, MIDDLE),
        (1.0e-3, MEDIUM),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_D3_SplitSphereAndPlane() {
    let a_sphere = make_split_sphere(Point3::new(0.0, 1.0, 0.0));
    let a_plane = make_plane_patch(ORIGIN, dir(0.0, 1.0, 0.0), (-2.0, 2.0, -2.0, 2.0));
    expect_proximity_value(
        (&a_sphere, &a_plane),
        2.0,
        1.0e-2,
        (BORDER, MIDDLE),
        (1.0e-3, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_D4_TrimmedSphereAndPlane() {
    let a_sphere = make_sphere_patch(Point3::new(0.0, 1.0, 0.0), 0.0, PI);
    let a_plane = make_plane_patch(ORIGIN, dir(0.0, 1.0, 0.0), (-1.0, 1.0, -1.0, 1.0));
    expect_proximity_value(
        (&a_sphere, &a_plane),
        2.0,
        1.0e-2,
        (MIDDLE, MIDDLE),
        (1.0e-3, FINE),
        true,
    );
}

const POLES_E1: [[f64; 3]; 7] = [
    [10.0, 84.0, 0.0],
    [54.0, 96.0, 0.0],
    [145.0, 146.0, 0.0],
    [167.0, 167.0, 0.0],
    [185.0, 212.0, 0.0],
    [187.0, 234.0, 0.0],
    [176.0, 302.0, 0.0],
];
const POLES_E2: [[f64; 3]; 8] = [
    [120.0, 72.0, 0.0],
    [170.0, 87.0, 0.0],
    [227.0, 118.0, 0.0],
    [238.0, 126.0, 0.0],
    [243.0, 157.0, 0.0],
    [203.0, 216.0, 0.0],
    [134.0, 281.0, 0.0],
    [94.0, 324.0, 0.0],
];

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_E1_BezierEdgesEndpointReference() {
    let an_edge1 = make_bezier_edge(&POLES_E1);
    let an_edge2 = make_bezier_edge(&POLES_E2);
    let last = |shape: &Shape| {
        let vertex = topexp::last_vertex(&rustycad_tkbrep::topods::edge(shape).unwrap(), true);
        BRepTool::pnt(&vertex).unwrap()
    };
    let expected = last(&an_edge1).distance(&last(&an_edge2));
    expect_proximity_value(
        (&an_edge1, &an_edge2),
        expected,
        1.0e-3,
        (BORDER, BORDER),
        (1.0e-3, FINE),
        true,
    );
}

#[test]
fn BRepExtrema_ShapeProximityTest__LowAlgo_E2_DifferentDegreeBezierEdges() {
    let an_edge1 = make_bezier_edge(&POLES_E1[1..]);
    let an_edge2 = make_bezier_edge(&POLES_E2[..7]);
    expect_proximity_value(
        (&an_edge1, &an_edge2),
        76.34,
        1.0e-3,
        (MIDDLE, MIDDLE),
        (1.0e-3, FINE),
        true,
    );
}
