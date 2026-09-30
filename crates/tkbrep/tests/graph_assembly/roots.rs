#[test]
fn BRepGraph_AssemblyTest__RootProductIds_Query() {
    let mut graph = assembly_box();
    assert_eq!(assembly_roots(&graph), vec![ProductId::start()]);
    let root = assembly_root(&mut graph);
    assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(assembly_roots(&graph), vec![root]);
}
#[test]
fn BRepGraph_AssemblyTest__RootProductIds_ReflectsAssemblyMutation() {
    let mut graph = assembly_box();
    assert_eq!(assembly_roots(&graph), vec![ProductId::start()]);
    let root = assembly_root(&mut graph);
    assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(assembly_roots(&graph), vec![root]);
}
#[test]
fn BRepGraph_AssemblyTest__RootProductIds_ShapelessRootAssembly_UsesProductId() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    assert!(assembly_origin(&mut graph, root, ProductId::start()).is_valid());
    assert_eq!(graph.root_product_ids(), &[root]);
}
#[test]
fn BRepGraph_AssemblyTest__RootProducts_RemovedOccurrence_DoesNotAffectRoots() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    let occurrence = assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(assembly_roots(&graph), vec![root]);
    assembly_remove(&mut graph, occurrence);
    let roots = assembly_roots(&graph);
    assert!(!roots.is_empty());
    assert!(roots.contains(&root));
}
#[test]
fn BRepGraph_AssemblyTest__RootProductRemovalCompactsInPlaceAfterRepeatedRemoves() {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let first = assembly_root(&mut graph);
    let second = assembly_root(&mut graph);
    let third = assembly_root(&mut graph);
    assert!(first.is_valid() && second.is_valid() && third.is_valid());
    assert_eq!(assembly_roots(&graph).len(), 3);
    assembly_remove_node(&mut graph, second);
    let roots = assembly_roots(&graph);
    assert_eq!(roots.len(), 2);
    assert!(roots.contains(&first));
    assert!(!roots.contains(&second));
    assert!(roots.contains(&third));
    let fourth = assembly_root(&mut graph);
    assert!(fourth.is_valid());
    assembly_remove_node(&mut graph, first);
    assembly_remove_node(&mut graph, fourth);
    assert_eq!(assembly_roots(&graph), vec![third]);
}
