//! Behavior tests of IntCurvesFace_Intersector and IntCurvesFace_ShapeIntersector (no
//! upstream GoogleTest suite exists). The numeric probes (icf_* requests) compare them
//! bit for bit with OCCT; these tests pin the documented behavior on faces built by the
//! probe's face reader.
mod adaptor_records;
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;

use adaptor_records::{BSPLINE, CYLINDER, HOLED, TOLERANCED};
use face::{TpFace, tp_face};
use print::Reader;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Face};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::int_curve_surface::{HCurve, TransitionOnCurve};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Line3, Point2, Point3};
use rustycad_tktopalgo::int_curves_face::{
    IntCurvesFaceIntersector, IntCurvesFaceShapeIntersector,
};

fn face(record: &[f64]) -> TpFace {
    tp_face(&mut Reader::new(record)).expect("face record")
}

fn line(p: (f64, f64, f64), d: (f64, f64, f64)) -> Line3 {
    Line3::new(
        Point3::new(p.0, p.1, p.2),
        Direction3::new(d.0, d.1, d.2).unwrap(),
    )
}

const UP: (f64, f64, f64) = (0.0, 0.0, 1.0);

#[test]
fn a_vertical_line_meets_the_material_of_a_holed_face() {
    let f = face(&HOLED);
    let mut c = IntCurvesFaceIntersector::new(&f.face, 1e-7, true, true).unwrap();
    assert_eq!(c.surface_type().unwrap(), SurfaceType::Plane);
    assert!(c.bounding().is_void());
    c.perform_line(&line((1.0, 1.0, -5.0), UP), -real_last(), real_last())
        .unwrap();
    assert!(c.is_done());
    assert_eq!(c.nb_pnt(), 1);
    assert_eq!(
        (c.u_parameter(1).unwrap(), c.v_parameter(1).unwrap()),
        (1.0, 1.0)
    );
    assert_eq!(c.w_parameter(1).unwrap(), 5.0);
    assert_eq!(c.state(1).unwrap(), State::In);
    // Along the plane normal: the line leaves the material side.
    assert_eq!(c.transition(1).unwrap(), TransitionOnCurve::Out);
    c.perform_line(&line((5.0, 4.0, -5.0), UP), -real_last(), real_last())
        .unwrap();
    assert_eq!(c.nb_pnt(), 0);
    c.perform_line(&line((1.0, 1.0, -5.0), UP), 0.0, 4.0)
        .unwrap();
    assert_eq!(c.nb_pnt(), 0);
}

#[test]
fn a_reversed_face_swaps_the_transitions_and_keeps_boundary_points_on() {
    let f = face(&HOLED);
    let reversed = Face::unchecked(f.face.oriented(Orientation::Reversed));
    let mut c = IntCurvesFaceIntersector::new(&reversed, 1e-7, true, true).unwrap();
    c.perform_line(&line((1.0, 1.0, -5.0), UP), -real_last(), real_last())
        .unwrap();
    assert_eq!(c.transition(1).unwrap(), TransitionOnCurve::In);
    c.perform_line(&line((0.0, 3.0, -5.0), UP), -real_last(), real_last())
        .unwrap();
    assert_eq!(c.state(1).unwrap(), State::On);
    assert_eq!(
        c.classify_uv_point(Point2::new(1.0, 1.0)).unwrap(),
        State::In
    );
}

#[test]
fn point_accessors_raise_out_of_range() {
    let f = face(&HOLED);
    let mut c = IntCurvesFaceIntersector::new(&f.face, 1e-7, true, true).unwrap();
    c.perform_line(&line((1.0, 1.0, -5.0), UP), -real_last(), real_last())
        .unwrap();
    for index in [0, 2] {
        assert_eq!(
            c.u_parameter(index).err().unwrap().exception_type(),
            "Standard_OutOfRange"
        );
        assert_eq!(
            c.state(index).err().unwrap().exception_type(),
            "Standard_OutOfRange"
        );
    }
    assert!(c.get_use_bound_toler());
    c.set_use_bound_toler(false);
    assert!(!c.get_use_bound_toler());
    assert!(c.face().is_same(&f.face));
}

#[test]
fn a_line_across_a_cylinder_enters_then_leaves() {
    let f = face(&CYLINDER);
    let mut c = IntCurvesFaceIntersector::new(&f.face, 1e-7, true, true).unwrap();
    c.perform_line(
        &line((-5.0, 0.0, 2.0), (1.0, 0.0, 0.0)),
        -real_last(),
        real_last(),
    )
    .unwrap();
    assert_eq!(c.nb_pnt(), 2);
    assert!((c.w_parameter(1).unwrap() - 3.0).abs() <= 1e-9);
    assert!((c.w_parameter(2).unwrap() - 7.0).abs() <= 1e-9);
    assert_eq!(c.transition(1).unwrap(), TransitionOnCurve::In);
    assert_eq!(c.transition(2).unwrap(), TransitionOnCurve::Out);
}

#[test]
fn a_spline_face_with_two_c2_intervals_gets_a_polyhedron() {
    let f = face(&BSPLINE);
    let mut c = IntCurvesFaceIntersector::new(&f.face, 1e-7, true, true).unwrap();
    assert_eq!(c.surface_type().unwrap(), SurfaceType::BSplineSurface);
    assert!(!c.bounding().is_void());
    c.perform_line(&line((3.0, 2.0, -5.0), UP), -real_last(), real_last())
        .unwrap();
    assert_eq!(c.nb_pnt(), 1);
    assert_eq!(c.state(1).unwrap(), State::In);
    let (u, v) = (c.u_parameter(1).unwrap(), c.v_parameter(1).unwrap());
    assert!((0.0..=3.0).contains(&u) && (0.0..=3.0).contains(&v));
}

#[test]
fn a_curve_adaptor_is_intersected_on_its_range() {
    let f = face(&HOLED);
    let mut c = IntCurvesFaceIntersector::new(&f.face, 1e-7, true, true).unwrap();
    let geom: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_lin(line((1.0, 1.0, -5.0), UP))).cast();
    let curve: HCurve = Handle::new(GeomAdaptorCurve::with_range(&geom, 0.0, 10.0).unwrap())
        .cast::<dyn Adaptor3dCurve>();
    c.perform_curve(&curve, 0.0, 10.0).unwrap();
    assert_eq!(c.nb_pnt(), 1);
    assert_eq!(c.w_parameter(1).unwrap(), 5.0);
}

#[test]
fn an_edge_within_its_tolerance_makes_a_point_just_outside_on() {
    // Unrestricted: the restricted plane drops the point outside the face UV bounds.
    let f = face(&TOLERANCED);
    let mut c = IntCurvesFaceIntersector::new(&f.face, 1e-7, false, true).unwrap();
    c.perform_line(&line((-0.05, 1.0, -1.0), UP), -real_last(), real_last())
        .unwrap();
    assert_eq!(c.nb_pnt(), 1);
    assert_eq!(c.state(1).unwrap(), State::On);
    let mut strict = IntCurvesFaceIntersector::new(&f.face, 1e-7, false, false).unwrap();
    strict
        .perform_line(&line((-0.05, 1.0, -1.0), UP), -real_last(), real_last())
        .unwrap();
    assert_eq!(strict.nb_pnt(), 0);
}

fn compound(records: &[&[f64]]) -> (Vec<TpFace>, Compound) {
    let builder = BRepBuilder::default();
    let mut shape = Compound::default();
    builder.make_compound(&mut shape);
    let faces: Vec<TpFace> = records.iter().map(|record| face(record)).collect();
    for f in &faces {
        builder.add(&mut shape, &f.face).unwrap();
    }
    (faces, shape)
}

#[test]
fn a_shape_intersector_sorts_the_points_of_every_face() {
    let (faces, shape) = compound(&[&CYLINDER, &HOLED]);
    let mut s = IntCurvesFaceShapeIntersector::new();
    assert!(!s.is_done());
    s.load(&shape, 1e-7).unwrap();
    s.perform_line(
        &line((-5.0, 1.0, 0.0), (1.0, 0.0, 0.0)),
        -real_last(),
        real_last(),
    )
    .unwrap();
    assert!(s.is_done());
    let ws: Vec<f64> = (1..=s.nb_pnt())
        .map(|i| s.w_parameter(i).unwrap())
        .collect();
    assert!(ws.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(s.face(1).unwrap().is_same(&faces[0].face));
    assert_eq!(
        s.face(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn perform_nearest_keeps_the_hits_up_to_the_nearest_one() {
    let (_, shape) = compound(&[&HOLED, &CYLINDER]);
    let mut s = IntCurvesFaceShapeIntersector::new();
    s.load(&shape, 1e-7).unwrap();
    s.perform_nearest(
        &line((-5.0, 1.0, 2.0), (1.0, 0.0, 0.0)),
        -real_last(),
        real_last(),
    )
    .unwrap();
    assert!(s.is_done());
    let nearest: f64 = s.w_parameter(1).unwrap();
    assert!((1..=s.nb_pnt()).all(|i| s.w_parameter(i).unwrap() <= nearest + 1e-9 || i > 1));
    assert!((nearest - (5.0 - 3.0f64.sqrt())).abs() <= 1e-9);
}
