//! Real geometric adaptors shared by curve tests, the native oracle twin and GPU fixtures.
use rustycad_tkernel::{
    ncollection::Array1,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::*, geom_adaptor::GeomAdaptorCurve};
use rustycad_tkmath::gp::{Axis2, Point3, dx};

pub fn curve_geometry(kind: i32) -> Result<GeomAdaptorCurve, Failure> {
    let curve: Handle<dyn GeomCurve> = match kind {
        1 => Handle::new(GeomCircle::new(Axis2::default(), 3.)?).cast(),
        2 => Handle::new(GeomEllipse::new(Axis2::default(), 5., 2.)?).cast(),
        3 => Handle::new(GeomBezierCurve::new(&Array1::from_shared(
            &[
                Point3::new(-4., -3., 0.),
                Point3::new(-2., 4., 0.),
                Point3::new(2., -4., 0.),
                Point3::new(4., 3., 0.),
            ],
            1,
        ))?)
        .cast(),
        4 => Handle::new(curve_spline()?).cast(),
        _ => Handle::new(GeomLine::from_point(Point3::new(-3., -2., 0.), dx())).cast(),
    };
    match kind {
        0 => GeomAdaptorCurve::with_range(&curve, 0., 6.),
        6 => GeomAdaptorCurve::with_range(&curve, -precision::infinite(), 3.),
        7 => GeomAdaptorCurve::with_range(&curve, 2., precision::infinite()),
        _ => GeomAdaptorCurve::with_curve(&curve),
    }
}
fn curve_spline() -> Result<GeomBSplineCurve, Failure> {
    let poles: Vec<Point3> = (0..10)
        .map(|i| Point3::new(f64::from(i) - 4.5, f64::from(i % 3) - 1., 0.))
        .collect();
    let knots: Vec<f64> = (0..10).map(f64::from).collect();
    let mults: Vec<i32> = (0..10)
        .map(|i| if i == 0 || i == 9 { 2 } else { 1 })
        .collect();
    GeomBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&knots, 1),
        &Array1::from_shared(&mults, 1),
        1,
        false,
    )
}
