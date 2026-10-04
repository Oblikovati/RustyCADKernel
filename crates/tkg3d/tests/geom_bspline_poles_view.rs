//! Read-through pole access matching the native const-reference behavior within a lock scope.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface};
use rustycad_tkmath::gp::Point3;

fn point(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

fn array<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut result = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

fn curve() -> Handle<GeomBSplineCurve> {
    let poles = array(&[point(0., 0., 0.), point(1., 0., 0.)]);
    let knots = array(&[0., 1.]);
    let mults = array(&[2, 2]);
    Handle::new(GeomBSplineCurve::new(&poles, &knots, &mults, 1, false).unwrap())
}

fn surface() -> Handle<GeomBSplineSurface> {
    let mut poles = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = point(0., 0., 0.);
    poles[(1, 2)] = point(0., 1., 0.);
    poles[(2, 1)] = point(1., 0., 0.);
    poles[(2, 2)] = point(1., 1., 0.);
    let knots = array(&[0., 1.]);
    let mults = array(&[2, 2]);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &mults),
            (&knots, &mults),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    )
}

#[test]
fn bspline_curve_poles_array_reads_current_state_without_copying() {
    let curve = curve();
    let snapshot = curve.poles();
    curve.set_pole(1, point(5., 6., 7.)).unwrap();
    let observed = curve.poles_array(|poles| (poles[1].x(), poles[1].y(), poles[1].z()));
    assert_eq!(observed, (5., 6., 7.));
    assert_eq!(snapshot[1].x(), 0.);
}

#[test]
fn bspline_surface_poles_array_reads_current_state_without_copying() {
    let surface = surface();
    let snapshot = surface.poles();
    surface.set_pole(1, 1, point(5., 6., 7.)).unwrap();
    let observed =
        surface.poles_array(|poles| (poles[(1, 1)].x(), poles[(1, 1)].y(), poles[(1, 1)].z()));
    assert_eq!(observed, (5., 6., 7.));
    assert_eq!(snapshot[(1, 1)].x(), 0.);
}
