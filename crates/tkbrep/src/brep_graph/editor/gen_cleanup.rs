// Rust translation of EditorView::GenOps::CleanupRemovedReferences of
// ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GenOps::CleanupRemovedReferences`: the forward references to removed nodes are
//! retired, in the native phase order.
use super::gen_ops::GenOps;
use super::gen_remove::mark_owned_ref_removed;
use super::helpers::clear_co_edge_face_scoped_representations;
use crate::brep_graph::*;
use crate::brep_graph_inc::{NodeTag, RefTag, Storage};
use rustycad_tkernel::ncollection::FlatMap;

/// The removed nodes (`NCollection_FlatMap<BRepGraph_NodeId>`: its iteration order is
/// the order of the reference removals of the first phase).
type RemovedSet = FlatMap<NodeId>;

const REMOVED_SCAN_KINDS: [NodeKind; 11] = [
    NodeKind::VERTEX,
    NodeKind::EDGE,
    NodeKind::CO_EDGE,
    NodeKind::WIRE,
    NodeKind::FACE,
    NodeKind::SHELL,
    NodeKind::SOLID,
    NodeKind::COMPOUND,
    NodeKind::COMP_SOLID,
    NodeKind::PRODUCT,
    NodeKind::OCCURRENCE,
];

impl GenOps<'_> {
    /// `CleanupRemovedReferences()`: after `RemoveNode` calls, the references owned by
    /// removed nodes are removed (in removed-set order), coedges, edges, wires, faces,
    /// shells, solids, compounds, compsolids, occurrences (retired when their child is
    /// removed) and products drop what points at removed nodes, and the active counts
    /// are recounted.
    ///
    /// ```
    /// use rustycad_tkbrep::brep_graph::BRepGraph;
    /// let mut graph = BRepGraph::new();
    /// graph.editor().generic().cleanup_removed_references();
    /// assert!(graph.validate_relations());
    /// ```
    pub fn cleanup_removed_references(&mut self) {
        let mut removed = collect_removed(self.graph.inc_storage());
        let owners: Vec<NodeId> = removed.iter().copied().collect();
        for node in owners {
            for reference in owned_refs(self.graph.inc_storage(), node) {
                mark_owned_ref_removed(self.graph, reference);
            }
        }
        clear_co_edge_links(self.graph.inc_storage_mut(), &removed);
        clear_edge_vertex_refs(self.graph, &removed);
        retire_orphan_co_edges(self.graph.inc_storage_mut(), &removed);
        prune_wire_co_edges(self.graph.inc_storage_mut(), &removed);
        prune_container_refs(self.graph.inc_storage_mut(), &removed);
        retire_orphan_occurrences(self.graph.inc_storage_mut(), &mut removed);
        prune_product_refs(self.graph.inc_storage_mut(), &removed);
        self.graph.inc_storage_mut().recount_active_counts();
    }
}

// Phase 1: every removed node of every kind, in kind then index order.
fn collect_removed(storage: &Storage) -> RemovedSet {
    struct Scan<'s, 'm>(&'s Storage, &'m mut RemovedSet);
    impl NodeVisitor for Scan<'_, '_> {
        type Output = ();
        fn visit<T: NodeTag>(self, _id: TypedId<T>) {
            for index in 0..self.0.nb::<T>() {
                let id = TypedId::<T>::new(index);
                if self.0.is_removed(id) {
                    self.1.add(id.into());
                }
            }
        }
    }
    let mut removed = RemovedSet::default();
    for kind in REMOVED_SCAN_KINDS {
        NodeId::start(kind).visit(Scan(storage, &mut removed));
    }
    removed
}

// The references a removed node owns: an edge's boundary references, a face's wire
// references, a container's child references.
fn owned_refs(storage: &Storage, node: NodeId) -> Vec<RefId> {
    fn refs<I: Copy + Into<RefId>>(ids: &[I]) -> Vec<RefId> {
        ids.iter().map(|id| (*id).into()).collect()
    }
    match node.kind {
        NodeKind::EDGE => {
            let def = storage.edge(EdgeId::from_node_id(node));
            vec![def.start_vertex_ref_id.into(), def.end_vertex_ref_id.into()]
        }
        NodeKind::FACE => refs(
            &storage
                .face_relations(FaceId::from_node_id(node))
                .wire_ref_ids,
        ),
        NodeKind::SHELL => refs(
            &storage
                .shell_relations(ShellId::from_node_id(node))
                .face_ref_ids,
        ),
        NodeKind::SOLID => refs(
            &storage
                .solid_relations(SolidId::from_node_id(node))
                .shell_ref_ids,
        ),
        NodeKind::COMP_SOLID => refs(
            &storage
                .comp_solid_relations(CompSolidId::from_node_id(node))
                .solid_ref_ids,
        ),
        NodeKind::COMPOUND => refs(
            &storage
                .compound_relations(CompoundId::from_node_id(node))
                .child_ref_ids,
        ),
        NodeKind::PRODUCT => refs(
            &storage
                .product_relations(ProductId::from_node_id(node))
                .occurrence_ref_ids,
        ),
        _ => Vec::new(),
    }
}

// The active ids of one kind, the removed state read when each is visited (the
// native `BRepGraph_Iterator` skips removed slots as it advances).
fn for_each_active<T: NodeTag>(
    storage: &mut Storage,
    mut visit: impl FnMut(&mut Storage, TypedId<T>),
) {
    for index in 0..storage.nb::<T>() {
        let id = TypedId::<T>::new(index);
        if !storage.is_removed(id) {
            visit(storage, id);
        }
    }
}

fn contains<T: NodeTag>(removed: &RemovedSet, storage: &Storage, id: TypedId<T>) -> bool {
    id.is_valid_for(storage.nb::<T>()) && removed.contains(&id.into())
}

// Phase 2: active coedges lose a removed edge (relation rebound) or a removed face
// (face-scoped records dropped).
fn clear_co_edge_links(storage: &mut Storage, removed: &RemovedSet) {
    for_each_active::<CoEdgeTag>(storage, |storage, co_edge| {
        let edge = storage.co_edge(co_edge).child_edge_id;
        if contains(removed, storage, edge) {
            storage.change_co_edge(co_edge).child_edge_id = EdgeId::default();
            storage.rebind_co_edge_edge(co_edge, edge, EdgeId::default());
        }
        let face = storage.co_edge(co_edge).face_id;
        if contains(removed, storage, face) {
            storage.change_co_edge(co_edge).face_id = FaceId::default();
            clear_co_edge_face_scoped_representations(storage, co_edge);
        }
    });
}

// Phase 3: active edges drop a removed boundary reference or one to a removed vertex.
fn clear_edge_vertex_refs(graph: &mut BRepGraph, removed: &RemovedSet) {
    for index in 0..graph.inc_storage().nb_edges() {
        let edge = EdgeId::new(index);
        if graph.inc_storage().is_removed(edge) {
            continue;
        }
        clear_edge_slot(graph, removed, edge, true);
        clear_edge_slot(graph, removed, edge, false);
    }
}

fn clear_edge_slot(graph: &mut BRepGraph, removed: &RemovedSet, edge: EdgeId, start: bool) {
    let storage = graph.inc_storage();
    let def = storage.edge(edge);
    let reference = if start {
        def.start_vertex_ref_id
    } else {
        def.end_vertex_ref_id
    };
    if !reference.is_valid_for(storage.nb_vertex_refs()) {
        return;
    }
    let vertex = storage.vertex_ref(reference).child_vertex_id;
    if !storage.is_removed(reference) && !contains(removed, storage, vertex) {
        return;
    }
    mark_owned_ref_removed(graph, reference.into());
    let storage = graph.inc_storage_mut();
    let def = storage.change_edge(edge);
    if start {
        def.start_vertex_ref_id = VertexRefId::default();
    } else {
        def.end_vertex_ref_id = VertexRefId::default();
    }
    storage.rebind_vertex_edge(vertex, VertexId::default(), edge, VertexRefId::default());
}

// Phase 4a: an active coedge of a removed wire leaves it and is removed; one without
// an edge is removed.
fn retire_orphan_co_edges(storage: &mut Storage, removed: &RemovedSet) {
    for_each_active::<CoEdgeTag>(storage, |storage, co_edge| {
        let def = storage.co_edge(co_edge);
        let (wire, edge) = (def.parent_wire_id, def.child_edge_id);
        if contains(removed, storage, wire) {
            storage.detach_co_edge_use(wire, co_edge);
            storage.mark_removed(co_edge.into());
        } else if !edge.is_valid() {
            storage.mark_removed(co_edge.into());
        }
    });
}

// Phase 4b: active wires detach removed coedges and coedges without a live edge.
fn prune_wire_co_edges(storage: &mut Storage, removed: &RemovedSet) {
    for_each_active::<WireTag>(storage, |storage, wire| {
        let mut index = 0;
        while let Some(co_edge) = storage.wire_relations(wire).co_edge_ids.get(index).copied() {
            if !co_edge.is_valid_for(storage.nb_co_edges()) {
                index += 1;
                continue;
            }
            let edge = storage.co_edge(co_edge).child_edge_id;
            if storage.is_removed(co_edge) || !edge.is_valid() || contains(removed, storage, edge) {
                storage.detach_co_edge_use(wire, co_edge);
                continue;
            }
            index += 1;
        }
    });
}

// Phases 5 to 9: the active faces, shells, solids, compounds and compsolids drop the
// active references to removed children.
fn prune_container_refs(storage: &mut Storage, removed: &RemovedSet) {
    for_each_active::<FaceTag>(storage, |storage, face| {
        prune_list(
            storage,
            |s| &s.face_relations(face).wire_ref_ids,
            |s, r| contains(removed, s, s.wire_ref(r).child_wire_id),
            |s, r| s.detach_wire_from_face(face, r),
        );
    });
    for_each_active::<ShellTag>(storage, |storage, shell| {
        prune_list(
            storage,
            |s| &s.shell_relations(shell).face_ref_ids,
            |s, r| contains(removed, s, s.face_ref(r).child_face_id),
            |s, r| s.detach_face_from_shell(shell, r),
        );
    });
    for_each_active::<SolidTag>(storage, |storage, solid| {
        prune_list(
            storage,
            |s| &s.solid_relations(solid).shell_ref_ids,
            |s, r| contains(removed, s, s.shell_ref(r).child_shell_id),
            |s, r| s.detach_shell_from_solid(solid, r),
        );
    });
    prune_compound_and_comp_solid_refs(storage, removed);
}

fn prune_compound_and_comp_solid_refs(storage: &mut Storage, removed: &RemovedSet) {
    for_each_active::<CompoundTag>(storage, |storage, compound| {
        prune_list(
            storage,
            |s| &s.compound_relations(compound).child_ref_ids,
            |s, r| {
                let child = s.child_ref(r).child_node_id;
                child.is_valid() && removed.contains(&child)
            },
            |s, r| s.detach_child_from_compound(compound, r),
        );
    });
    for_each_active::<CompSolidTag>(storage, |storage, comp_solid| {
        prune_list(
            storage,
            |s| &s.comp_solid_relations(comp_solid).solid_ref_ids,
            |s, r| contains(removed, s, s.solid_ref(r).child_solid_id),
            |s, r| s.detach_solid_from_comp_solid(comp_solid, r),
        );
    });
}

// The shared loop of phases 5 to 9 and 11: walks the parent's live list; an active
// reference whose child is removed is marked removed and detached (the list shrinks,
// so the index stays), any other entry is skipped.
fn prune_list<T: RefTag>(
    storage: &mut Storage,
    list: impl Fn(&Storage) -> &[TypedId<T>],
    child_removed: impl Fn(&Storage, TypedId<T>) -> bool,
    detach: impl Fn(&mut Storage, TypedId<T>) -> bool,
) where
    TypedId<T>: Into<RefId>,
{
    let mut index = 0;
    while let Some(reference) = list(storage).get(index).copied() {
        if storage.is_removed(reference) || !child_removed(storage, reference) {
            index += 1;
            continue;
        }
        storage.mark_removed_ref(reference.into());
        detach(storage, reference);
    }
}

// Phase 10: an active occurrence of a removed child loses it and is retired (it joins
// the removed set for phase 11).
fn retire_orphan_occurrences(storage: &mut Storage, removed: &mut RemovedSet) {
    for index in 0..storage.nb_occurrences() {
        let occurrence = OccurrenceId::new(index);
        if storage.is_removed(occurrence) {
            continue;
        }
        let child = storage.occurrence(occurrence).child_node_id;
        if child.is_valid() && removed.contains(&child) {
            storage.change_occurrence(occurrence).child_node_id = NodeId::default();
            storage.mark_removed(occurrence.into());
            removed.add(occurrence.into());
        }
    }
}

// Phase 11: active products drop the active references to removed occurrences.
fn prune_product_refs(storage: &mut Storage, removed: &RemovedSet) {
    for_each_active::<ProductTag>(storage, |storage, product| {
        prune_list(
            storage,
            |s| &s.product_relations(product).occurrence_ref_ids,
            |s, r| contains(removed, s, s.occurrence_ref(r).child_occurrence_id),
            |s, r| s.detach_occurrence_from_product(product, r),
        );
    });
}
