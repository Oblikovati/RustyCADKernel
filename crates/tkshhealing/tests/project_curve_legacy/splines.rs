#[test]
fn BSplineOnPlane_B1() {
    let curve = planar_bspline();
    assert!(!curve.is_null());
    let pcurve = successful(whole(&fixture(&plane(0.), precision::confusion()), &curve));
    endpoints(
        &pcurve,
        (curve.first_parameter(), curve.last_parameter()),
        [0., 0., 10., 0.],
        0.1,
    );
}
#[test]
fn HelicalBSplineOnCylinder_B2() {
    let curve = helical_bspline(5., 9);
    assert!(!curve.is_null());
    let analysis = Handle::new(ShapeAnalysisSurface::new(&cylinder()).unwrap());
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init_analysis(&analysis, precision::confusion());
    let pcurve = successful(whole(&projector, &curve));
    verify_projection(&curve, &pcurve, &analysis, 0.1);
}
#[test]
fn HighDegreeBSpline_B3() {
    let points: Vec<Point3> = (1..=50)
        .map(|i| {
            let x = f64::from(i - 1) * 0.5;
            Point3::new(x, 5. * (x * PI / 10.).sin(), 0.)
        })
        .collect();
    let curve: Handle<dyn GeomCurve> = fit(&points, (3, 8)).cast();
    assert!(!curve.is_null());
    let analysis = Handle::new(ShapeAnalysisSurface::new(&plane(0.)).unwrap());
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init_analysis(&analysis, precision::confusion());
    let pcurve = successful(whole(&projector, &curve));
    verify_projection(&curve, &pcurve, &analysis, 0.5);
}
#[test]
fn DoublyTrimmedCurve_E2() {
    let curve = planar_bspline();
    assert!(!curve.is_null());
    let first = curve.first_parameter();
    let last = curve.last_parameter();
    let first_trim = trimmed(&curve, (first, (first + last) / 2.));
    let first = first_trim.first_parameter();
    let last = first_trim.last_parameter();
    let second_trim = trimmed(&first_trim, (first, first + (last - first) / 4.));
    successful(whole(
        &fixture(&plane(0.), precision::confusion()),
        &second_trim,
    ));
}
#[test]
fn DifferentEndpointTolerances_H2() {
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(5., 5., 0.05),
        Point3::new(10., 10., 0.1),
    ];
    let curve: Handle<dyn GeomCurve> = fit(&points, (2, 2)).cast();
    assert!(!curve.is_null());
    let projector = fixture(&plane(0.), precision::confusion());
    let mut pcurve = Handle::default();
    let done = projector
        .perform(
            &curve,
            (curve.first_parameter(), curve.last_parameter()),
            &mut pcurve,
            (0.01, 0.2),
        )
        .unwrap();
    successful((done, pcurve));
}
#[test]
fn MiddlePortionBSpline_I2() {
    let curve = planar_bspline();
    assert!(!curve.is_null());
    let first = curve.first_parameter();
    let range = curve.last_parameter() - first;
    successful(perform(
        &fixture(&plane(0.), precision::confusion()),
        &curve,
        (first + 0.25 * range, first + 0.75 * range),
    ));
}
