//! Drawing and variable tests of the DrawTrSurf drawables on a recording window system
//! (the batch oracle scripts never draw).
use super::*;
use crate::draw::fake_window::FakeWindowSystem;
use crate::draw::{
    DrawColorKind, DrawIStream, DrawSession, DrawViewer, DrawableHandle, RecordingConsole,
};
use rustycad_tkmath::gp::{Point2, Point3};
use std::rc::Rc;

/// The GeomTools records the tests restore.
const CIRCLE: &str = "2 0 0 0 0 0 1 1 0 0 0 1 0 5";
const BSPLINE: &str = "7 0 0 2 4 3 0 0 0 1 2 0 2 2 0 3 0 0 0 3 1 1 2 3";
const BEZIER_2D: &str = "6 0 2 0 0 1 1 2 0";
const PLANE: &str = "1 0 0 0 0 0 1 1 0 0 0 1 0";
const BSPLINE_SURFACE: &str =
    "9 0 0 0 0 2 1 4 2 3 2 0 0 0 0 1 0 1 0 0 1 1 1 2 0 0 2 1 0 3 0 0 3 1 0 0 3 1 1 2 3 0 2 1 2";

/// A session with a 3D view (1) and a 2D view (2) on fake windows, and the
/// DrawTrSurf commands.
fn session() -> (Rc<DrawSession>, Rc<FakeWindowSystem>) {
    let system = FakeWindowSystem::new();
    let viewer = DrawViewer::with_window_system(system.clone());
    let session = DrawSession::new(Rc::new(RecordingConsole::default()), viewer);
    basic_commands(&session).unwrap();
    session
        .viewer()
        .make_view(1, "+X+Y", 0, 0, 400, 400)
        .unwrap();
    session
        .viewer()
        .make_view(2, "-2D-", 0, 0, 400, 400)
        .unwrap();
    (session, system)
}

/// The drawable `type_name` restores from `record`.
fn restore(session: &DrawSession, type_name: &str, record: &str) -> DrawableHandle {
    let mut stream = DrawIStream::new(record.as_bytes().to_vec());
    session.restore(type_name, &mut stream).unwrap()
}

/// Stores and displays `drawable` in `name`, returning the window calls it made.
fn display(
    session: &DrawSession,
    system: &FakeWindowSystem,
    name: &str,
    drawable: DrawableHandle,
) -> Vec<String> {
    system.log.borrow_mut().clear();
    session.set_drawable(name, Some(drawable), true).unwrap();
    system.log.borrow().clone()
}

fn segments(log: &[String]) -> usize {
    log.iter().filter(|line| line.contains("segment")).count()
}

fn uses_color(log: &[String], kind: DrawColorKind) -> bool {
    log.iter()
        .any(|line| line.ends_with(&format!("color {}", kind.index())))
}

#[test]
fn a_circle_draws_its_polyline_and_orientation_arrow() {
    let (session, system) = session();
    let circle = restore(&session, "DrawTrSurf_Curve", CIRCLE);
    let log = display(&session, &system, "c", circle.clone());
    assert!(uses_color(&log, DrawColorKind::Jaune), "{log:?}");
    let plain = segments(&log);
    assert!(plain > 30, "{plain} segments");
    let curve = circle.downcast_kind::<DrawTrSurfCurve>().unwrap();
    curve.style().show_curvature();
    curve.style().set_display_origin(false);
    system.log.borrow_mut().clear();
    session.viewer().repaint_all().unwrap();
    // The radii add one segment per sample; the arrow (two segments) is gone.
    assert!(
        segments(&system.log.borrow()) > plain,
        "{:?}",
        system.log.borrow()
    );
}

#[test]
fn a_bspline_curve_draws_poles_curve_and_knots() {
    let (session, system) = session();
    let bspline = restore(&session, "DrawTrSurf_BSplineCurve", BSPLINE);
    let log = display(&session, &system, "bs", bspline.clone());
    assert!(uses_color(&log, DrawColorKind::Rouge) && uses_color(&log, DrawColorKind::Violet));
    let drawable = bspline.downcast_kind::<DrawTrSurfBSplineCurve>().unwrap();
    drawable.poles().clear();
    drawable.knots().clear();
    system.log.borrow_mut().clear();
    session.viewer().repaint_all().unwrap();
    let log = system.log.borrow().clone();
    assert!(!uses_color(&log, DrawColorKind::Rouge) && !uses_color(&log, DrawColorKind::Violet));
}

#[test]
fn the_ranged_bspline_drawing_covers_the_spans_of_the_range() {
    let (session, system) = session();
    let bspline = restore(&session, "DrawTrSurf_BSplineCurve", BSPLINE);
    let drawable = bspline.downcast_kind::<DrawTrSurfBSplineCurve>().unwrap();
    system.log.borrow_mut().clear();
    let mut viewer = session.viewer();
    let mut display = viewer.make_display(1);
    drawable
        .draw_on_range(&mut display, (0.5, 1.5), 2, (true, true))
        .unwrap();
    display.flush();
    drop(viewer);
    // Two spans of 30 points each plus the pole legs around pole 2.
    assert!(
        segments(&system.log.borrow()) >= 58,
        "{:?}",
        system.log.borrow()
    );
}

#[test]
fn poles_and_knots_are_found_near_the_pick_point() {
    let (session, _system) = session();
    let bspline = restore(&session, "DrawTrSurf_BSplineCurve", BSPLINE);
    let drawable = bspline.downcast_kind::<DrawTrSurfBSplineCurve>().unwrap();
    let mut viewer = session.viewer();
    let display = viewer.make_display(1);
    let pole_one = display.project(Point3::new(0.0, 0.0, 0.0));
    let zoom = display.zoom();
    let pick = (pole_one.x() * zoom, pole_one.y() * zoom);
    assert_eq!(drawable.find_pole(pick, &display, 2.0, 0).unwrap(), 1);
    assert_eq!(drawable.find_pole(pick, &display, 2.0, 1).unwrap(), 0);
    let first_knot = (pole_one.x(), pole_one.y());
    assert_eq!(
        drawable.find_knot(first_knot, &display, 1e-9, 0).unwrap(),
        1
    );
}

#[test]
fn surfaces_draw_isos_and_bounds_and_spline_surfaces_their_nets() {
    let (session, system) = session();
    let plane = restore(&session, "DrawTrSurf_Surface", PLANE);
    let log = display(&session, &system, "p", plane);
    assert!(uses_color(&log, DrawColorKind::Bleu) && uses_color(&log, DrawColorKind::Vert));
    let surface = restore(&session, "DrawTrSurf_BSplineSurface", BSPLINE_SURFACE);
    let log = display(&session, &system, "s", surface.clone());
    assert!(
        uses_color(&log, DrawColorKind::Rouge) && segments(&log) > 10,
        "{log:?}"
    );
    let part = surface.downcast_kind::<DrawTrSurfSurface>().unwrap();
    // One inner U knot (3 knots) and no inner V knot: the knot iso counts.
    assert_eq!(part.nb_isos(), (1, 0));
    part.show_isos(-2, 3);
    assert_eq!(part.nb_isos(), (2, 3));
    let bspline = surface.downcast_kind::<DrawTrSurfBSplineSurface>().unwrap();
    bspline.show_knots_isos().unwrap();
    assert_eq!(part.nb_isos(), (1, 0));
}

#[test]
fn points_and_two_dimensional_curves_draw_in_the_two_dimensional_view() {
    let (session, system) = session();
    session.set_point_2d("p", Point2::new(1.0, 2.0)).unwrap();
    session
        .set_curve_2d("b", &bezier_2d(&session), true)
        .unwrap();
    let log = system.log.borrow().clone();
    assert!(
        log.iter().any(|line| line.starts_with("102 segment")),
        "{log:?}"
    );
    assert!(
        !log.iter().any(|line| line.starts_with("101 segment")),
        "{log:?}"
    );
}

fn bezier_2d(
    session: &DrawSession,
) -> rustycad_tkernel::standard::Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve> {
    let drawable = restore(session, "DrawTrSurf_BezierCurve2d", BEZIER_2D);
    drawable
        .downcast_kind::<DrawTrSurfCurve2d>()
        .unwrap()
        .curve()
}

#[test]
fn the_getters_find_the_geometry_of_each_drawable_kind() {
    let (session, _system) = session();
    session
        .set_drawable(
            "bs",
            Some(restore(&session, "DrawTrSurf_BSplineCurve", BSPLINE)),
            false,
        )
        .unwrap();
    session
        .set_drawable(
            "c",
            Some(restore(&session, "DrawTrSurf_Curve", CIRCLE)),
            false,
        )
        .unwrap();
    session.set_point("p", Point3::new(1.0, 2.0, 3.0)).unwrap();
    session
        .set_curve_2d("b2", &bezier_2d(&session), false)
        .unwrap();
    assert!(session.get_curve("bs").is_some() && session.get_bspline_curve("bs").is_some());
    assert!(session.get_geometry("bs").is_some() && session.get_surface("bs").is_null());
    assert!(session.get_bspline_curve("c").is_null() && session.get_curve("c").is_some());
    assert_eq!(session.get_point("p"), Some(Point3::new(1.0, 2.0, 3.0)));
    assert_eq!(session.get_point_2d("p"), None);
    assert!(session.get_bezier_curve_2d("b2").is_some());
    assert!(session.get_bspline_curve_2d("b2").is_null());
    let drawable = session.get("bs").unwrap();
    assert!(drawable.downcast_kind::<DrawTrSurfDrawable>().is_some());
    assert!(drawable.downcast_kind::<DrawTrSurfPoint>().is_none());
}

#[test]
fn triangulation_edges_are_free_on_the_border_and_internal_once() {
    let (session, _system) = session();
    let record = "Poly_Triangulation\n4 2 0\n0\n0 0 0 1 0 0 1 1 0 0 1 0\n1 2 3 1 3 4\n";
    let drawable = restore(&session, "DrawTrSurf_Triangulation", record);
    let mesh = drawable.downcast_kind::<DrawTrSurfTriangulation>().unwrap();
    assert_eq!(mesh.edges().free.len(), 4);
    assert_eq!(mesh.edges().internal, vec![(3, 1)]);
}

#[test]
fn an_unreadable_record_gives_a_drawable_that_fails_to_draw() {
    let (session, system) = session();
    let broken = restore(&session, "DrawTrSurf_BezierCurve", CIRCLE);
    let failure = session.set_drawable("x", Some(broken), true).unwrap_err();
    assert!(
        failure.to_string().contains("Standard_NullObject"),
        "{failure}"
    );
    drop(system);
}

#[test]
fn an_infinite_line_is_drawn_over_a_doubled_finite_range() {
    type Failure = rustycad_tkernel::standard::Failure;
    let points = |t: f64| Ok::<Point3, Failure>(Point3::new(t, 0.0, 0.0));
    let infinite = (-2e100, 2e100);
    // The range doubles from 2 until the ends are 400 apart: [-256, 256].
    let range = super::curve_common::finite_range(infinite, points).unwrap();
    assert_eq!(range, (-256.0, 256.0));
    let half = super::curve_common::finite_range((0.0, 2e100), points).unwrap();
    assert_eq!(half, (0.0, 512.0));
    let finite = super::curve_common::finite_range((1.0, 2.0), points).unwrap();
    assert_eq!(finite, (1.0, 2.0));
    let (session, system) = session();
    let line = restore(&session, "DrawTrSurf_Curve", "1 0 0 0 1 0 0");
    assert!(segments(&display(&session, &system, "l", line)) >= 1);
}
