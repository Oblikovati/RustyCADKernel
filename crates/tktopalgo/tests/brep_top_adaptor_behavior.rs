//! Behavior tests of BRepTopAdaptor_FClass2d after the cases of the upstream suite
//! `BRepTopAdaptor_FClass2d_Test.cxx`, which is not mapped: its faces come from
//! BRepBuilderAPI_MakeEdge/MakeWire/MakeFace (TKTopAlgo BRepBuilderAPI), which the port
//! does not provide yet. The unchanged suite passes natively (5 of 5). Here the same
//! faces are built with BRep_Builder by the probe's face reader (edges with 3d lines and
//! pcurves, as MakeEdge and MakeFace give on a plane), and each case checks the same
//! properties with the same inputs.
mod adaptor_records;
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;

use adaptor_records::{CYLINDER, DISCONNECTED, HOLED};

use face::{TpFace, tp_face};
use print::Reader;
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::cslib::{Class2d, Class2dResult};
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_class::BRepClassFaceClassifier;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use std::f64::consts::PI;

fn face(record: &[f64]) -> TpFace {
    tp_face(&mut Reader::new(record)).expect("face record")
}

/// `BRepClass_FaceClassifier(F, P, Tol)`: the exact state of `point`.
fn exact_state(face: &TpFace, point: Point2) -> State {
    BRepClassFaceClassifier::with_face_point2d(
        &face.face,
        &point,
        precision::p_confusion(),
        false,
        0.1,
    )
    .unwrap()
    .state()
}

#[test]
fn linear_vector_view_zero_based_order_is_classified_correctly() {
    let mut points: LinearVector<Point2> = LinearVector::default();
    points.append(Point2::new(0.0, 0.0));
    points.append(Point2::new(7.0, 0.0));
    points.append(Point2::new(7.0, 3.0));
    points.append(Point2::new(0.0, 3.0));
    points.append(Point2::new(0.0, 0.0));
    let view = points.to_array1_view();
    assert_eq!(view.lower(), 0);
    assert_eq!(view.upper(), 4);
    assert!(
        view.value(0)
            .unwrap()
            .is_equal(points[0], precision::p_confusion())
    );
    assert!(
        view.value(3)
            .unwrap()
            .is_equal(points[3], precision::p_confusion())
    );
    let classifier =
        Class2d::from_array(&points.to_array1_view(), 1.0e-7, 1.0e-7, 0.0, 0.0, 7.0, 3.0);
    assert_eq!(
        classifier.si_dans(Point2::new(2.0, 1.0)),
        Class2dResult::Inside
    );
    assert_eq!(
        classifier.si_dans(Point2::new(8.0, 1.0)),
        Class2dResult::Outside
    );
}

#[test]
fn planar_hole_perform_and_restriction_preserve_wire_roles() {
    let f = face(&HOLED);
    let classifier = BRepTopAdaptorFClass2d::new(&f.face, precision::p_confusion()).unwrap();
    let material = Point2::new(1.0, 1.0);
    let hole = Point2::new(5.0, 4.0);
    let outside = Point2::new(14.0, 4.0);
    assert_eq!(classifier.perform(material, true).unwrap(), State::In);
    assert_eq!(classifier.perform(hole, true).unwrap(), State::Out);
    assert_eq!(classifier.perform(outside, true).unwrap(), State::Out);
    assert_eq!(
        classifier
            .test_on_restriction(material, 1.0e-7, true)
            .unwrap(),
        State::In
    );
    assert_eq!(
        classifier.test_on_restriction(hole, 1.0e-7, true).unwrap(),
        State::Out
    );
    assert_eq!(
        classifier
            .test_on_restriction(outside, 1.0e-7, true)
            .unwrap(),
        State::Out
    );
}

#[test]
fn boundary_perform_uses_repeatable_exact_fallback() {
    let f = face(&HOLED);
    let classifier = BRepTopAdaptorFClass2d::new(&f.face, precision::p_confusion()).unwrap();
    let boundary = Point2::new(0.0, 3.0);
    let exact = exact_state(&f, boundary);
    assert_eq!(exact, State::On);
    for _ in 0..20 {
        assert_eq!(classifier.perform(boundary, true).unwrap(), exact);
        assert_eq!(
            classifier
                .test_on_restriction(boundary, 1.0e-6, true)
                .unwrap(),
            State::On
        );
    }
}

#[test]
fn periodic_cylinder_recadres_equivalent_parameters() {
    let f = face(&CYLINDER);
    let classifier = BRepTopAdaptorFClass2d::new(&f.face, precision::p_confusion()).unwrap();
    let canonical = Point2::new(0.75, 2.0);
    let shifted = Point2::new(canonical.x() + 4.0 * PI, canonical.y());
    assert_eq!(classifier.perform(canonical, true).unwrap(), State::In);
    assert_eq!(
        classifier.perform(shifted, true).unwrap(),
        classifier.perform(canonical, true).unwrap()
    );
    assert_eq!(
        classifier
            .test_on_restriction(shifted, 1.0e-7, true)
            .unwrap(),
        classifier
            .test_on_restriction(canonical, 1.0e-7, true)
            .unwrap()
    );
}

#[test]
fn disconnected_wire_falls_back_to_stable_exact_classification() {
    let f = face(&DISCONNECTED);
    let point = Point2::new(5.0, 5.0);
    let expected = exact_state(&f, point);
    let classifier = BRepTopAdaptorFClass2d::new(&f.face, precision::p_confusion()).unwrap();
    for _ in 0..10 {
        assert_eq!(classifier.perform(point, false).unwrap(), expected);
        let on_restriction = classifier.test_on_restriction(point, precision::p_confusion(), false);
        assert_eq!(on_restriction.unwrap(), expected);
    }
}
