//! 1:1 translation of src/ModelingAlgorithms/TKTopAlgo/GTests/BRepTopAdaptor_FClass2d_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Wire};
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::cslib::{Class2d, Class2dResult};
use rustycad_tkmath::gp::{self, Axis3, Plane3, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_class::BRepClassFaceClassifier;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use std::f64::consts::PI;

fn make_rectangle(the_x_min: f64, the_y_min: f64, the_x_max: f64, the_y_max: f64) -> Wire {
    let a_points = [
        Point3::new(the_x_min, the_y_min, 0.0),
        Point3::new(the_x_max, the_y_min, 0.0),
        Point3::new(the_x_max, the_y_max, 0.0),
        Point3::new(the_x_min, the_y_max, 0.0),
    ];
    let mut a_wire_builder = BRepBuilderAPIMakeWire::new();
    for an_index in 0..4 {
        let an_edge_builder =
            BRepBuilderAPIMakeEdge::from_points(a_points[an_index], a_points[(an_index + 1) % 4])
                .unwrap();
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

/// `gp_Pln(gp::XOY())`.
fn xoy_plane() -> Plane3 {
    Plane3::from(Axis3::from_axis2(gp::xoy()))
}

fn make_face_with_hole() -> Face {
    let an_inner_wire = make_rectangle(3.0, 2.0, 9.0, 6.0);
    if an_inner_wire.is_null() {
        return Face::default();
    }

    let mut a_face_builder =
        BRepBuilderAPIMakeFace::from_plane_bounds(xoy_plane(), (0.0, 12.0, 0.0, 8.0)).unwrap();
    if !a_face_builder.is_done() {
        return Face::default();
    }
    a_face_builder
        .add(&topods::wire(&an_inner_wire.reversed()).unwrap())
        .unwrap();
    if a_face_builder.is_done() {
        a_face_builder.face().unwrap()
    } else {
        Face::default()
    }
}

fn make_face_with_disconnected_wire() -> Face {
    let a_face_builder =
        BRepBuilderAPIMakeFace::from_plane_bounds(xoy_plane(), (0.0, 10.0, 0.0, 10.0)).unwrap();
    if !a_face_builder.is_done() {
        return Face::default();
    }

    let mut a_face = a_face_builder.face().unwrap();
    let mut a_first_edge = Edge::default();
    let mut a_third_edge = Edge::default();
    let mut an_edge_index = 0;
    let mut an_explorer = Explorer::new(&a_face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    while an_explorer.more() {
        if an_edge_index == 0 {
            a_first_edge = topods::edge(an_explorer.current()).unwrap();
        } else if an_edge_index == 2 {
            a_third_edge = topods::edge(an_explorer.current()).unwrap();
        }
        an_edge_index += 1;
        an_explorer.next().unwrap();
    }
    if a_first_edge.is_null() || a_third_edge.is_null() {
        return Face::default();
    }

    let a_builder = BRepBuilder::default();
    let mut a_disconnected_wire = Wire::default();
    a_builder.make_wire(&mut a_disconnected_wire);
    a_builder
        .add(&mut a_disconnected_wire, &a_first_edge)
        .unwrap();
    a_builder
        .add(&mut a_disconnected_wire, &a_third_edge)
        .unwrap();
    a_builder.add(&mut a_face, &a_disconnected_wire).unwrap();
    a_face
}

/// `BRepClass_FaceClassifier(F, P, Tol)` (the native defaults useBndBox = false and
/// theGapCheckTol = 0.1).
fn exact_classifier(face: &Face, point: Point2) -> BRepClassFaceClassifier {
    BRepClassFaceClassifier::with_face_point2d(face, &point, precision::p_confusion(), false, 0.1)
        .unwrap()
}

#[test]
fn BRepTopAdaptorFClass2dTest__LinearVectorView_ZeroBasedOrderIsClassifiedCorrectly() {
    let mut a_points: LinearVector<Point2> = LinearVector::default();
    a_points.append(Point2::new(0.0, 0.0));
    a_points.append(Point2::new(7.0, 0.0));
    a_points.append(Point2::new(7.0, 3.0));
    a_points.append(Point2::new(0.0, 3.0));
    a_points.append(Point2::new(0.0, 0.0));

    let a_view = a_points.to_array1_view();
    assert_eq!(a_view.lower(), 0);
    assert_eq!(a_view.upper(), 4);
    assert!(
        a_view
            .value(0)
            .unwrap()
            .is_equal(a_points[0], precision::p_confusion())
    );
    assert!(
        a_view
            .value(3)
            .unwrap()
            .is_equal(a_points[3], precision::p_confusion())
    );

    let a_classifier = Class2d::from_array(
        &a_points.to_array1_view(),
        1.0e-7,
        1.0e-7,
        0.0,
        0.0,
        7.0,
        3.0,
    );
    assert_eq!(
        a_classifier.si_dans(Point2::new(2.0, 1.0)),
        Class2dResult::Inside
    );
    assert_eq!(
        a_classifier.si_dans(Point2::new(8.0, 1.0)),
        Class2dResult::Outside
    );
}

#[test]
fn BRepTopAdaptorFClass2dTest__PlanarHole_PerformAndRestrictionPreserveWireRoles() {
    let a_face = make_face_with_hole();
    assert!(!a_face.is_null());
    let a_classifier = BRepTopAdaptorFClass2d::new(&a_face, precision::p_confusion()).unwrap();

    let a_material_point = Point2::new(1.0, 1.0);
    let a_hole_point = Point2::new(5.0, 4.0);
    let an_outside_point = Point2::new(14.0, 4.0);
    assert_eq!(
        a_classifier.perform(a_material_point, true).unwrap(),
        State::In
    );
    assert_eq!(
        a_classifier.perform(a_hole_point, true).unwrap(),
        State::Out
    );
    assert_eq!(
        a_classifier.perform(an_outside_point, true).unwrap(),
        State::Out
    );
    assert_eq!(
        a_classifier
            .test_on_restriction(a_material_point, 1.0e-7, true)
            .unwrap(),
        State::In
    );
    assert_eq!(
        a_classifier
            .test_on_restriction(a_hole_point, 1.0e-7, true)
            .unwrap(),
        State::Out
    );
    assert_eq!(
        a_classifier
            .test_on_restriction(an_outside_point, 1.0e-7, true)
            .unwrap(),
        State::Out
    );
}

#[test]
fn BRepTopAdaptorFClass2dTest__Boundary_PerformUsesRepeatableExactFallback() {
    let a_face = make_face_with_hole();
    assert!(!a_face.is_null());
    let a_classifier = BRepTopAdaptorFClass2d::new(&a_face, precision::p_confusion()).unwrap();
    let a_boundary_point = Point2::new(0.0, 3.0);

    let an_exact = exact_classifier(&a_face, a_boundary_point);
    assert_eq!(an_exact.state(), State::On);
    for _an_iteration in 0..20 {
        assert_eq!(
            a_classifier.perform(a_boundary_point, true).unwrap(),
            an_exact.state()
        );
        assert_eq!(
            a_classifier
                .test_on_restriction(a_boundary_point, 1.0e-6, true)
                .unwrap(),
            State::On
        );
    }
}

#[test]
fn BRepTopAdaptorFClass2dTest__PeriodicCylinder_RecadresEquivalentParameters() {
    let a_surface: Handle<dyn GeomSurface> = Handle::new(
        GeomCylindricalSurface::new(Axis3::from_normal(gp::origin(), gp::dz()).unwrap(), 2.0)
            .unwrap(),
    )
    .cast();
    assert!(!a_surface.is_null());
    let a_face_builder = BRepBuilderAPIMakeFace::from_surface_bounds(
        &a_surface,
        (0.0, 2.0 * PI, 0.0, 5.0),
        precision::confusion(),
    )
    .unwrap();
    assert!(a_face_builder.is_done());
    let a_classifier =
        BRepTopAdaptorFClass2d::new(&a_face_builder.face().unwrap(), precision::p_confusion())
            .unwrap();

    let a_canonical_point = Point2::new(0.75, 2.0);
    let a_shifted_point = Point2::new(a_canonical_point.x() + 4.0 * PI, a_canonical_point.y());
    assert_eq!(
        a_classifier.perform(a_canonical_point, true).unwrap(),
        State::In
    );
    assert_eq!(
        a_classifier.perform(a_shifted_point, true).unwrap(),
        a_classifier.perform(a_canonical_point, true).unwrap()
    );
    assert_eq!(
        a_classifier
            .test_on_restriction(a_shifted_point, 1.0e-7, true)
            .unwrap(),
        a_classifier
            .test_on_restriction(a_canonical_point, 1.0e-7, true)
            .unwrap()
    );
}

#[test]
fn BRepTopAdaptorFClass2dTest__DisconnectedWire_FallsBackToStableExactClassification() {
    let a_face = make_face_with_disconnected_wire();
    assert!(!a_face.is_null());
    let a_point = Point2::new(5.0, 5.0);
    let an_exact = exact_classifier(&a_face, a_point);
    let a_classifier = BRepTopAdaptorFClass2d::new(&a_face, precision::p_confusion()).unwrap();

    let an_expected = an_exact.state();
    for _an_iteration in 0..10 {
        assert_eq!(a_classifier.perform(a_point, false).unwrap(), an_expected);
        assert_eq!(
            a_classifier
                .test_on_restriction(a_point, precision::p_confusion(), false)
                .unwrap(),
            an_expected
        );
    }
}
