//! Behavior tests of BRepApprox (no upstream GoogleTest suite exists): the surface tool,
//! the approximation line and the approximation of a line of points alone and on two
//! faces. The numeric probes (bra_* requests) compare the package bit for bit with OCCT on
//! walking lines of BRep faces; these tests pin the documented behavior.
mod adaptor_records;
#[allow(dead_code)]
#[path = "../examples/tp_probe/face.rs"]
mod face;
#[allow(dead_code)]
#[path = "../../tkg2d/examples/g2_probe/print.rs"]
mod print;

use adaptor_records::{CYLINDER, HOLED};
use face::tp_face;
use print::Reader;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeomalgo::approx_int::{ApproxIntLine, ApproxIntRange};
use rustycad_tkgeomalgo::int_imp::ThePSurfaceTool;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_approx::{
    BRepApproxApprox, BRepApproxApproxLine, BRepApproxSurface, BRepApproxSurfaceTool as Tool,
};
use std::f64::consts::TAU;

fn surface(record: &[f64]) -> BRepApproxSurface {
    let f = tp_face(&mut Reader::new(record)).expect("face record");
    Handle::new(BRepAdaptorSurface::with_face(&f.face, true).unwrap())
}

fn exception<T>(result: Result<T, Failure>) -> String {
    result
        .err()
        .expect("an exception")
        .exception_type()
        .to_string()
}

#[test]
fn the_surface_tool_forwards_the_adaptor_and_samples_ten() {
    let s = surface(&CYLINDER);
    assert_eq!(Tool::first_u_parameter(&s).unwrap(), 0.0);
    assert_eq!(Tool::last_u_parameter(&s).unwrap(), TAU);
    assert_eq!(Tool::get_type(&s).unwrap(), SurfaceType::Cylinder);
    assert_eq!(Tool::cylinder(&s).unwrap().radius(), 2.0);
    assert!(Tool::is_u_periodic(&s).unwrap());
    assert_eq!(Tool::u_period(&s).unwrap(), TAU);
    assert!(Tool::plane(&s).is_err());
    assert_eq!(
        Tool::value(&s, 0.0, 1.0).unwrap(),
        Point3::new(2.0, 0.0, 1.0)
    );
    // Unlike Adaptor3d_HSurfaceTool (15 in V for a cylinder), always 10.
    assert_eq!(Tool::nb_samples_u(&s), 10);
    assert_eq!(Tool::nb_samples_v(&s), 10);
    assert_eq!(Tool::nb_samples_u_range(&s, 0.0, 1.0), 10);
    assert_eq!(Tool::nb_samples_v_range(&s, 0.0, 1.0), 10);
    // The IntImp trait answers as the tool.
    let d1 = <Tool as ThePSurfaceTool>::d1(&s, 0.5, 2.0).unwrap();
    assert_eq!(d1.point, Tool::d1(&s, 0.5, 2.0).unwrap().point);
}

/// A clamped cubic B-spline curve through four poles.
fn cubic(poles: [Point3; 4]) -> Handle<GeomBSplineCurve> {
    let mut p: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    for (i, pole) in poles.into_iter().enumerate() {
        p[i as i32 + 1] = pole;
    }
    let mut k: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut m: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    (k[1], k[2], m[1], m[2]) = (0.0, 1.0, 4, 4);
    Handle::new(GeomBSplineCurve::new(&p, &k, &m, 3, false).unwrap())
}

#[test]
fn the_approximation_line_reads_curve_poles_before_nothing() {
    let poles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 2.0, 0.0),
        Point3::new(2.0, 2.0, 1.0),
        Point3::new(3.0, 0.0, 1.0),
    ];
    let line =
        BRepApproxApproxLine::from_curves(&cubic(poles), &Handle::default(), &Handle::default());
    assert_eq!(line.nb_pnts().unwrap(), 4);
    let p = line.point(2).unwrap();
    assert_eq!(p.value(), poles[1]);
    // A null pcurve gives the default parameters (0, 0).
    assert_eq!(p.parameters(), (0.0, 0.0, 0.0, 0.0));
    assert_eq!(exception(line.point(5)), "Standard_OutOfRange");
    let none = BRepApproxApproxLine::from_curves(
        &Handle::default(),
        &Handle::default(),
        &Handle::default(),
    );
    assert_eq!(exception(none.nb_pnts()), "Standard_NullObject");
}

#[test]
fn an_empty_line_of_points_falls_back_to_the_null_curves() {
    let line = BRepApproxApproxLine::from_line(&Handle::new(IntSurfLineOn2S::new(None)), true);
    assert_eq!(line.nb_pnts().unwrap(), 0);
    assert_eq!(line.point(7).unwrap(), IntSurfPntOn2S::new());
    // The ApproxInt constructor of TheLine.
    let built = <BRepApproxApproxLine as ApproxIntLine>::from_points(
        &Handle::new(IntSurfLineOn2S::new(None)),
        false,
    );
    assert_eq!(ApproxIntLine::nb_pnts(&built).unwrap(), 0);
}

/// The circle of radius 2 at z = 0: on the plane z = 0 (parameters x, y) and on the
/// bottom of the cylinder (parameters angle, 0).
fn bottom_circle(n: i32) -> Handle<BRepApproxApproxLine> {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for i in 0..n {
        let t: f64 = 0.25 * f64::from(i) / f64::from(n - 1) * TAU;
        let (x, y) = (2.0 * t.cos(), 2.0 * t.sin());
        let mut p = IntSurfPntOn2S::new();
        p.set_value_all(Point3::new(x, y, 0.0), x, y, t, 0.0);
        points.add(&p);
    }
    Handle::new(BRepApproxApproxLine::from_line(&points, false))
}

fn whole(n: i32) -> ApproxIntRange {
    ApproxIntRange {
        indicemin: 1,
        indicemax: n,
        ..ApproxIntRange::default()
    }
}

#[test]
fn a_line_alone_is_approximated_in_three_spaces() {
    let mut approx = BRepApproxApprox::new();
    approx.perform(&bottom_circle(20), whole(20)).unwrap();
    assert!(approx.is_done());
    assert_eq!(approx.nb_multi_curves(), 1);
    assert_eq!(approx.value(1).nb_curves().unwrap(), 3);
    assert!(approx.tol_reached3d() < 1e-3);
}

#[test]
fn a_line_on_a_plane_and_a_cylinder_uses_the_plane_as_quadric() {
    let (plane, cylinder) = (surface(&HOLED), surface(&CYLINDER));
    let mut approx = BRepApproxApprox::new();
    approx.set_parameters(1e-7, 1e-7, 4, 8, 0, 30, true, Default::default());
    approx
        .perform_on_surfaces(&plane, &cylinder, &bottom_circle(20), whole(20))
        .unwrap();
    assert!(approx.is_done());
    assert!(approx.tol_reached3d() < 1e-3);
    assert!(approx.tol_reached2d() < 1e-3);
}
