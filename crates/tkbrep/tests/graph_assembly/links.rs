#[test]
fn BRepGraph_AssemblyTest__LinkProducts_LinksCorrectly() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    let occurrence = assembly_x(&mut graph, root, ProductId::start(), 100.0);
    assert!(occurrence.is_valid());
    assert_eq!(
        graph.topo().occurrences().product(occurrence),
        ProductId::start()
    );
    assert_eq!(graph.topo().occurrences().parent_product(occurrence), root);
    assert_eq!(graph.topo().products().nb_components(root), 1);
    assert_eq!(graph.topo().products().component(root, 0), occurrence);
}
#[test]
fn BRepGraph_AssemblyTest__DAGSharing_MultipleOccurrencesSamePart() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    let first = assembly_x(&mut graph, root, ProductId::start(), 100.0);
    let second = assembly_x(&mut graph, root, ProductId::start(), 200.0);
    assert_ne!(first, second);
    assert_eq!(
        graph.topo().occurrences().product(first),
        graph.topo().occurrences().product(second)
    );
    assert_eq!(graph.topo().products().nb_components(root), 2);
}
#[test]
fn BRepGraph_AssemblyTest__LinkProducts_ParentOccurrenceMustMatchParentProduct() {
    let mut graph = assembly_box();
    let first = assembly_root(&mut graph);
    let second = assembly_root(&mut graph);
    let context = assembly_origin(&mut graph, first, ProductId::start());
    assert!(context.is_valid());
    let invalid = assembly_link(
        &mut graph,
        second,
        ProductId::start(),
        &Location::default(),
        context,
    );
    assert!(!invalid.is_valid());
    assert_eq!(graph.topo().occurrences().nb(), 2);
    assert_eq!(graph.topo().products().nb_components(second), 0);
}
#[test]
fn BRepGraph_AssemblyTest__OccurrencesOfProduct_Relations() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    assembly_origin(&mut graph, root, ProductId::start());
    assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(graph.topo().products().nb_components(root), 2);
}
#[test]
fn BRepGraph_AssemblyTest__OccurrencesOfProduct_ViaRelations() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    assembly_origin(&mut graph, root, ProductId::start());
    assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(graph.topo().products().nb_components(root), 2);
    assert_eq!(graph.topo().products().nb_components(ProductId::start()), 1);
}
#[test]
fn BRepGraph_AssemblyTest__LinkProducts_InvalidParent_ReturnsInvalid() {
    let mut graph = assembly_box();
    assert!(!assembly_origin(&mut graph, ProductId::new(999), ProductId::start()).is_valid());
    assert!(!assembly_origin(&mut graph, ProductId::start(), ProductId::new(999)).is_valid());
}
#[test]
fn BRepGraph_AssemblyTest__LinkProducts_SelfReference_ReturnsInvalid() {
    let mut graph = assembly_box();
    assert!(!assembly_origin(&mut graph, ProductId::start(), ProductId::start()).is_valid());
}
#[test]
fn BRepGraph_AssemblyTest__LinkProducts_RemovedProduct_ReturnsInvalid() {
    let mut graph = assembly_box();
    let first = assembly_root(&mut graph);
    assembly_remove_node(&mut graph, first);
    assert!(!assembly_origin(&mut graph, first, ProductId::start()).is_valid());
    let second = assembly_root(&mut graph);
    assembly_remove_node(&mut graph, ProductId::start());
    assert!(!assembly_origin(&mut graph, second, ProductId::start()).is_valid());
}
