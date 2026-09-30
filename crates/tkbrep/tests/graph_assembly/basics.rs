#[test]
fn BRepGraph_AssemblyTest__Build_SingleSolid_AutoCreatesRootProduct() {
    let graph = assembly_source(&support::make_box(10.0, 20.0, 30.0));
    assert_eq!(graph.topo().products().nb(), 1);
    assert_eq!(graph.topo().occurrences().nb(), 1);
    let _definition = graph.topo().products().definition(ProductId::start());
    assert!(
        graph
            .topo()
            .products()
            .shape_root(ProductId::start())
            .is_valid()
    );
    assert!(graph.topo().products().is_part(ProductId::start()));
    assert!(!graph.topo().products().is_assembly(ProductId::start()));
}
#[test]
fn BRepGraph_AssemblyTest__Build_Compound_AutoCreatesRootProduct() {
    let compound = assembly_compound(&[
        support::make_box(10.0, 10.0, 10.0).into_shape(),
        assembly_sphere(),
    ]);
    let graph = assembly_source(&compound);
    assert_eq!(graph.topo().products().nb(), 1);
    assert_eq!(graph.topo().occurrences().nb(), 1);
    let root = graph.topo().products().shape_root(ProductId::start());
    assert!(root.is_valid());
    assert_eq!(root.kind, NodeKind::COMPOUND);
}
#[test]
fn BRepGraph_AssemblyTest__AddProduct_IsPart() {
    let mut graph = assembly_box();
    let id = graph
        .editor()
        .products()
        .add(SolidId::start().into(), &Location::default());
    graph.editor().products().append_document_root(id).unwrap();
    assert!(id.is_valid());
    assert!(graph.topo().products().is_part(id));
    assert!(!graph.topo().products().is_assembly(id));
}
#[test]
fn BRepGraph_AssemblyTest__AddProduct_InvalidShapeRoot_ReturnsInvalid() {
    let mut graph = assembly_box();
    assert!(
        !graph
            .editor()
            .products()
            .add(ProductId::start().into(), &Location::default())
            .is_valid()
    );
    assembly_remove_node(&mut graph, SolidId::start());
    assert!(
        !graph
            .editor()
            .products()
            .add(SolidId::start().into(), &Location::default())
            .is_valid()
    );
}
#[test]
fn BRepGraph_AssemblyTest__CreateEmptyProduct_EmptyIsNotAssemblyYet() {
    let mut graph = assembly_box();
    let id = assembly_root(&mut graph);
    assert!(id.is_valid());
    assert!(!graph.topo().products().is_assembly(id));
    assert!(!graph.topo().products().is_part(id));
}
#[test]
fn BRepGraph_AssemblyTest__NbNodes_IncludesAssembly() {
    let mut graph = assembly_box();
    let before = graph.topo().generic().nb_nodes();
    assert!(before >= 1);
    let root = assembly_root(&mut graph);
    assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(graph.topo().generic().nb_nodes(), before + 2);
}
#[test]
fn BRepGraph_AssemblyTest__Product_Count() {
    let mut graph = assembly_box();
    assembly_root(&mut graph);
    assert_eq!(DefIterator::<ProductTag>::new(&graph).count(), 2);
}
#[test]
fn BRepGraph_AssemblyTest__Occurrence_Count() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    assembly_origin(&mut graph, root, ProductId::start());
    assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(DefIterator::<OccurrenceTag>::new(&graph).count(), 3);
}
#[test]
fn BRepGraph_AssemblyTest__NodeId_Helpers() {
    assert!(NodeKind::is_topology_kind(NodeKind::SOLID));
    assert!(NodeKind::is_topology_kind(NodeKind::VERTEX));
    assert!(NodeKind::is_topology_kind(NodeKind::COMP_SOLID));
    assert!(!NodeKind::is_topology_kind(NodeKind::PRODUCT));
    assert!(!NodeKind::is_topology_kind(NodeKind::OCCURRENCE));
    assert!(NodeKind::is_assembly_kind(NodeKind::PRODUCT));
    assert!(NodeKind::is_assembly_kind(NodeKind::OCCURRENCE));
    assert!(!NodeKind::is_assembly_kind(NodeKind::SOLID));
    assert!(!NodeKind::is_assembly_kind(NodeKind::FACE));
}
#[test]
fn BRepGraph_AssemblyTest__UID_IsAssembly() {
    let graph = assembly_box();
    // UID classifies the Product kind even when this product is a part.
    let uid = graph.uids().of_node(ProductId::start().into());
    assert!(uid.is_valid());
    assert!(uid.is_assembly());
    assert!(!uid.is_topology());
}
#[test]
fn BRepGraph_AssemblyTest__MutProduct_RAII() {
    let mut graph = assembly_box();
    {
        let mut guard = graph
            .editor()
            .products()
            .mutate(ProductId::start())
            .unwrap();
        guard.mark_dirty();
    }
    assert!(
        graph
            .topo()
            .products()
            .definition(ProductId::start())
            .base
            .own_gen
            > 0
    );
}
#[test]
fn BRepGraph_AssemblyTest__MutInvalidAssemblyDefs_ThrowProgramError() {
    let mut graph = BRepGraph::new();
    let error = graph
        .editor()
        .products()
        .mutate(ProductId::new(7))
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    let error = graph
        .editor()
        .occurrences()
        .mutate(OccurrenceId::new(7))
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}
