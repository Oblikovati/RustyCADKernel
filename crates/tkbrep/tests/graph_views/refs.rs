#[test]
fn BRepGraph_ViewsTest__RefsView_ActiveCounts_MatchFreshBuild() {
    let graph = views_box();
    let refs = graph.refs();
    assert_eq!(refs.shells().nb_active(), refs.shells().nb());
    assert_eq!(refs.faces().nb_active(), refs.faces().nb());
    assert_eq!(refs.wires().nb_active(), refs.wires().nb());
    assert_eq!(
        graph.topo().co_edges().nb_active(),
        graph.topo().co_edges().nb()
    );
    assert_eq!(refs.vertices().nb_active(), refs.vertices().nb());
    assert_eq!(refs.solids().nb_active(), refs.solids().nb());
    assert_eq!(refs.children().nb_active(), refs.children().nb());
    assert_eq!(refs.occurrences().nb_active(), refs.occurrences().nb());
}
#[test]
fn BRepGraph_ViewsTest__RefsView_RefIdsOf_MatchFreshBuild() {
    let graph = views_box();
    let refs = graph.refs();
    let faces = refs.faces().ids_of(ShellId::new(0));
    let wires = refs.wires().ids_of(FaceId::new(0));
    let coedges = &graph.topo().wires().relations(WireId::new(0)).co_edge_ids;
    let shells = refs.shells().ids_of(SolidId::new(0));
    assert_eq!(views_active_refs(&graph, faces), faces.len());
    assert_eq!(views_active_refs(&graph, wires), wires.len());
    assert_eq!(
        coedges
            .iter()
            .filter(|id| !graph.topo().generic().is_removed((**id).into()))
            .count(),
        coedges.len()
    );
    assert_eq!(views_active_refs(&graph, shells), shells.len());
}
#[test]
fn BRepGraph_ViewsTest__RefsView_FaceRefIdsOf_LocalFilteringHandlesRemoved() {
    let mut graph = views_box();
    let ids = graph.refs().faces().ids_of(ShellId::new(0));
    assert!(!ids.is_empty());
    let before = ids.len();
    let first = ids[0];
    graph.editor().generic().remove_ref(first.into()).unwrap();
    assert_eq!(
        views_active_refs(&graph, graph.refs().faces().ids_of(ShellId::new(0))),
        before - 1
    );
}
#[test]
fn BRepGraph_ViewsTest__RefsView_VertexRefIdsOfEdge_ContainsBoundaryVertices() {
    let graph = views_box();
    let mut count = 0;
    for id in RefsVertexOfEdge::new(&graph, EdgeId::start()) {
        let entry = graph.refs().vertices().entry(id);
        assert!(!id.is_removed(&graph));
        assert!(
            entry
                .child_vertex_id
                .is_valid_for(graph.topo().vertices().nb())
        );
        count += 1;
    }
    assert!(count >= 2);
}
fn views_typed_ref_queries(graph: &BRepGraph, id: FaceRefId) {
    let entry = graph.refs().faces().entry(id);
    let generic = graph.refs().generic();
    assert_eq!(
        generic.child_node(id.into()),
        NodeId::from(entry.child_face_id)
    );
    assert!(generic.local_location(id.into()).is_identity());
    assert_eq!(
        generic.orientation(id.into()),
        entry.orientation.orientation()
    );
    assert_eq!(generic.nb(RefKind::FACE), graph.refs().faces().nb());
    assert!(generic.is_valid(id.into()));
    assert!(generic.is_active(id.into()));
    assert!(!generic.is_removed(id.into()));
}
#[test]
fn BRepGraph_ViewsTest__RefsView_GenericRefHelpers_RoundTripForTypedRef() {
    let mut graph = views_box();
    let id = views_face_ref(&graph);
    let mut guard = graph.editor().faces().mutate_ref(id).unwrap();
    editor::FaceOps::set_ref_orientation_guarded(&mut guard, ParityOrientation::REVERSED);
    drop(guard);
    views_typed_ref_queries(&graph, id);
    let coedge = graph.topo().wires().relations(WireId::start()).co_edge_ids[0];
    let entry = graph.topo().co_edges().definition(coedge);
    assert!(entry.child_edge_id.is_valid());
    assert!(entry.face_id.is_valid());
    assert_ne!(entry.orientation.orientation(), Orientation::Internal);
    assert_ne!(entry.orientation.orientation(), Orientation::External);
}
#[test]
fn BRepGraph_ViewsTest__RefsView_RefAtStep_RoundTrip() {
    let graph = views_box();
    let solid = SolidId::new(0);
    let reference = graph.topo().solids().relations(solid).shell_ref_ids[0];
    let generic = graph.refs().generic();
    assert_eq!(generic.ref_at_step(solid.into(), 0), RefId::from(reference));
    assert!(!generic.ref_at_step(WireId::new(0).into(), 0).is_valid());
    assert!(!generic.ref_at_step(WireId::new(0).into(), 100).is_valid());
}
fn views_product(graph: &mut BRepGraph, root: NodeId) -> ProductId {
    let id = graph.editor().products().add(root, &Location::default());
    graph.editor().products().append_document_root(id).unwrap();
    id
}
fn views_empty_product(graph: &mut BRepGraph) -> ProductId {
    let id = graph.editor().products().add_empty();
    graph.editor().products().append_document_root(id).unwrap();
    id
}
#[test]
fn BRepGraph_ViewsTest__RefsView_GenericRefHelpers_OccurrenceLocalLocation() {
    let mut graph = views_box();
    let part = views_product(&mut graph, SolidId::start().into());
    let assembly = views_empty_product(&mut graph);
    assert!(part.is_valid());
    assert!(assembly.is_valid());
    let trsf = Transform3::translation_vector(Vector3::new(4.0, 5.0, 6.0));
    let placement = Location::new(trsf);
    assert!(
        graph
            .editor()
            .products()
            .append(assembly, part, &placement, OccurrenceId::default())
            .unwrap()
            .0
            .is_valid()
    );
    let ids = graph.refs().occurrences().ids_of(assembly);
    assert_eq!(ids.len(), 1);
    views_occurrence_queries(&graph, assembly, ids[0]);
}
fn views_occurrence_queries(graph: &BRepGraph, assembly: ProductId, id: OccurrenceRefId) {
    let entry = graph.refs().occurrences().entry(id);
    let generic = graph.refs().generic();
    assert_eq!(generic.ref_at_step(assembly.into(), 0), RefId::from(id));
    assert_eq!(
        generic.child_node(id.into()),
        NodeId::from(entry.child_occurrence_id)
    );
    assert!(
        generic
            .local_location(id.into())
            .is_equal(&entry.local_location)
    );
    assert_eq!(generic.orientation(id.into()), Orientation::Forward);
    assert!(!generic.is_removed(id.into()));
}
fn views_invalid_ref_queries(graph: &BRepGraph, id: RefId) {
    let generic = graph.refs().generic();
    assert!(!generic.child_node(id).is_valid());
    assert!(generic.local_location(id).is_equal(&Location::default()));
    assert_eq!(generic.orientation(id), Orientation::Forward);
    assert!(!generic.is_valid(id));
    assert!(!generic.is_active(id));
    assert!(generic.is_removed(id));
}
#[test]
fn BRepGraph_ViewsTest__RefsView_GenericRefHelpers_InvalidAndRemoved() {
    let mut graph = views_box();
    views_invalid_ref_queries(&graph, RefId::default());
    let id = views_face_ref(&graph).into();
    graph.editor().generic().remove_ref(id).unwrap();
    assert!(graph.refs().generic().is_valid(id));
    assert!(!graph.refs().generic().is_active(id));
    assert!(graph.refs().generic().is_removed(id));
    views_invalid_ref_queries(&graph, FaceRefId::new(graph.refs().faces().nb()).into());
}
fn views_grouped_assembly_queries(
    graph: &BRepGraph,
    part: ProductId,
    sub: ProductId,
    root: ProductId,
    occurrence: OccurrenceId,
) {
    assert_eq!(NodeId::from(part), NodeId::from(part));
    assert_eq!(
        graph.topo().products().shape_root(part),
        NodeId::from(SolidId::start())
    );
    assert!(!graph.topo().products().shape_root(root).is_valid());
    assert_eq!(graph.refs().occurrences().ids_of(root).len(), 1);
    assert_eq!(graph.refs().occurrences().ids_of(sub).len(), 1);
    assert_eq!(NodeId::from(occurrence), NodeId::from(occurrence));
    assert_eq!(graph.topo().occurrences().product(occurrence), part);
    assert_eq!(graph.topo().occurrences().parent_product(occurrence), sub);
}
#[test]
fn BRepGraph_ViewsTest__TopoView_GroupedProductAndOccurrenceOps_Parity() {
    let mut graph = views_box();
    let part = views_product(&mut graph, SolidId::start().into());
    let sub = views_empty_product(&mut graph);
    let root = views_empty_product(&mut graph);
    assert!(part.is_valid());
    assert!(sub.is_valid());
    assert!(root.is_valid());
    let sub_occ = views_append_occurrence(&mut graph, root, sub, OccurrenceId::default());
    let part_occ = views_append_occurrence(&mut graph, sub, part, sub_occ);
    assert!(sub_occ.is_valid());
    assert!(part_occ.is_valid());
    views_grouped_assembly_queries(&graph, part, sub, root, part_occ);
    let ids = graph.refs().occurrences().ids_of(sub);
    assert_eq!(ids.len(), 1);
    let id = ids[0];
    graph.editor().generic().remove_ref(id.into()).unwrap();
    assert_eq!(graph.topo().products().nb_components(sub), 0);
}

fn views_append_occurrence(
    graph: &mut BRepGraph,
    parent: ProductId,
    child: ProductId,
    ancestor: OccurrenceId,
) -> OccurrenceId {
    graph
        .editor()
        .products()
        .append(parent, child, &Location::default(), ancestor)
        .unwrap()
        .0
}
