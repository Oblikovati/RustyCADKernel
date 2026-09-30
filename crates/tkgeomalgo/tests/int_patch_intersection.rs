//! Behavior of IntPatch_Intersection, the Perform overloads of
//! IntPatch_PrmPrmIntersection and IntPatch_WLineTool::ExtendTwoWLines (design section 18)
//! beyond the numeric probe, which compares them with native OCCT bit for bit.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_patch::*;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTypeTrans};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3};

const SQUARE: (f64, f64, f64, f64) = (-5.0, 5.0, -5.0, 5.0);

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}

/// The plane z = `height` (parameters x and y), bounded by `SQUARE`.
fn level(height: f64) -> Adaptor3dSurfaceHandle {
    let axis = Axis3::from_normal(Point3::new(0.0, 0.0, height), z()).unwrap();
    let s: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(axis)).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&s, SQUARE).unwrap()).cast()
}

/// The unit sphere at the origin.
fn ball() -> Adaptor3dSurfaceHandle {
    let axis = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), z()).unwrap();
    let s: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(axis, 1.0).unwrap()).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&s).unwrap()).cast()
}

fn domain(s: &Adaptor3dSurfaceHandle) -> Adaptor3dTopolTool {
    Adaptor3dTopolTool::with_surface(s).unwrap()
}

/// Every point of the line is on the circle of the unit sphere at z = 0.5.
fn assert_on_circle(line: &Handle<dyn IntPatchLine>) {
    let wline: Handle<IntPatchWLine> = line.cast();
    let nb = wline.nb_pnts().unwrap();
    assert!(nb >= 2, "a walking line has at least 2 points, got {nb}");
    for i in 1..=nb {
        let p = wline.point(i).unwrap().value();
        assert!(
            (p.z() - 0.5).abs() < 1.0e-6,
            "point {i} is off the plane: {p:?}"
        );
        let r = (p.x() * p.x() + p.y() * p.y()).sqrt();
        assert!(
            (r - 0.75f64.sqrt()).abs() < 1.0e-6,
            "point {i} is off the circle: {p:?}"
        );
    }
}

#[test]
fn analytic_intersection_of_a_sphere_and_a_plane_is_a_circle() {
    let (s1, s2) = (ball(), level(0.5));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let inter = IntPatchIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1.0e-7, 1.0e-7).unwrap();
    assert!(inter.is_done());
    assert!(!inter.is_empty().unwrap());
    assert!(!inter.tangent_faces().unwrap());
    assert!(inter.nb_lines().unwrap() >= 1);
    assert_eq!(inter.line(1).unwrap().arc_type(), IntPatchIType::Circle);
    assert_eq!(
        inter.sequence_of_line().len() as i32,
        inter.nb_lines().unwrap()
    );
}

#[test]
fn parametric_intersection_of_a_sphere_and_a_plane_walks_the_circle() {
    let (s1, s2) = (ball(), level(0.5));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let mut inter = IntPatchIntersection::new();
    let options = IntPatchIntersectionOptions {
        is_geom_int: false,
        ..Default::default()
    };
    inter
        .perform(&s1, &d1, &s2, &d2, (1.0e-7, 1.0e-7), options)
        .unwrap();
    assert!(inter.is_done());
    assert!(inter.nb_lines().unwrap() >= 1);
    for line in inter.sequence_of_line() {
        assert_eq!(line.arc_type(), IntPatchIType::Walking);
        assert_on_circle(line);
    }
}

#[test]
fn accessors_out_of_range_raise() {
    let (s1, s2) = (ball(), level(3.0));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let inter = IntPatchIntersection::with_surfaces(&s1, &d1, &s2, &d2, 1.0e-7, 1.0e-7).unwrap();
    assert!(inter.is_done());
    assert!(inter.is_empty().unwrap());
    assert!(inter.line(0).is_err());
    assert!(inter.line(1).is_err());
    assert!(inter.point(1).is_err());
    inter.dump(0, &s1, &d1, &s2, &d2);
}

#[test]
fn accessors_before_perform_raise_not_done() {
    let inter = IntPatchIntersection::new();
    assert!(!inter.is_done());
    assert!(inter.is_empty().is_err());
    assert!(inter.nb_lines().is_err());
}

#[test]
fn a_pole_near_the_other_surface_is_singular() {
    let (s1, near, far) = (ball(), level(1.0 + 5.0e-6), level(3.0));
    let d1 = domain(&s1);
    let (singular, dist) = IntPatchIntersection::check_singular_points(&s1, &d1, &near).unwrap();
    assert!(singular);
    assert!(
        (dist - 5.0e-6).abs() < 1.0e-9,
        "distance to the pole: {dist}"
    );
    let (dn, df) = (domain(&near), domain(&far));
    assert_eq!(
        IntPatchIntersection::define_uv_max_step(&s1, &d1, &near, &dn).unwrap(),
        0.0001
    );
    assert_eq!(
        IntPatchIntersection::define_uv_max_step(&s1, &d1, &far, &df).unwrap(),
        0.001
    );
    let (same, _) = IntPatchIntersection::check_singular_points(&s1, &d1, &s1).unwrap();
    assert!(!same);
}

#[test]
fn prmprm_perform_walks_the_circle() {
    let (s1, s2) = (ball(), level(0.5));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let mut inter = IntPatchPrmPrmIntersection::new();
    inter
        .perform(&s1, &d1, &s2, &d2, 1.0e-8, 1.0e-8, 0.01, 0.01, true)
        .unwrap();
    assert!(inter.is_done());
    assert!(!inter.is_empty().unwrap());
    for i in 1..=inter.nb_lines().unwrap() {
        assert_on_circle(&inter.line(i).unwrap());
    }
}

#[test]
fn prmprm_perform_with_no_start_point_is_done_and_empty() {
    let (s1, s2) = (ball(), level(0.5));
    let (d1, d2) = (domain(&s1), domain(&s2));
    let tolerances = IntPatchPrmPrmTolerances {
        tol_tangency: 1.0e-8,
        epsilon: 1.0e-8,
        deflection: 0.01,
        increment: 0.01,
    };
    let mut inter = IntPatchPrmPrmIntersection::new();
    inter
        .perform_with_points((&s1, &d1), (&s2, &d2), tolerances, &mut Vec::new())
        .unwrap();
    assert!(inter.is_done());
    assert!(inter.is_empty().unwrap());
}

/// The plane through the origin of normal (0, 0.6, 0.8), which holds the x axis.
fn tilted() -> Adaptor3dSurfaceHandle {
    let normal = Direction3::new(0.0, 0.6, 0.8).unwrap();
    let axis = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), normal).unwrap();
    let s: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(axis)).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&s, SQUARE).unwrap()).cast()
}

fn plane_parameters(s: &Adaptor3dSurfaceHandle, p: Point3) -> (f64, f64) {
    elslib::parameters(&s.plane().unwrap(), p)
}

/// A walking line of 6 points of the x axis from `x0` to `x1` on `s1` and `s2`, with
/// vertices at its ends.
fn segment(
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
    x0: f64,
    x1: f64,
) -> Handle<dyn IntPatchLine> {
    let points: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    for i in 0..6 {
        let p = Point3::new(x0 + (x1 - x0) * f64::from(i) / 5.0, 0.0, 0.0);
        let ((u1, v1), (u2, v2)) = (plane_parameters(s1, p), plane_parameters(s2, p));
        let mut q = IntSurfPntOn2S::new();
        q.set_value_all(p, u1, v1, u2, v2);
        points.add(&q);
    }
    let line = IntPatchWLine::new(&points, false, IntSurfTypeTrans::In, IntSurfTypeTrans::Out);
    line.set_creating_way_info(IntPatchWLType::ImpImp);
    for i in [1, 6] {
        let q = points.value(i).unwrap();
        let mut v = IntPatchPoint::new();
        v.set_value_tangent(q.value(), 1.0e-7, false);
        let (a, b, c, d) = q.parameters();
        v.set_parameters(a, b, c, d);
        v.set_parameter(f64::from(i));
        line.add_vertex(&v, false);
    }
    Handle::new(line).cast()
}

fn square_box() -> BoundingBox2 {
    let mut b = BoundingBox2::default();
    b.add_point(Point2::new(SQUARE.0, SQUARE.2));
    b.add_point(Point2::new(SQUARE.1, SQUARE.3));
    b.enlarge(precision::p_confusion());
    b
}

fn extended(
    lines: &mut Vec<Handle<dyn IntPatchLine>>,
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
) {
    let b = square_box();
    IntPatchWLineTool::extend_two_wlines(lines, s1, s2, 1.0e-7, &[0.0; 4], &b, &b, &[]).unwrap();
}

#[test]
fn extend_two_wlines_joins_lines_across_a_gap() {
    let (s1, s2) = (level(0.0), tilted());
    let mut lines = vec![segment(&s1, &s2, 0.0, 1.0), segment(&s1, &s2, 1.1, 2.0)];
    extended(&mut lines, &s1, &s2);
    assert_eq!(lines.len(), 1, "the two lines of the x axis become one");
    let wline: Handle<IntPatchWLine> = lines[0].cast();
    let nb = wline.nb_pnts().unwrap();
    let (first, last) = (
        wline.point(1).unwrap().value(),
        wline.point(nb).unwrap().value(),
    );
    assert!(
        first.distance(&Point3::new(0.0, 0.0, 0.0)) < 1.0e-9,
        "first point: {first:?}"
    );
    assert!(
        last.distance(&Point3::new(2.0, 0.0, 0.0)) < 1.0e-9,
        "last point: {last:?}"
    );
}

#[test]
fn extend_two_wlines_keeps_a_single_line() {
    let (s1, s2) = (level(0.0), tilted());
    let mut lines = vec![segment(&s1, &s2, 0.0, 1.0)];
    extended(&mut lines, &s1, &s2);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].cast::<IntPatchWLine>().nb_pnts().unwrap(), 6);
}
