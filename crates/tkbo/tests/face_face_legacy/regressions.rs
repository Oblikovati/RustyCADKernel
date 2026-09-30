#[test]
fn OCC24005_PlaneCylinderIntersection() {
    let origin = Point3::new(-72.948737453424499, 754.30437716359393, 259.52151854671678);
    let normal = Direction3::new(6.2471473085930200e-7, -0.99999999999980493, 0.).unwrap();
    let x = Direction3::new(0.99999999999980493, 6.2471473085930200e-7, 0.).unwrap();
    let plane =
        bare_face(Handle::new(GeomPlane::new(Axis3::new(origin, normal, x).unwrap())).into_cast());
    let origin = Point3::new(-6.4812490053250649, 753.39408794522092, 279.16400974257465);
    let frame = Axis3::new(origin, Direction3::X, Direction3::Y).unwrap();
    let cylinder = bare_face(
        Handle::new(GeomCylindricalSurface::new(frame, 19.712534607908712).unwrap()).into_cast(),
    );
    let intersection = intersect(&plane, &cylinder, [false, true, true]);
    assert!(intersection.lines().unwrap().len() + intersection.points().len() >= 1);
}
#[test]
fn OCC32850_TrimmedCylindersKeepFourCurves() {
    let (first, second) = occ32850_cylinders();
    let mut intersection = intersect(&first, &second, [true, false, false]);
    intersection.prepare_lines_3d(false).unwrap();
    assert_eq!(intersection.lines().unwrap().len(), 4);
    let mut maximum_tolerance = 0_f64;
    for curve in intersection.lines().unwrap() {
        assert!(!curve.curve().is_null());
        maximum_tolerance = maximum_tolerance.max(curve.tolerance());
    }
    assert!(maximum_tolerance <= 1e-12);
}
#[test]
fn OCC24313_ExtrusionAndPlaneIntersection() {
    let circle: Handle<dyn GeomCurve> = Handle::new(
        GeomCircle::new(
            Axis2::with_x_direction(Point3::default(), Direction3::Z, Direction3::X).unwrap(),
            100.,
        )
        .unwrap(),
    )
    .into_cast();
    let extrusion = bare_face(
        Handle::new(GeomSurfaceOfLinearExtrusion::new(&circle, Direction3::Z).unwrap()).into_cast(),
    );
    let plane = bare_face(
        Handle::new(GeomPlane::from_point(Point3::default(), Direction3::X).unwrap()).into_cast(),
    );
    let mut intersection = intersect(&extrusion, &plane, [true, false, false]);
    intersection.prepare_lines_3d(false).unwrap();
    assert!(intersection.lines().unwrap().len() + intersection.points().len() > 0);
}
#[test]
fn OCC23644_CoaxialConesNoRegularCurve() {
    let first = cone_first_face(10.);
    let second = cone_first_face(20.);
    let mut intersection = intersect(&first, &second, [true, false, false]);
    intersection.prepare_lines_3d(false).unwrap();
    assert_eq!(intersection.lines().unwrap().len(), 0);
    assert_eq!(intersection.points().len(), 1);
    assert!(
        intersection.points()[0]
            .p1()
            .pnt()
            .is_equal(&Point3::new(0., 0., 20.), 1e-7)
    );
}
