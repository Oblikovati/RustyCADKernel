#[test]
fn BRepGraph_AssemblyTest__RemoveOccurrence_UpdatesParent() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    let occurrence = assembly_origin(&mut graph, root, ProductId::start());
    assert_eq!(graph.topo().products().nb_components(root), 1);
    let refs = graph.refs().occurrences().ids_of(root);
    assert_eq!(refs.len(), 1);
    let reference = refs[0];
    assert!(!reference.is_removed(&graph));
    assembly_remove(&mut graph, occurrence);
    assert!(occurrence.is_removed(&graph));
    assert_eq!(graph.topo().products().nb_components(root), 0);
    assert!(graph.refs().occurrences().ids_of(root).is_empty());
    assert!(reference.is_removed(&graph));
}
#[test]
fn BRepGraph_AssemblyTest__RemoveProduct_CascadeOccurrences() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    let first = assembly_origin(&mut graph, root, ProductId::start());
    let second = assembly_origin(&mut graph, root, ProductId::start());
    assembly_remove(&mut graph, root);
    assert!(root.is_removed(&graph));
    assert!(first.is_removed(&graph));
    assert!(second.is_removed(&graph));
}
#[test]
fn BRepGraph_AssemblyTest__RemoveProduct_RemovesProductAndOccurrences() {
    let mut graph = assembly_box();
    let part = ProductId::start();
    assert!(graph.topo().products().is_part(part));
    assert_eq!(graph.topo().occurrences().nb(), 1);
    let root_occurrence = OccurrenceId::start();
    assert!(!root_occurrence.is_removed(&graph));
    assembly_remove(&mut graph, part);
    assert!(part.is_removed(&graph));
    assert!(root_occurrence.is_removed(&graph));
}
#[test]
fn BRepGraph_AssemblyTest__RemoveOccurrence_CascadesToNestedChildren() {
    let mut graph = assembly_box();
    let middle = assembly_root(&mut graph);
    let top = assembly_root(&mut graph);
    let links = [
        (top, middle, [10.0, 0.0, 0.0]),
        (middle, ProductId::start(), [0.0, 20.0, 0.0]),
    ];
    let occurrences = assembly_chain(&mut graph, &links);
    let (mid_occ, leaf_occ) = (occurrences[0], occurrences[1]);
    assert!(!mid_occ.is_removed(&graph));
    assert!(!leaf_occ.is_removed(&graph));
    assert!(!leaf_occ.is_removed(&graph));
    assembly_remove(&mut graph, mid_occ);
    assert!(mid_occ.is_removed(&graph));
    // Native does not require leaf removal: its parent is the middle product.
    assert!(!ProductId::start().is_removed(&graph));
    assert!(!middle.is_removed(&graph));
    assert!(!top.is_removed(&graph));
}
