fn assembly_bounds(shape: &Shape) -> [f64; 6] {
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add_optimal(shape, &mut bounds, false, false).unwrap();
    bounds.get().unwrap()
}
fn assembly_bounds_near(actual: &Shape, expected: &Shape) {
    for (actual, expected) in assembly_bounds(actual)
        .into_iter()
        .zip(assembly_bounds(expected))
    {
        assert!(
            (actual - expected).abs() <= 1e-7,
            "bound {actual}, expected {expected} within 1e-7"
        );
    }
}
fn assembly_root_location(graph: &BRepGraph, product: ProductId) -> Location {
    let id = graph
        .topo()
        .products()
        .relations(product)
        .occurrence_ref_ids[0];
    graph.refs().occurrences().entry(id).local_location.clone()
}
fn assembly_import_bounds(
    graph: &BRepGraph,
    result: &rustycad_tkbrep::brep_graph::shapes_view::AddResult,
    placed: &Shape,
    local: &Shape,
) {
    let product = assembly_shape(graph, result.product);
    assembly_bounds_near(&product, placed);
    let topology = assembly_shape(graph, result.topology_root);
    assert!(topology.location().is_identity());
    assembly_bounds_near(&topology, local);
}
#[test]
fn BRepGraph_AssemblyTest__Add_RootProduct_PreservesShapeLocation() {
    let mut graph = BRepGraph::new();
    let local = support::make_box(10.0, 20.0, 30.0).into_shape();
    let mut placed = local.clone();
    placed
        .set_location(&assembly_location(5.0, 6.0, 7.0), true)
        .unwrap();
    let result = graph.shapes_mut().add(&placed).unwrap();
    assert!(result.is_ok());
    assert!(result.product.is_valid());
    assert!(result.topology_root.is_valid());
    assert!(result.occurrence.is_valid());
    assert_eq!(graph.shapes().find_node(&placed), result.topology_root);
    assert_eq!(graph.root_product_ids(), &[result.product]);
    assert!(assembly_root_location(&graph, result.product).is_equal(placed.location()));
    assembly_import_bounds(&graph, &result, &placed, &local);
}
#[test]
fn BRepGraph_AssemblyTest__Add_NoAutoProduct_TopologyOnly() {
    let mut graph = BRepGraph::new();
    let options = Options {
        create_auto_product: false,
        ..Options::default()
    };
    let shape = support::make_box(10.0, 10.0, 10.0);
    let result = graph
        .shapes_mut()
        .add_with_options(&shape, &options)
        .unwrap();
    assert!(result.is_ok());
    assert!(!result.product.is_valid());
    assert!(!result.occurrence.is_valid());
    assert!(result.topology_root.is_valid());
    assert!(graph.root_product_ids().is_empty());
}
#[test]
fn BRepGraph_AssemblyTest__Add_NullShape_ReturnsInvalidResult() {
    let mut graph = BRepGraph::new();
    let result = graph.shapes_mut().add(&Shape::default()).unwrap();
    assert!(!result.is_ok());
    assert!(!result.product.is_valid());
    assert!(!result.topology_root.is_valid());
}
#[test]
fn BRepGraph_AssemblyTest__Add_ProductParent_CreatesChildPartAndOccurrence() {
    let mut graph = BRepGraph::new();
    let parent = assembly_root(&mut graph);
    assert!(parent.is_valid());
    let mut sphere = assembly_sphere();
    sphere
        .set_location(&assembly_location(2.0, 0.0, 0.0), true)
        .unwrap();
    let result = graph
        .shapes_mut()
        .add_under(&sphere, parent.into())
        .unwrap();
    assert!(result.is_ok());
    assert!(result.topology_root.is_valid());
    assert!(result.product.is_valid());
    assert!(result.occurrence.is_valid());
    assert_ne!(result.product, parent);
    let refs = &graph.topo().products().relations(parent).occurrence_ref_ids;
    assert_eq!(refs.len(), 1);
    assert!(assembly_root_location(&graph, parent).is_equal(sphere.location()));
}
fn assembly_first_box_face() -> Shape {
    let shape = support::make_box(2.0, 2.0, 2.0);
    let explorer = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let face = if explorer.more() {
        explorer.current().clone()
    } else {
        Shape::default()
    };
    assert!(!face.is_null());
    face
}
#[test]
fn BRepGraph_AssemblyTest__Add_CompoundParent_AppendsAsChild() {
    let mut graph = BRepGraph::new();
    let compound = assembly_compound(&[support::make_box(1.0, 1.0, 1.0).into_shape()]);
    let root = graph.shapes_mut().add(&compound).unwrap();
    assert!(root.is_ok());
    assert_eq!(root.topology_root.kind, NodeKind::COMPOUND);
    let id = CompoundId::new(root.topology_root.index);
    let before = graph.topo().compounds().relations(id).child_ref_ids.len();
    let child = graph
        .shapes_mut()
        .add_under(&assembly_first_box_face(), id.into())
        .unwrap();
    assert!(child.is_ok());
    assert!(child.inserted_ref.is_valid());
    assert_eq!(
        graph.topo().compounds().relations(id).child_ref_ids.len(),
        before + 1
    );
}
#[test]
fn BRepGraph_AssemblyTest__Add_InvalidParent_ReturnsInvalidResult() {
    let mut graph = BRepGraph::new();
    graph
        .shapes_mut()
        .add(&support::make_box(1.0, 1.0, 1.0))
        .unwrap();
    let shape = support::make_box(2.0, 2.0, 2.0);
    let result = graph
        .shapes_mut()
        .add_under(&shape, NodeId::default())
        .unwrap();
    assert!(!result.is_ok());
}
