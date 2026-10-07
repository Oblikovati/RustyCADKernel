// Tests of BRepBlend_Walking on the fillet of a box edge.
use super::extremities::recadre_if_periodic;
use super::*;
use crate::blend_func::{ConstRad, ConstRadInv};
use crate::chfi_ds::lock;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dTopolTool};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

/// The plane of normal `n` and X direction (1, 0, 0) through the origin, trimmed.
fn plane(n: Direction3, bounds: (f64, f64, f64, f64)) -> Handle<dyn Adaptor3dSurface> {
    let frame = Axis3::new(Point3::default(), n, Direction3::X).expect("frame");
    let geom: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&geom, bounds).expect("surface")).cast()
}

/// The box edge: z = 0 (u in [0, 10], v in [0, 5]) and y = 0 (u in [0, l2], v in
/// [-5, 0]) along the X axis on [-1, 11].
fn box_edge(l2: f64) -> (BRepBlendWalking, ConstRad, ConstRadInv) {
    let s1 = plane(Direction3::Z, (0.0, 10.0, 0.0, 5.0));
    let s2 = plane(Direction3::Y, (0.0, l2, -5.0, 0.0));
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::default(), Direction3::X)).cast();
    let spine = ElSpine::new();
    spine.set_curve(&line).expect("spine");
    spine.set_first_parameter(-1.0);
    spine.set_last_parameter(11.0);
    let spine = Handle::new(spine);
    let guide: Handle<dyn Adaptor3dCurve> = spine.clone().cast();
    let mut f = ConstRad::new(&s1, &s2, &guide).expect("function");
    f.set_radius(1.0, 1);
    let mut finv = ConstRadInv::new(&s1, &s2, &guide);
    finv.set_radius(1.0, 1);
    let d1: WalkingDomain = Rc::new(Adaptor3dTopolTool::with_surface(&s1).expect("domain"));
    let d2: WalkingDomain = Rc::new(Adaptor3dTopolTool::with_surface(&s2).expect("domain"));
    (BRepBlendWalking::new(&s1, &s2, &d1, &d2, &spine), f, finv)
}

fn settings() -> WalkingTolerances {
    WalkingTolerances {
        max_step: 2.0,
        tol3d: 1e-6,
        tol_guide: 1e-5,
        fleche: 0.01,
    }
}

fn start(t: f64) -> Vector<'static> {
    math_vector(&[t, 1.0, t, -1.0]).expect("start")
}

#[test]
fn a_walking_that_is_not_done_has_no_line() {
    let (mut walking, mut f, mut finv) = box_edge(10.0);
    assert_eq!(
        walking
            .line()
            .err()
            .map(|e| e.exception_type().to_string())
            .as_deref(),
        Some("StdFail_NotDone")
    );
    assert!(walking.complete(&mut f, &mut finv, 0.0).is_err());
    assert!(walking.continu(&mut f, &mut finv, 0.0).is_err());
}

#[test]
fn the_box_edge_walks_to_both_restrictions() {
    let (mut walking, mut f, mut finv) = box_edge(10.0);
    walking
        .perform(
            &mut f,
            &mut finv,
            (1.0, 12.0),
            settings(),
            &start(1.0),
            true,
        )
        .expect("perform");
    assert!(walking.is_done());
    let line = lock(walking.line().expect("line"));
    let last = *line.point(line.nb_points()).expect("last point");
    assert_eq!(last.parameter(), 10.0);
    assert_eq!(last.point_on_s1(), Point3::new(10.0, 1.0, 0.0));
    assert_eq!(line.end_point_on_first().nb_point_on_rst(), 1);
    assert_eq!(line.end_point_on_second().nb_point_on_rst(), 1);
    assert_eq!(line.start_point_on_first().nb_point_on_rst(), 0);
}

#[test]
fn the_shorter_face_stops_the_walking_on_its_restriction() {
    let (mut walking, mut f, mut finv) = box_edge(8.0);
    walking
        .perform(
            &mut f,
            &mut finv,
            (3.0, 12.0),
            settings(),
            &start(3.0),
            true,
        )
        .expect("perform");
    let line = lock(walking.line().expect("line"));
    let last = *line.point(line.nb_points()).expect("last point");
    assert_eq!(last.parameter(), 8.0);
    assert_eq!(line.end_point_on_first().nb_point_on_rst(), 0);
    assert_eq!(line.end_point_on_second().nb_point_on_rst(), 1);
}

#[test]
fn complete_walks_back_once() {
    let (mut walking, mut f, mut finv) = box_edge(10.0);
    walking
        .perform(
            &mut f,
            &mut finv,
            (3.0, 12.0),
            settings(),
            &start(3.0),
            true,
        )
        .expect("perform");
    assert!(walking.complete(&mut f, &mut finv, -2.0).expect("complete"));
    let first = *lock(walking.line().expect("line"))
        .point(1)
        .expect("first point");
    assert_eq!(first.parameter(), 0.0);
    assert!(
        walking
            .complete(&mut f, &mut finv, -2.0)
            .expect("complete again")
    );
}

#[test]
fn singular_points_are_kept_sorted() {
    let (mut walking, _, _) = box_edge(10.0);
    for t in [5.0, 2.0, 8.0, 3.0, 9.0] {
        let mut p = BlendPoint::default();
        p.set_parameter(t);
        walking.add_singular_point(&p);
    }
    let order: Vec<f64> = walking.jalons.iter().map(|p| p.parameter()).collect();
    assert_eq!(order, [2.0, 3.0, 5.0, 8.0, 9.0]);
}

#[test]
fn periodic_parameters_come_back_within_half_a_period() {
    let period = 2.0 * std::f64::consts::PI;
    let (u, v) = recadre_if_periodic((7.0, 0.5), (0.5, 0.5), (period, 0.0));
    assert_eq!((u, v), (7.0 - period, 0.5));
    assert_eq!(
        recadre_if_periodic((-6.0, 9.0), (0.0, 1.0), (period, 4.0)),
        (-6.0 + period, 1.0)
    );
}
