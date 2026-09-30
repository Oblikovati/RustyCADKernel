#[test]
fn BRepGraph_BuildTest__Compound_TwoPrimitives_IsNotEmpty() {
    let shape = build_compound(&[build_box(10.0, 10.0, 10.0), build_sphere(5.0)]);
    let _graph = build_graph(&shape);
}
#[test]
fn BRepGraph_BuildTest__Compound_TwoPrimitives_DefCountsAddUp() {
    let shape = build_compound(&[build_box(10.0, 10.0, 10.0), build_sphere(5.0)]);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex],
    );
}
#[test]
fn BRepGraph_BuildTest__Compound_ThreeBoxes_DefCounts() {
    let boxes: Vec<Shape> = (1..=3)
        .map(|i| build_box(5.0 * f64::from(i), 10.0, 10.0))
        .collect();
    let shape = build_compound(&boxes);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[ShapeEnum::Solid, ShapeEnum::Face, ShapeEnum::Edge],
    );
}
#[test]
fn BRepGraph_BuildTest__Compound_Nested_DefCounts() {
    let inner = build_compound(&[build_box(10.0, 10.0, 10.0)]);
    let cylinder = build_shape(&mut BRepPrimAPIMakeCylinder::new(3.0, 10.0));
    let shape = build_compound(&[inner, cylinder]);
    let graph = build_graph(&shape);
    assert_build_counts(
        &graph,
        &shape,
        &[ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex],
    );
}
