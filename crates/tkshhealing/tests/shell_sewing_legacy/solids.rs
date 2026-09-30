#[test]
fn AutomaticTolerancePreservesValidBox() {
    let input = box_shape(0., (2., 3., 4.));
    let area = surface_area(&input);
    let result = sew(&input, 0.);
    solid_counts(&result, 1, 6);
    near(area, 52.);
    near(surface_area(&result), area);
    near(surface_area(&result), 52.);
    valid(&result);
}
#[test]
fn ExplicitTolerancePreservesValidBox() {
    let input = box_shape(0., (2., 3., 4.));
    let result = sew(&input, 1e-5);
    solid_counts(&result, 1, 6);
    valid(&result);
}
#[test]
fn ReorientsInsideOutSolid() {
    let input = box_shape(0., (2., 3., 4.)).reversed();
    let result = sew(&input, 1e-6);
    assert!(!result.is_null());
    assert_eq!(count_subshapes(&result, ShapeEnum::Solid), 1);
    let solid = Explorer::new(&result, ShapeEnum::Solid, ShapeEnum::Shape)
        .unwrap()
        .current()
        .clone();
    let mut classifier = BRepClass3dSolidClassifier::with_shape(&solid).unwrap();
    classifier.perform_infinite_point(1e-6).unwrap();
    assert_eq!(classifier.state(), State::Out);
}
#[test]
fn ProcessesMultipleShellsInCompound() {
    let input = compound(&[box_shape(0., (2., 3., 4.)), box_shape(10., (2., 3., 4.))]);
    let area = surface_area(&input);
    let result = sew(&input, 1e-6);
    solid_counts(&result, 2, 12);
    assert_eq!(count_subshapes(&result, ShapeEnum::Shell), 2);
    near(area, 104.);
    near(surface_area(&result), area);
    near(surface_area(&result), 104.);
    valid(&result);
}
#[test]
fn NegativeToleranceUsesAutomaticTolerance() {
    let input = box_shape(0., (2., 3., 4.));
    let result = sew(&input, -1.);
    solid_counts(&result, 1, 6);
    valid(&result);
}
