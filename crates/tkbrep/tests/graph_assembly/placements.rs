#[test]
fn BRepGraph_AssemblyTest__MutOccurrenceRef_LocalLocation() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    assert!(assembly_origin(&mut graph, root, ProductId::start()).is_valid());
    let refs = graph.refs().occurrences().ids_of(root);
    assert_eq!(refs.len(), 1);
    let id = refs[0];
    {
        let mut guard = graph.editor().occurrences().mutate_ref(id).unwrap();
        OccurrenceOps::set_ref_local_location_guarded(
            &mut guard,
            &assembly_location(50.0, 0.0, 0.0),
        );
    }
    let location = &graph.refs().occurrences().entry(id).local_location;
    assembly_near(location.transformation().translation_part().x(), 50.0);
}
#[test]
fn BRepGraph_AssemblyTest__GlobalPlacement_DeepNesting() {
    let mut graph = assembly_box();
    let sub = assembly_root(&mut graph);
    let root = assembly_root(&mut graph);
    let links = [
        (root, sub, [0.0, 200.0, 0.0]),
        (sub, ProductId::start(), [100.0, 0.0, 0.0]),
    ];
    let occurrences = assembly_chain(&mut graph, &links);
    let (parent, child) = (occurrences[0], occurrences[1]);
    // Native OccurrenceLocation is local; explorers compose the parent chain.
    let part = assembly_local_translation(&graph, child);
    assembly_near(part.x(), 100.0);
    assembly_near(part.y(), 0.0);
    let sub = assembly_local_translation(&graph, parent);
    assembly_near(sub.x(), 0.0);
    assembly_near(sub.y(), 200.0);
}
#[test]
fn BRepGraph_AssemblyTest__GlobalPlacement_DAGSharing_DistinctPathsGiveDistinctPlacements() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    let first = assembly_x(&mut graph, root, ProductId::start(), 100.0);
    let second = assembly_link(
        &mut graph,
        root,
        ProductId::start(),
        &assembly_location(0.0, 200.0, 0.0),
        OccurrenceId::default(),
    );
    let first = assembly_local_translation(&graph, first);
    let second = assembly_local_translation(&graph, second);
    assembly_near(first.x(), 100.0);
    assembly_near(first.y(), 0.0);
    assembly_near(second.x(), 0.0);
    assembly_near(second.y(), 200.0);
}
#[test]
fn BRepGraph_AssemblyTest__GlobalPlacement_ThreeLevelNesting() {
    let mut graph = assembly_box();
    let middle = assembly_root(&mut graph);
    let root = assembly_root(&mut graph);
    let top = assembly_root(&mut graph);
    let links = [
        (top, root, [0.0, 0.0, 3.0]),
        (root, middle, [0.0, 2.0, 0.0]),
        (middle, ProductId::start(), [1.0, 0.0, 0.0]),
    ];
    let occurrences = assembly_chain(&mut graph, &links);
    let (root_occ, mid_occ, leaf_occ) = (occurrences[0], occurrences[1], occurrences[2]);
    assembly_near(assembly_local_translation(&graph, leaf_occ).x(), 1.0);
    assembly_near(assembly_local_translation(&graph, mid_occ).y(), 2.0);
    assembly_near(assembly_local_translation(&graph, root_occ).z(), 3.0);
}
#[test]
fn BRepGraph_AssemblyTest__OccurrenceLocation_AlwaysTerminates() {
    let mut graph = assembly_box();
    let root = assembly_root(&mut graph);
    let first = assembly_x(&mut graph, root, ProductId::start(), 1.0);
    let second = assembly_x(&mut graph, root, ProductId::start(), 1.0);
    assembly_near(assembly_local_translation(&graph, first).x(), 1.0);
    assert!(second.is_valid());
    assert!(
        !graph
            .topo()
            .occurrences()
            .occurrence_location(second)
            .is_identity()
    );
}
