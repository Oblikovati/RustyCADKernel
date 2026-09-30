//! Native analytic, spline and swept-surface grid-count inputs.
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    standard::Handle,
};
use rustycad_tkg3d::geom::*;
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Plane3, Point3};
pub fn cells_surface(kind: i32) -> Handle<dyn GeomSurface> {
    match kind {
        0 => Handle::new(GeomPlane::from_pln(Plane3::default())).cast(),
        1 => Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap()).cast(),
        2 => Handle::new(GeomToroidalSurface::new(Axis3::default(), 3.0, 1.0).unwrap()).cast(),
        3..=5 => spline_surface(kind),
        _ => swept_surface(kind),
    }
}
fn spline_surface(kind: i32) -> Handle<dyn GeomSurface> {
    let (nu, nv) = if kind == 3 {
        (4, 2)
    } else if kind == 4 {
        (2, 4)
    } else {
        (4, 4)
    };
    let mut poles = Array2::with_bounds(1, nu, 1, nv).unwrap();
    for u in 1..=nu {
        for v in 1..=nv {
            poles[(u, v)] = Point3::new(f64::from(u), f64::from(v), f64::from((u * v) % 3) * 0.25);
        }
    }
    if kind != 5 {
        return Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast();
    }
    let knots = two_values(0.0, 1.0);
    let mults = two_values(4, 4);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &mults),
            (&knots, &mults),
            (3, 3),
            (false, false),
        )
        .unwrap(),
    )
    .cast()
}
fn swept_surface(kind: i32) -> Handle<dyn GeomSurface> {
    let curve = if kind < 8 {
        Handle::new(GeomLine::new(Axis1::new(
            Point3::new(2.0, 0.0, 0.0),
            Direction3::Z,
        )))
        .cast()
    } else {
        spline_curve(if kind < 10 { 3 } else { 1 })
    };
    if kind % 2 == 0 {
        return Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, Direction3::Y).unwrap())
            .cast();
    }
    Handle::new(GeomSurfaceOfRevolution::new(&curve, Axis1::default()).unwrap()).cast()
}
fn spline_curve(degree: i32) -> Handle<dyn GeomCurve> {
    let mut poles = Array1::with_bounds(1, degree + 1).unwrap();
    for index in 1..=degree + 1 {
        poles[index] = Point3::new(2.0 + f64::from(index % 2) * 0.25, 0.0, f64::from(index));
    }
    Handle::new(
        GeomBSplineCurve::new(
            &poles,
            &two_values(0.0, 1.0),
            &two_values(degree + 1, degree + 1),
            degree,
            false,
        )
        .unwrap(),
    )
    .cast()
}
fn two_values<T: Default + Clone>(first: T, last: T) -> Array1<'static, T> {
    let mut values = Array1::with_bounds(1, 2).unwrap();
    values[1] = first;
    values[2] = last;
    values
}
