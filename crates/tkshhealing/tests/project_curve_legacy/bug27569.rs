#[test]
fn Bug27569_ManyKnotsBSpline_M1() {
    let curve = bug27569_uniform_curve();
    assert!(!curve.is_null());
    assert!(curve.nb_knots() >= 10);
    let surface = Handle::new(
        GeomPlane::from_point(
            Point3::new(0., -15., 0.),
            Direction3::new(0., 1., 0.).unwrap(),
        )
        .unwrap(),
    )
    .cast();
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init(&surface, precision::confusion()).unwrap();
    successful(whole(&projector, &curve.cast()));
}
#[test]
fn Bug27569_HighMultiplicityBSpline_M2() {
    // Native construction succeeds at the pinned source: its guarded skip and plane fallback are not taken.
    let curve = bug27569_degenerate_curve();
    assert!(!curve.is_null());
    let surface = bug27569_surface();
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init(&surface, precision::confusion()).unwrap();
    successful(whole(&projector, &curve));
}
fn bug27569_degenerate_curve() -> Handle<dyn GeomCurve> {
    let mut points = Array1::with_bounds(1, 43).unwrap();
    for index in 1..=15 {
        let t = (f64::from(index) - 1.) / 14.;
        points[index] = Point3::new(144. - t * 74., -t * 0.0002, 8.3 + t * 51.7);
    }
    for index in 16..=28 {
        points[index] = Point3::new(70., 0., 60.);
    }
    for index in 29..=43 {
        let t = (f64::from(index) - 29.) / 14.;
        points[index] = Point3::new(70. - t * 80., -t * 32., 60. - t * 60.);
    }
    let knots = Array1::from_shared(&[0., 1., 2., 3., 4., 5., 6.], 1);
    let mults = Array1::from_shared(&[8, 7, 7, 7, 7, 7, 8], 1);
    Handle::new(GeomBSplineCurve::new(&points, &knots, &mults, 7, false).unwrap()).cast()
}
fn bug27569_surface() -> Handle<dyn GeomSurface> {
    let mut poles = Array2::with_bounds(1, 8, 1, 2).unwrap();
    for (v, row) in BUG27569_SURFACE_ROWS.into_iter().enumerate() {
        for (u, (x, y, z)) in row.into_iter().enumerate() {
            poles.set_value(u as i32 + 1, v as i32 + 1, Point3::new(x, y, z)).unwrap();
        }
    }
    let knots = Array1::from_shared(&[0., 1.], 1);
    let umults = Array1::from_shared(&[8, 8], 1);
    let vmults = Array1::from_shared(&[2, 2], 1);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &umults),
            (&knots, &vmults),
            (7, 1),
            (false, false),
        )
        .unwrap(),
    )
    .cast()
}

fn bug27569_uniform_curve() -> Handle<GeomBSplineCurve> {
    let points = bug27569_uniform_poles();
    let knots: Vec<f64> = (1..=15).map(|i| (f64::from(i) - 1.) / 14.).collect();
    let mut mults = vec![1; 15];
    mults[0] = 4;
    mults[14] = 4;
    let curve = GeomBSplineCurve::new(
        &Array1::from_shared(&points, 1),
        &Array1::from_shared(&knots, 1),
        &Array1::from_shared(&mults, 1),
        3,
        false,
    )
    .unwrap();
    Handle::new(curve)
}

const BUG27569_SURFACE_ROWS: [[(f64, f64, f64); 8]; 2] = [
    [
        (70., 0., 60.),
        (85.01, 0., 60.),
        (98.87, 0., 55.07),
        (111.6, 0., 46.57),
        (123.15, 0., 35.68),
        (133.46, 0., 23.56),
        (142.46, 0., 11.32),
        (150., 0., 0.),
    ],
    [
        (54.99, 0., 60.),
        (41.13, 0., 55.07),
        (28.4, 0., 46.57),
        (16.85, 0., 35.68),
        (6.54, 0., 23.56),
        (-2.46, 0., 11.32),
        (-10., 0., 0.),
        (-10., -35.91, 0.),
    ],
];

fn bug27569_uniform_poles() -> Vec<Point3> {
    (1..=17)
        .map(|i| {
            let t = (f64::from(i) - 1.) / 16.;
            let angle = t * 2. * PI;
            let radius = 50. + 20. * (3. * angle).cos();
            Point3::new(radius * angle.cos(), -30. * t, radius * angle.sin())
        })
        .collect()
}
