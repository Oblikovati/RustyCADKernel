fn count_subshapes(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut shapes = MapOfShape::default();
    for subshape in Explorer::new(shape, kind, ShapeEnum::Shape).unwrap() {
        shapes.add(subshape);
    }
    shapes.extent()
}
fn surface_area(shape: &Shape) -> f64 {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}
fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "{actual} expected {expected} +/-1e-12"
    );
}
fn valid(shape: &Shape) {
    assert!(
        BRepCheckAnalyzer::new(shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}
fn sew(shape: &Shape, tolerance: f64) -> Shape {
    ShapeUpgradeShellSewing::new()
        .apply_sewing(shape, tolerance)
        .unwrap()
}
fn counts(result: &Shape, faces: usize, edges: usize) {
    assert!(!result.is_null());
    assert_eq!(count_subshapes(result, ShapeEnum::Face), faces);
    assert_eq!(count_subshapes(result, ShapeEnum::Edge), edges);
}
fn solid_counts(result: &Shape, solids: usize, faces: usize) {
    assert!(!result.is_null());
    assert_eq!(count_subshapes(result, ShapeEnum::Solid), solids);
    assert_eq!(count_subshapes(result, ShapeEnum::Face), faces);
}
