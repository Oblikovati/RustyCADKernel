#[test]
fn NullInputReturnsNull() {
    let result = sew(&Shape::default(), 0.);
    assert!(result.is_null());
}
#[test]
fn ShapeWithoutShellReturnsNull() {
    let result = sew(&rectangle(0., 1.), 1e-6);
    assert!(result.is_null());
}
#[test]
fn EmptyShellReturnsNull() {
    let result = sew(&shell(&[]), 1e-6);
    assert!(result.is_null());
}
#[test]
fn SewsOpenShellWithExplicitTolerance() {
    let input = open_shell(0.0001);
    let area = surface_area(&input);
    let result = sew(&input, 1e-3);
    counts(&result, 2, 7);
    near(area, 1.9999);
    near(surface_area(&result), area);
    near(surface_area(&result), 1.9999);
    valid(&result);
}
#[test]
fn SewsExactlyCoincidentOpenShell() {
    let result = sew(&open_shell(0.), 1e-7);
    counts(&result, 2, 7);
}
#[test]
fn KeepsEdgesBeyondExplicitToleranceSeparate() {
    let result = sew(&open_shell(0.01), 1e-4);
    counts(&result, 2, 8);
}
#[test]
fn CompoundWithoutShellReturnsNull() {
    let input = compound(&[rectangle(0., 1.), rectangle(2., 3.)]);
    assert!(sew(&input, 1e-6).is_null());
}
#[test]
fn PreservesFaceOutsideProcessedShell() {
    let input = compound(&[open_shell(0.), rectangle(10., 11.)]);
    let result = sew(&input, 1e-6);
    counts(&result, 3, 11);
}
#[test]
fn HandlesReversedOpenShell() {
    let input = open_shell(0.).reversed();
    let result = sew(&input, 1e-6);
    counts(&result, 2, 7);
    valid(&result);
}
#[test]
fn ProcessesSingleFaceShell() {
    let input = shell(&[rectangle(0., 1.)]);
    let result = sew(&input, 1e-6);
    counts(&result, 1, 4);
}
