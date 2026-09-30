#[test]
fn PerpendicularCylinderBoundaryTouch_OrderIndependent() {
    let first = cylinder(Point3::default(), Direction3::Z, 2., (-5., 5.), (0., TAU));
    let second = cylinder(
        Point3::new(0., 2., 0.),
        Direction3::X,
        0.5,
        (0., 4.),
        (0., TAU),
    );
    let forward = intersect(&first, &second, [true; 3]);
    let reverse = intersect(&second, &first, [true; 3]);
    assert_eq!(
        forward.lines().unwrap().len(),
        reverse.lines().unwrap().len()
    );
    assert_eq!(forward.points().len(), reverse.points().len());
}
#[test]
fn HalfCylinderOutsideCircularPlane_NoIntersection() {
    let plane = circular_plane(10.);
    let cylinder = cylinder(
        Point3::new(20., 0., 0.),
        Direction3::Y,
        2.,
        (-1., 1.),
        (0., PI),
    );
    let intersection = intersect(&plane, &cylinder, [true; 3]);
    assert_eq!(intersection.lines().unwrap().len(), 0);
    assert_eq!(intersection.points().len(), 0);
}
#[test]
fn HalfCylinderInsideCircularPlane_HasIntersection() {
    let plane = circular_plane(20.);
    let cylinder = cylinder(Point3::default(), Direction3::Y, 5., (-1., 1.), (0., PI));
    let intersection = intersect(&plane, &cylinder, [true; 3]);
    assert!(!intersection.lines().unwrap().is_empty());
}
#[test]
fn OppositeHalfInsideCircle_HasIntersection() {
    let plane = circular_plane(15.);
    let cylinder = cylinder(
        Point3::new(10., 0., -5.),
        Direction3::Y,
        3.,
        (-1., 1.),
        (PI, TAU),
    );
    let intersection = intersect(&plane, &cylinder, [true; 3]);
    assert!(!intersection.lines().unwrap().is_empty());
}
#[test]
fn PartialCrossing_ProperlyTrimmed() {
    let plane = circular_plane(15.);
    let cylinder = cylinder(
        Point3::new(12., 0., -8.),
        Direction3::Y,
        3.,
        (-1., 1.),
        (0., PI),
    );
    let intersection = intersect(&plane, &cylinder, [true; 3]);
    assert!(!intersection.lines().unwrap().is_empty());
}
#[test]
fn BothHalvesCompletelyOutside_NoIntersection() {
    let plane = circular_plane(5.);
    let center = Point3::new(20., 0., 20.);
    let first = cylinder(center, Direction3::Y, 2., (-1., 1.), (0., PI));
    let intersection = intersect(&plane, &first, [true; 3]);
    assert_eq!(intersection.lines().unwrap().len(), 0);
    assert_eq!(intersection.points().len(), 0);
    let second = cylinder(center, Direction3::Y, 2., (-1., 1.), (PI, TAU));
    let intersection = intersect(&plane, &second, [true; 3]);
    assert_eq!(intersection.lines().unwrap().len(), 0);
    assert_eq!(intersection.points().len(), 0);
}
