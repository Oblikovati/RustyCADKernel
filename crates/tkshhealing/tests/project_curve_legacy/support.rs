fn plane(z: f64) -> Handle<dyn GeomSurface> {
    Handle::new(
        GeomPlane::from_point(Point3::new(0., 0., z), Direction3::new(0., 0., 1.).unwrap())
            .unwrap(),
    )
    .cast()
}
fn frame(z: f64) -> Axis2 {
    Axis2::new(Point3::new(0., 0., z), Direction3::new(0., 0., 1.).unwrap()).unwrap()
}
fn segment(first: Point3, last: Point3) -> Handle<dyn GeomCurve> {
    MakeSegment::from_points(first, last)
        .unwrap()
        .value()
        .unwrap()
        .cast()
}
fn diagonal(z: f64) -> Handle<dyn GeomCurve> {
    segment(Point3::new(0., 0., z), Point3::new(10., 10., z))
}
fn circle(radius: f64, z: f64) -> Handle<dyn GeomCurve> {
    Handle::new(GeomCircle::new(frame(z), radius).unwrap()).cast()
}
fn cylinder() -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(Axis3::from_axis2(frame(0.)), 5.).unwrap()).cast()
}
fn sphere() -> Handle<dyn GeomSurface> {
    Handle::new(GeomSphericalSurface::new(Axis3::from_axis2(frame(0.)), 10.).unwrap()).cast()
}
fn fixture(
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
) -> ShapeConstructProjectCurveOnSurface {
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init_analysis(
        &Handle::new(ShapeAnalysisSurface::new(surface).unwrap()),
        tolerance,
    );
    projector
}
fn perform(
    projector: &ShapeConstructProjectCurveOnSurface,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> (bool, Handle<dyn Geom2dCurve>) {
    let mut result = Handle::default();
    let done = projector
        .perform(curve, range, &mut result, (-1., -1.))
        .unwrap();
    (done, result)
}
fn whole(
    projector: &ShapeConstructProjectCurveOnSurface,
    curve: &Handle<dyn GeomCurve>,
) -> (bool, Handle<dyn Geom2dCurve>) {
    perform(
        projector,
        curve,
        (curve.first_parameter(), curve.last_parameter()),
    )
}
fn successful(result: (bool, Handle<dyn Geom2dCurve>)) -> Handle<dyn Geom2dCurve> {
    assert!(result.0, "Projection should succeed");
    assert!(!result.1.is_null(), "PCurve should not be null");
    result.1
}
fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} expected {expected} +/- {tolerance}"
    );
}
fn endpoints(
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    expected: [f64; 4],
    tolerance: f64,
) {
    let first = curve.value(range.0).unwrap();
    let last = curve.value(range.1).unwrap();
    for (actual, expected) in [first.x(), first.y(), last.x(), last.y()]
        .into_iter()
        .zip(expected)
    {
        near(actual, expected, tolerance);
    }
}
fn fit(points: &[Point3], degrees: (i32, i32)) -> Handle<GeomBSplineCurve> {
    let array = Array1::from_shared(points, 1);
    PointsToBSpline::with_points(&array, degrees.0, degrees.1, Shape::C2, 1e-3)
        .unwrap()
        .curve()
        .unwrap()
}
fn planar_bspline() -> Handle<dyn GeomCurve> {
    fit(
        &[
            Point3::new(0., 0., 0.),
            Point3::new(2., 3., 0.),
            Point3::new(5., 4., 0.),
            Point3::new(8., 2., 0.),
            Point3::new(10., 0., 0.),
        ],
        (3, 8),
    )
    .cast()
}
fn helical_bspline(radius: f64, count: i32) -> Handle<dyn GeomCurve> {
    let points: Vec<Point3> = (1..=count)
        .map(|i| {
            let angle = f64::from(i - 1) * PI / 4.;
            Point3::new(radius * angle.cos(), radius * angle.sin(), f64::from(i - 1))
        })
        .collect();
    fit(&points, (3, 8)).cast()
}
fn verify_projection(
    curve: &Handle<dyn GeomCurve>,
    pcurve: &Handle<dyn Geom2dCurve>,
    surface: &ShapeAnalysisSurface,
    tolerance: f64,
) {
    let first = curve.first_parameter();
    let last = curve.last_parameter();
    for index in 0..=4 {
        let parameter = first + f64::from(index) * 0.25 * (last - first);
        let point = curve.value(parameter).unwrap();
        let uv = pcurve.value(parameter).unwrap();
        near(
            point.distance(&surface.value(uv.x(), uv.y()).unwrap()),
            0.,
            tolerance,
        );
    }
}
fn trimmed(curve: &Handle<dyn GeomCurve>, range: (f64, f64)) -> Handle<dyn GeomCurve> {
    Handle::new(GeomTrimmedCurve::new(curve, range.0, range.1, true, true).unwrap()).cast()
}
