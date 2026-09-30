fn dedup_graph(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(shape).unwrap();
    graph
}

fn dedup_compound(children: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for child in children {
        builder.add(&mut compound, child).unwrap();
    }
    compound.into()
}

fn dedup_copy(shape: &Shape) -> Shape {
    let mut copy = BRepBuilderAPICopy::with_shape(shape, true, false).unwrap();
    let result = copy.shape().unwrap();
    result.clone()
}

fn dedup_first(shape: &Shape, kind: ShapeEnum) -> Shape {
    let explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    assert!(explorer.more());
    explorer.current().clone()
}

fn dedup_box_face() -> Shape {
    let shape = support::make_box(10., 20., 30.);
    let kind = ShapeEnum::Face;
    dedup_first(&shape, kind)
}

fn makeNCopiedIdenticalFaces(count: u32) -> Shape {
    let face = dedup_box_face();
    let copies: Vec<Shape> = (0..count).map(|_| dedup_copy(&face)).collect();
    dedup_compound(&copies)
}

fn makeTwoCopiedIdenticalFaces() -> Shape {
    let face = dedup_box_face();
    let copies = [dedup_copy(&face), dedup_copy(&face)];
    dedup_compound(&copies)
}

fn dedup_sphere(radius: f64) -> Shape {
    let mut builder = BRepPrimAPIMakeSphere::new(radius).unwrap();
    let shape = builder.shape().unwrap();
    shape.clone()
}

fn dedup_cylinder(radius: f64, height: f64) -> Shape {
    let mut builder = BRepPrimAPIMakeCylinder::new(radius, height);
    let shape = builder.shape().unwrap();
    shape.clone()
}

fn makeMixedCompound() -> Shape {
    let face = dedup_box_face();
    let cylinder = dedup_cylinder(5., 15.);
    let curved = dedup_first(&cylinder, ShapeEnum::Face);
    dedup_compound(&[dedup_copy(&face), dedup_copy(&face), curved])
}

fn makeNestedCompound() -> Shape {
    let face = dedup_box_face();
    let inner = dedup_compound(&[dedup_copy(&face), dedup_copy(&face)]);
    dedup_compound(&[inner, dedup_copy(&face)])
}

fn makeThreeDistinctPrimitives() -> Shape {
    let box_shape = support::make_box(10., 20., 30.);
    let sphere = dedup_sphere(5.);
    let cone = BRepPrimAPIMakeCone::new(3., 1., 8.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    dedup_compound(&[box_shape.into(), sphere, cone])
}

fn makeTwoIdenticalBoxes() -> Shape {
    let first = support::make_box(10., 20., 30.);
    let second = support::make_box(10., 20., 30.);
    dedup_compound(&[first.into(), second.into()])
}

fn makeThreeCopiedIdenticalEdges() -> Shape {
    let box_shape = support::make_box(10., 20., 30.);
    let edge = dedup_first(&box_shape, ShapeEnum::Edge);
    dedup_compound(&[edge.clone(), dedup_copy(&edge), dedup_copy(&edge)])
}

fn makeTwoFaceTouchingBoxes() -> Shape {
    let first = support::make_box_at(Point3::new(0., 0., 0.), 10., 10., 10.);
    let second = support::make_box_at(Point3::new(0., 0., 10.), 10., 10., 10.);
    dedup_compound(&[first.into(), second.into()])
}

fn makeThreeCopiedBoxes() -> Shape {
    let boxes: Vec<Shape> = (0..3)
        .map(|_| support::make_box(10., 20., 30.).into())
        .collect();
    dedup_compound(&boxes)
}

fn dedup_copied_curved_faces(shape: &Shape, count: u32) -> Shape {
    let face = dedup_first(shape, ShapeEnum::Face);
    let copies: Vec<Shape> = (0..count).map(|_| dedup_copy(&face)).collect();
    dedup_compound(&copies)
}

fn dedup_history(graph: &BRepGraph) -> Handle<LayerHistory> {
    let registry = graph.layer_registry();
    registry.ensure::<LayerHistory>().unwrap()
}
