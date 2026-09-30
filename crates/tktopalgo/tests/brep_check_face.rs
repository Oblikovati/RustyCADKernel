//! 1:1 translation of src/ModelingAlgorithms/TKTopAlgo/GTests/BRepCheck_Face_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Face, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dOffsetCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Circle3, Direction2, Direction3, Line3, Plane3, Point2, Point3,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::{BRepCheckAnalyzer, BRepCheckFace, BRepCheckStatus};

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}

fn make_rectangle(the_x_min: f64, the_y_min: f64, the_x_max: f64, the_y_max: f64) -> Wire {
    let a_p1 = Point3::new(the_x_min, the_y_min, 0.0);
    let a_p2 = Point3::new(the_x_max, the_y_min, 0.0);
    let a_p3 = Point3::new(the_x_max, the_y_max, 0.0);
    let a_p4 = Point3::new(the_x_min, the_y_max, 0.0);

    let an_edge_builders = [
        BRepBuilderAPIMakeEdge::from_points(a_p1, a_p2).unwrap(),
        BRepBuilderAPIMakeEdge::from_points(a_p2, a_p3).unwrap(),
        BRepBuilderAPIMakeEdge::from_points(a_p3, a_p4).unwrap(),
        BRepBuilderAPIMakeEdge::from_points(a_p4, a_p1).unwrap(),
    ];
    let mut a_wire_builder = BRepBuilderAPIMakeWire::new();
    for an_edge_builder in &an_edge_builders {
        if !an_edge_builder.is_done() {
            return Wire::default();
        }
        a_wire_builder
            .add_edge(&an_edge_builder.edge().unwrap())
            .unwrap();
    }
    if a_wire_builder.is_done() {
        a_wire_builder.wire().unwrap()
    } else {
        Wire::default()
    }
}

fn make_planar_face() -> Face {
    let an_outer_wire = make_rectangle(0.0, 0.0, 100.0, 100.0);
    if an_outer_wire.is_null() {
        return Face::default();
    }

    let a_face_builder = BRepBuilderAPIMakeFace::from_plane_wire(
        Plane3::new(Point3::new(0.0, 0.0, 0.0), z()).unwrap(),
        &an_outer_wire,
        true,
    )
    .unwrap();
    if a_face_builder.is_done() {
        a_face_builder.face().unwrap()
    } else {
        Face::default()
    }
}

fn add_circular_hole(the_face: &mut Face, the_center: Point3, the_radius: f64) -> bool {
    let circle = Circle3::new(Axis2::new(the_center, z()).unwrap(), the_radius).unwrap();
    let an_edge_builder = BRepBuilderAPIMakeEdge::from_circ(circle).unwrap();
    if !an_edge_builder.is_done() {
        return false;
    }

    let a_wire_builder =
        BRepBuilderAPIMakeWire::from_edge(&an_edge_builder.edge().unwrap()).unwrap();
    if !a_wire_builder.is_done() {
        return false;
    }

    let mut a_hole = a_wire_builder.wire().unwrap();
    a_hole.reverse();
    BRepBuilder::default().add(the_face, &a_hole).unwrap();
    true
}

fn classify_wires(the_face: &Face) -> BRepCheckStatus {
    let a_checker = BRepCheckFace::new(the_face).unwrap();
    a_checker.classify_wires(false).unwrap()
}

//=================================================================================================

#[test]
fn BRepCheck_FaceTest__ClassifyWires_DisjointHolesAreValid() {
    let mut a_face = make_planar_face();
    assert!(!a_face.is_null());

    for a_row in 0..5 {
        for a_column in 0..8 {
            let a_center = Point3::new(
                10.0 + 11.0 * f64::from(a_column),
                10.0 + 11.0 * f64::from(a_row),
                0.0,
            );
            assert!(add_circular_hole(&mut a_face, a_center, 2.0));
        }
    }

    assert_eq!(classify_wires(&a_face), BRepCheckStatus::NoError);
    assert!(
        BRepCheckAnalyzer::new(&a_face, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

//=================================================================================================

#[test]
fn BRepCheck_FaceTest__ClassifyWires_OffsetPCurve() {
    let a_face = make_planar_face();
    assert!(!a_face.is_null());

    let a_center2d = Point2::new(50.0, 50.0);
    let a_basis: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dCircle::from_axis(
            Axis2d::new(a_center2d, Direction2::new(1.0, 0.0).unwrap()),
            4.0,
            true,
        )
        .unwrap(),
    )
    .cast();
    let an_offset: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&a_basis, 1.0, false).unwrap()).cast();
    let a_radius = an_offset
        .get()
        .unwrap()
        .value(0.0)
        .unwrap()
        .distance(a_center2d);

    let circle = Circle3::new(
        Axis2::new(Point3::new(50.0, 50.0, 0.0), z()).unwrap(),
        a_radius,
    )
    .unwrap();
    let an_edge_builder = BRepBuilderAPIMakeEdge::from_circ(circle).unwrap();
    assert!(an_edge_builder.is_done());

    let an_edge = an_edge_builder.edge().unwrap();
    BRepBuilder::default()
        .update_edge_pcurve_on_face(&an_edge, &an_offset, &a_face, precision::confusion())
        .unwrap();
    let a_wire_builder = BRepBuilderAPIMakeWire::from_edge(&an_edge).unwrap();
    assert!(a_wire_builder.is_done());

    let mut a_hole = a_wire_builder.wire().unwrap();
    a_hole.reverse();
    let mut a_face = a_face;
    BRepBuilder::default().add(&mut a_face, &a_hole).unwrap();

    assert_eq!(classify_wires(&a_face), BRepCheckStatus::NoError);
}

//=================================================================================================

#[test]
fn BRepCheck_FaceTest__ClassifyWires_OpenWireFallbackIsOrientationIndependent() {
    let mut a_face = make_planar_face();
    assert!(!a_face.is_null());

    let an_edge_builder = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(120.0, 10.0, 0.0),
        Point3::new(130.0, 10.0, 0.0),
    )
    .unwrap();
    assert!(an_edge_builder.is_done());
    let a_wire_builder =
        BRepBuilderAPIMakeWire::from_edge(&an_edge_builder.edge().unwrap()).unwrap();
    assert!(a_wire_builder.is_done());
    BRepBuilder::default()
        .add(&mut a_face, &a_wire_builder.wire().unwrap())
        .unwrap();

    let a_forward_status = classify_wires(&a_face);
    let a_reversed_face = Face::unchecked(a_face.reversed());
    assert_eq!(a_forward_status, BRepCheckStatus::NoError);
    assert_eq!(classify_wires(&a_reversed_face), a_forward_status);
}

//=================================================================================================

#[test]
fn BRepCheck_FaceTest__ClassifyWires_UnboundedWireFallbackIsOrientationIndependent() {
    let mut a_face = make_planar_face();
    assert!(!a_face.is_null());

    let a_line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(Line3::new(
        Point3::new(0.0, 120.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )))
    .cast();
    let an_edge_builder = BRepBuilderAPIMakeEdge::from_curve(&a_line).unwrap();
    assert!(an_edge_builder.is_done());
    let a_wire_builder =
        BRepBuilderAPIMakeWire::from_edge(&an_edge_builder.edge().unwrap()).unwrap();
    assert!(a_wire_builder.is_done());
    BRepBuilder::default()
        .add(&mut a_face, &a_wire_builder.wire().unwrap())
        .unwrap();

    let a_forward_status = classify_wires(&a_face);
    let a_reversed_face = Face::unchecked(a_face.reversed());
    assert_eq!(a_forward_status, BRepCheckStatus::NoError);
    assert_eq!(classify_wires(&a_reversed_face), a_forward_status);
}
