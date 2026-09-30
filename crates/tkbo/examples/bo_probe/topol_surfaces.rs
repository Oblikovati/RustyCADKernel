use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    standard::Handle,
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom::*, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3};
fn poles(n: i32) -> Array2<'static, Point3> {
    let mut poles = Array2::with_bounds(1, n, 1, n).unwrap();
    for i in 1..=n {
        for j in 1..=n {
            let z = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
            poles
                .set_value(i, j, Point3::new(f64::from(i), f64::from(j), z))
                .unwrap();
        }
    }
    poles
}
fn spline(n: i32, bspline: bool) -> Handle<dyn GeomSurface> {
    let grid = poles(n);
    if !bspline {
        return Handle::new(GeomBezierSurface::new(&grid).unwrap()).cast();
    }
    bspline_surface(n, &grid)
}
fn bspline_surface(n: i32, grid: &Array2<'_, Point3>) -> Handle<dyn GeomSurface> {
    let knots = [0.0, 1.0];
    let mult = [n, n];
    let knots = Array1::from_shared(&knots, 1);
    let mult = Array1::from_shared(&mult, 1);
    Handle::new(
        GeomBSplineSurface::new(
            grid,
            (&knots, &mult),
            (&knots, &mult),
            (n - 1, n - 1),
            (false, false),
        )
        .unwrap(),
    )
    .cast()
}
fn swept(kind: i32) -> Handle<dyn GeomSurface> {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    if kind == 5 {
        return Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, Direction3::X).unwrap())
            .cast();
    }
    Handle::new(
        GeomSurfaceOfRevolution::new(&curve, Axis1::new(Point3::new(1., 0., 0.), Direction3::X))
            .unwrap(),
    )
    .cast()
}
pub fn surface(v: &[f64]) -> Handle<dyn Adaptor3dSurface> {
    let kind = v[0] as i32;
    let radius = v[1];
    let geom: Handle<dyn GeomSurface> = match kind {
        1 => Handle::new(GeomCylindricalSurface::new(Axis3::default(), radius).unwrap()).cast(),
        2 => Handle::new(GeomConicalSurface::new(Axis3::default(), 0.3, radius).unwrap()).cast(),
        3 => Handle::new(GeomSphericalSurface::new(Axis3::default(), radius).unwrap()).cast(),
        4 => Handle::new(GeomToroidalSurface::new(Axis3::default(), radius * 2., radius).unwrap())
            .cast(),
        5 | 6 => swept(kind),
        7 | 8 => spline(radius as i32, kind == 8),
        _ => Handle::new(GeomPlane::new(Axis3::default())).cast(),
    };
    Handle::new(
        GeomAdaptorSurface::with_range(&geom, ((v[2], v[3], v[4], v[5]), (0., 0.))).unwrap(),
    )
    .cast()
}
