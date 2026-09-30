//! Behavior tests of BRepIntCurveSurface_Inter (no upstream GoogleTest suite exists). The
//! numeric probes (bics_* requests) compare it bit for bit with OCCT; these tests pin the
//! documented behavior on faces built by the probe's face reader.
mod adaptor_records;
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;

use adaptor_records::{CYLINDER, HOLED};
use face::{TpFace, tp_face};
use print::Reader;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeomalgo::int_curve_surface::TransitionOnCurve;
use rustycad_tkmath::gp::{Axis2, Direction3, Line3, Point3};
use rustycad_tktopalgo::brep_int_curve_surface::BRepIntCurveSurfaceInter;

fn face(record: &[f64]) -> TpFace {
    tp_face(&mut Reader::new(record)).expect("face record")
}

fn line(p: (f64, f64, f64), d: (f64, f64, f64)) -> Line3 {
    Line3::new(
        Point3::new(p.0, p.1, p.2),
        Direction3::new(d.0, d.1, d.2).unwrap(),
    )
}

/// The compound of the faces of the records, in order.
fn compound(records: &[&[f64]]) -> (Vec<TpFace>, Shape) {
    let builder = BRepBuilder::default();
    let mut shape = Compound::default();
    builder.make_compound(&mut shape);
    let faces: Vec<TpFace> = records.iter().map(|record| face(record)).collect();
    for f in &faces {
        builder.add(&mut shape, &f.face).unwrap();
    }
    (faces, shape.into_shape())
}

fn exception(result: Result<impl Sized, rustycad_tkernel::standard::Failure>) -> String {
    result
        .err()
        .expect("an exception")
        .exception_type()
        .to_string()
}

#[test]
fn a_fresh_object_reports_a_face_but_no_point() {
    let s = BRepIntCurveSurfaceInter::new();
    assert!(s.more());
    assert_eq!(exception(s.point()), "StdFail_NotDone");
    assert_eq!(exception(s.pnt()), "StdFail_NotDone");
    assert_eq!(exception(s.u()), "StdFail_NotDone");
    assert_eq!(exception(s.v()), "StdFail_NotDone");
    assert_eq!(exception(s.w()), "StdFail_NotDone");
    assert_eq!(exception(s.state()), "StdFail_NotDone");
    assert_eq!(exception(s.transition()), "StdFail_NotDone");
    assert_eq!(exception(s.face()), "Standard_OutOfRange");
}

#[test]
fn a_vertical_line_meets_the_material_of_the_holed_face_once() {
    let (faces, shape) = compound(&[&HOLED]);
    let mut s = BRepIntCurveSurfaceInter::new();
    s.init_line(&shape, line((1.0, 1.0, -5.0), (0.0, 0.0, 1.0)), 1e-7)
        .unwrap();
    assert!(s.more());
    assert!(s.face().unwrap().is_same(&faces[0].face));
    assert_eq!((s.u().unwrap(), s.v().unwrap()), (1.0, 1.0));
    assert_eq!(s.w().unwrap(), 5.0);
    assert_eq!(s.state().unwrap(), State::In);
    assert_eq!(s.pnt().unwrap(), Point3::new(1.0, 1.0, 0.0));
    let p = s.point().unwrap();
    assert_eq!((p.u(), p.v(), p.w()), (1.0, 1.0, 5.0));
    assert_eq!(p.transition(), s.transition().unwrap());
    s.next().unwrap();
    assert!(!s.more());
    assert_eq!(exception(s.point()), "StdFail_NotDone");
    assert_eq!(exception(s.face()), "Standard_OutOfRange");
}

#[test]
fn a_line_through_the_hole_finds_nothing() {
    let (_, shape) = compound(&[&HOLED]);
    let mut s = BRepIntCurveSurfaceInter::new();
    s.init_line(&shape, line((6.0, 4.0, -5.0), (0.0, 0.0, 1.0)), 1e-7)
        .unwrap();
    assert!(!s.more());
    // Every Next past the end keeps moving the face index further, as natively.
    s.next().unwrap();
    s.next().unwrap();
    assert!(!s.more());
}

#[test]
fn a_line_across_the_cylinder_enters_then_leaves() {
    let (_, shape) = compound(&[&CYLINDER]);
    let mut s = BRepIntCurveSurfaceInter::new();
    s.init_line(&shape, line((-5.0, 0.0, 2.0), (1.0, 0.0, 0.0)), 1e-7)
        .unwrap();
    let mut hits: Vec<(f64, TransitionOnCurve)> = Vec::new();
    while s.more() {
        hits.push((s.w().unwrap(), s.transition().unwrap()));
        s.next().unwrap();
    }
    // IntCurveSurface_HInter does not sort its points (unlike IntCurvesFace).
    hits.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(hits.len(), 2);
    assert!((hits[0].0 - 3.0).abs() <= 1e-9 && (hits[1].0 - 7.0).abs() <= 1e-9);
    assert_eq!(hits[0].1, TransitionOnCurve::In);
    assert_eq!(hits[1].1, TransitionOnCurve::Out);
}

#[test]
fn faces_are_visited_in_order_and_boxes_reject_missed_faces() {
    let (faces, shape) = compound(&[&CYLINDER, &HOLED]);
    let mut s = BRepIntCurveSurfaceInter::new();
    s.load(&shape, 1e-7).unwrap();
    assert!(s.more());
    let far = GeomAdaptorCurve::with_curve(
        &Handle::new(GeomLine::from_lin(line((50.0, 50.0, 0.0), (0.0, 0.0, 1.0))))
            .cast::<dyn GeomCurve>(),
    )
    .unwrap();
    s.init(&far).unwrap();
    assert!(!s.more());
    let across = GeomAdaptorCurve::with_curve(
        &Handle::new(GeomLine::from_lin(line((1.0, 1.0, -5.0), (0.0, 0.0, 1.0))))
            .cast::<dyn GeomCurve>(),
    )
    .unwrap();
    s.init(&across).unwrap();
    // The vertical line at (1, 1) runs inside the cylinder of radius 2 (no point) and
    // crosses the material of the holed face, the second face.
    assert!(s.more());
    assert!(s.face().unwrap().is_same(&faces[1].face));
}

#[test]
fn a_circle_meets_the_holed_face_where_it_crosses_the_material() {
    let (_, shape) = compound(&[&HOLED]);
    let frame = Axis2::with_x_direction(
        Point3::new(1.0, 1.0, 0.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, 0.5).unwrap()).cast();
    let curve = GeomAdaptorCurve::with_curve(&circle).unwrap();
    let mut s = BRepIntCurveSurfaceInter::new();
    s.init_curve(&shape, &curve, 1e-7).unwrap();
    let mut count = 0;
    while s.more() {
        assert_eq!(s.state().unwrap(), State::In);
        assert!(s.pnt().unwrap().z().abs() <= 1e-12);
        count += 1;
        s.next().unwrap();
    }
    assert_eq!(count, 2);
}
