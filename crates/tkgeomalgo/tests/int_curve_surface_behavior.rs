//! Behavior tests of the IntCurveSurface port beyond the upstream suites: the
//! analytic and sampled paths of IntCurveSurface_HInter, the records, and the
//! regression of the offset-cylinder bounds (a line along the axis meets the
//! equivalent cylinder nowhere, which is not NoIntersection natively).
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCurve, GeomCylindricalSurface, GeomLine, GeomOffsetSurface, GeomPlane,
    GeomSphericalSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::int_curve_surface::inter_utils::{
    InfFlags, SortedStartPoints, UvLimits, est_lim_for_inf_offs, sort_start_points,
};
use rustycad_tkgeomalgo::int_curve_surface::{
    HCurve, HInter, Intersection, IntersectionPoint, TransitionOnCurve, tri_connex,
};
use rustycad_tkgeomalgo::int_imp::HSurfaceTool;
use rustycad_tkmath::gp::{Axis3, Direction3, Line3, Point3};

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

fn line_curve(p: Point3, d: Direction3, range: (f64, f64)) -> HCurve {
    let c: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(p, d)).cast();
    Handle::new(GeomAdaptorCurve::with_range(&c, range.0, range.1).unwrap()).cast()
}

fn adapted(s: Handle<dyn GeomSurface>) -> Adaptor3dSurfaceHandle {
    Handle::new(GeomAdaptorSurface::with_surface(&s).unwrap()).cast()
}

fn z_frame() -> Axis3 {
    Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap()
}

#[test]
fn line_crosses_a_sphere_in_two_points_entering_then_leaving() {
    let sphere = adapted(Handle::new(GeomSphericalSurface::new(z_frame(), 1.5).unwrap()).cast());
    let line = line_curve(Point3::new(-3.0, 0.0, 0.0), dir(1.0, 0.0, 0.0), (0.0, 6.0));
    let mut inter = HInter::new();
    inter.perform(&line, &sphere).unwrap();
    assert_eq!(inter.nb_points().unwrap(), 2);
    // IntAna_IntConicQuad gives the points in its root order, which the result keeps.
    let (mut first, mut second) = (inter.point(1).unwrap(), inter.point(2).unwrap());
    if first.w() > second.w() {
        std::mem::swap(&mut first, &mut second);
    }
    assert!((first.w() - 1.5).abs() < 1e-12 && (second.w() - 4.5).abs() < 1e-12);
    assert_eq!(first.transition(), TransitionOnCurve::In);
    assert_eq!(second.transition(), TransitionOnCurve::Out);
}

#[test]
fn line_in_a_plane_is_parallel_without_points() {
    let plane = adapted(Handle::new(GeomPlane::new(z_frame())).cast());
    let line = line_curve(Point3::new(-1.0, 0.0, 0.0), dir(1.0, 1.0, 0.0), (0.0, 3.0));
    let mut inter = HInter::new();
    inter.perform(&line, &plane).unwrap();
    assert!(inter.is_parallel());
    assert_eq!(inter.nb_points().unwrap(), 0);
}

#[test]
fn bezier_crossing_a_plane_is_found_by_the_polygon_search() {
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    (poles[1], poles[2], poles[3]) = (
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 1.0),
    );
    let c: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    let curve: HCurve = Handle::new(GeomAdaptorCurve::with_curve(&c).unwrap()).cast();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(z_frame())).cast();
    let bounded: Adaptor3dSurfaceHandle =
        Handle::new(GeomAdaptorSurface::with_bounds(&plane, (-5.0, 5.0, -5.0, 5.0)).unwrap())
            .cast();
    let mut inter = HInter::new();
    inter.perform(&curve, &bounded).unwrap();
    assert_eq!(inter.nb_points().unwrap(), 1);
    assert!((inter.point(1).unwrap().w() - 0.5).abs() < 1e-9);
}

#[test]
fn appended_points_within_1e8_and_same_transition_are_merged() {
    let mut records = Intersection::new();
    records.done = true;
    let p = Point3::new(0.0, 0.0, 0.0);
    records.append_point(IntersectionPoint::new(
        p,
        0.5,
        0.5,
        1.0,
        TransitionOnCurve::In,
    ));
    records.append_point(IntersectionPoint::new(
        p,
        0.5 + 5e-9,
        0.5,
        1.0,
        TransitionOnCurve::In,
    ));
    records.append_point(IntersectionPoint::new(
        p,
        0.5,
        0.5,
        1.0,
        TransitionOnCurve::Out,
    ));
    assert_eq!(records.nb_points().unwrap(), 2);
    assert_eq!(
        records.point(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn start_points_sort_by_w_then_snap_u_and_v() {
    let mut points = SortedStartPoints::default();
    points.append(0.3, 0.1, 2.0);
    points.append(0.2, 0.4, 1.0);
    points.append(0.1, 0.9, 1.0 + 1e-10);
    sort_start_points(&mut points);
    assert_eq!(points.tab_w, vec![1.0, 1.0, 2.0]);
    assert_eq!(points.tab_u, vec![0.1, 0.2, 0.3]);
}

#[test]
fn tri_connex_of_a_free_edge_gives_no_triangle() {
    // Point 1 is the grid corner: the edge 1-2 lies on the u = u0 border.
    let (returned, tri_con, _) = tri_connex(1, 1, 2, (3, 3));
    assert_eq!((returned, tri_con), (0, 0));
}

// Regression: the line along the axis of an offset cylinder meets the equivalent
// cylinder in no point; natively the V bounds then widen from RealLast to [-10, 10],
// and NoIntersection stays false (only a parallel line sets it).
#[test]
fn offset_cylinder_axis_line_bounds_v_to_ten() {
    let cyl: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(z_frame(), 1.0).unwrap()).cast();
    let offset = adapted(Handle::new(GeomOffsetSurface::new(&cyl, 0.3, false).unwrap()).cast());
    let axis = Line3::new(Point3::new(0.0, 0.0, -3.0), dir(0.0, 0.0, 1.0));
    let inf = InfFlags {
        u1: false,
        u2: false,
        v1: true,
        v2: true,
    };
    let mut limits = UvLimits {
        u1: 0.0,
        u2: 1.0,
        v1: -2e100,
        v2: 2e100,
    };
    let no_intersection =
        est_lim_for_inf_offs::<HSurfaceTool>(axis, &offset, 20, inf, &mut limits).unwrap();
    assert!(!no_intersection);
    assert_eq!((limits.v1, limits.v2), (-10.0, 10.0));
}
