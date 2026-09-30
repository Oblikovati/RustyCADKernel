#[test]
fn LineOnPlane_A1() {
    let projector = fixture(&plane(0.), precision::confusion());
    let line = diagonal(0.);
    let range = (line.first_parameter(), line.last_parameter());
    let pcurve = successful(perform(&projector, &line, range));
    endpoints(&pcurve, range, [0., 0., 10., 10.], 0.01);
}
#[test]
fn CircleOnCylinder_A2() {
    let projector = fixture(&cylinder(), precision::confusion());
    let pcurve = successful(perform(&projector, &circle(5., 3.), (0., 2. * PI)));
    near(pcurve.value(PI).unwrap().y(), 3., 0.01);
}
#[test]
fn IsoparametricLineOnCylinder_A3() {
    let projector = fixture(&cylinder(), precision::confusion());
    let line = segment(Point3::new(5., 0., 0.), Point3::new(5., 0., 10.));
    let pcurve = successful(whole(&projector, &line));
    near(
        pcurve.value(line.first_parameter()).unwrap().x(),
        pcurve.value(line.last_parameter()).unwrap().x(),
        0.01,
    );
}
#[test]
fn EquatorOnSphere_C1() {
    let projector = fixture(&sphere(), precision::confusion());
    successful(perform(&projector, &circle(10., 0.), (0., 2. * PI)));
}
#[test]
fn LatitudeOnSphere_C2() {
    let projector = fixture(&sphere(), precision::confusion());
    let latitude = PI / 4.;
    let curve = circle(10. * latitude.cos(), 10. * latitude.sin());
    let pcurve = successful(perform(&projector, &curve, (0., 2. * PI)));
    near(
        pcurve.value(0.).unwrap().y(),
        pcurve.value(PI).unwrap().y(),
        0.01,
    );
}
#[test]
fn CircleOnTorus_C3() {
    let torus =
        Handle::new(GeomToroidalSurface::new(Axis3::from_axis2(frame(0.)), 10., 2.).unwrap())
            .cast();
    // Original circle center is major+minor radius, not the torus section center.
    let axis = Axis2::new(
        Point3::new(12., 0., 0.),
        Direction3::new(0., 1., 0.).unwrap(),
    )
    .unwrap();
    let curve = Handle::new(GeomCircle::new(axis, 2.).unwrap()).cast();
    successful(perform(
        &fixture(&torus, precision::confusion()),
        &curve,
        (0., 2. * PI),
    ));
}
#[test]
fn CircleOnCone_D1() {
    let semi_angle = PI / 6.;
    let cone =
        Handle::new(GeomConicalSurface::new(Axis3::from_axis2(frame(0.)), semi_angle, 5.).unwrap())
            .cast();
    let curve = circle(5. + 5. * semi_angle.tan(), 5.);
    successful(perform(
        &fixture(&cone, precision::confusion()),
        &curve,
        (0., 2. * PI),
    ));
}
#[test]
fn ArcOnPlane_E1() {
    let arc = trimmed(&circle(5., 0.), (0., PI / 2.));
    let pcurve = successful(whole(&fixture(&plane(0.), precision::confusion()), &arc));
    endpoints(
        &pcurve,
        (arc.first_parameter(), arc.last_parameter()),
        [5., 0., 0., 5.],
        0.01,
    );
}
#[test]
fn EllipseOnPlane_F1() {
    let ellipse = Handle::new(GeomEllipse::new(frame(0.), 10., 5.).unwrap()).cast();
    let pcurve = successful(perform(
        &fixture(&plane(0.), precision::confusion()),
        &ellipse,
        (0., 2. * PI),
    ));
    endpoints(&pcurve, (0., PI / 2.), [10., 0., 0., 5.], 0.01);
}
#[test]
fn PartialCircle_I1() {
    let pcurve = successful(perform(
        &fixture(&plane(0.), precision::confusion()),
        &circle(5., 0.),
        (0., PI / 2.),
    ));
    endpoints(&pcurve, (0., PI / 2.), [5., 0., 0., 5.], 0.01);
}
