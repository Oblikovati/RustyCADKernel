#[test]
fn NearPoleOnSphere_K1() {
    let latitude = PI / 2. - 0.01;
    let curve = circle(10. * latitude.cos(), 10. * latitude.sin());
    successful(perform(
        &fixture(&sphere(), precision::confusion()),
        &curve,
        (0., 2. * PI),
    ));
}
#[test]
fn VeryShortCurve_L1() {
    let curve = segment(Point3::default(), Point3::new(1e-6, 0., 0.));
    // Native SUCCEED only asserts completion without an exception.
    let _ = whole(&fixture(&plane(0.), 1e-4), &curve);
}
#[test]
fn CurveFarFromSurface_L2() {
    let (_, pcurve) = whole(
        &fixture(&plane(0.), precision::confusion()),
        &diagonal(100.),
    );
    assert!(!pcurve.is_null());
}
#[test]
fn LargeTolerance_L3() {
    let projector = fixture(&plane(0.), 1.);
    successful(whole(&projector, &diagonal(0.1)));
}
