//! The ChFiKPart helpers: the straight pcurve, the projection of a line on a plane and
//! the guide circle of a corner between two planes.
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfikpart::{corner_spine, pcurve, proj_pc};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point2, Point3};

fn plane(n: (f64, f64, f64), x: (f64, f64, f64)) -> Handle<dyn GeomSurface> {
    let frame = Axis3::new(
        Point3::default(),
        Direction3::new(n.0, n.1, n.2).unwrap(),
        Direction3::new(x.0, x.1, x.2).unwrap(),
    )
    .unwrap();
    Handle::new(GeomPlane::new(frame)).cast()
}

#[test]
fn pcurve_is_the_straight_segment_on_the_range() {
    let pc = pcurve(Point2::new(0.0, 0.0), Point2::new(2.0, 4.0), 1.0, 3.0).unwrap();
    let c = pc.get().unwrap();
    assert_eq!(
        (c.first_parameter(), c.last_parameter(), c.degree()),
        (1.0, 3.0, 1)
    );
    assert!(c.value(2.0).unwrap().distance(Point2::new(1.0, 2.0)) < 1e-12);
}

#[test]
fn a_line_of_a_plane_projects_on_a_line() {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(1.0, 2.0, 0.0),
        Direction3::new(1.0, 1.0, 0.0).unwrap(),
    )))
    .cast();
    let cg = GeomAdaptorCurve::with_range(&line, 0.0, 2.0).unwrap();
    let sg = GeomAdaptorSurface::with_surface(&plane((0.0, 0.0, 1.0), (1.0, 0.0, 0.0))).unwrap();
    let pc = proj_pc(&cg, &sg).unwrap();
    assert!(pc.value(0.0).unwrap().distance(Point2::new(1.0, 2.0)) < 1e-12);
}

#[test]
fn the_corner_circle_joins_the_two_points() {
    let s1: Handle<dyn Adaptor3dSurface> = Handle::new(
        GeomAdaptorSurface::with_surface(&plane((0.0, 0.0, 1.0), (1.0, 0.0, 0.0))).unwrap(),
    )
    .cast();
    let s2: Handle<dyn Adaptor3dSurface> = Handle::new(
        GeomAdaptorSurface::with_surface(&plane((1.0, 0.0, 0.0), (0.0, 1.0, 0.0))).unwrap(),
    )
    .cast();
    let (cyl, circ, first, last) = corner_spine(
        (&s1, &s2),
        (
            Point2::new(1.0, 1.0),
            Point2::new(1.0, 1.0),
            Point2::new(2.0, 3.0),
        ),
        0.5,
    )
    .unwrap();
    assert_eq!(first, 0.0);
    assert!(last > 0.0);
    assert_eq!((cyl.radius(), circ.radius()), (0.5, 0.5));
    assert!(
        cyl.position()
            .location()
            .distance(&circ.position().location())
            < 1e-12
    );
}
