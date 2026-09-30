// Rust translation of isRefOwnedByAnyParent, detachRefRelations and
// detachOrderedParentRef of ModelingData/TKBRep/BRepGraph/BRepGraph_EditorView.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reference detachment shared by `RemoveRef` and the per-kind `Remove*` members.
use super::edge_vertices::is_vertex_ref_owned_by_active_edge;
use super::helpers::has_any_active_usage;
use crate::brep_graph::*;
use crate::brep_graph_inc::Storage;
use rustycad_tkernel::standard::Failure;

/// `isRefOwnedByAnyParent(theStorage, theRefId)`: the active reference is still
/// listed by its active parent (a vertex reference: used by an active edge).
pub(crate) fn is_ref_owned_by_any_parent(storage: &Storage, reference: RefId) -> bool {
    match reference.kind {
        RefKind::VERTEX => {
            is_vertex_ref_owned_by_active_edge(storage, VertexRefId::from_ref_id(reference))
        }
        RefKind::WIRE => {
            let id = WireRefId::from_ref_id(reference);
            owned(storage, id, |s| {
                let parent = s.wire_ref(id).parent_face_id;
                active(s, parent) && s.face_relations(parent).wire_ref_ids.contains(&id)
            })
        }
        RefKind::FACE => {
            let id = FaceRefId::from_ref_id(reference);
            owned(storage, id, |s| {
                let parent = s.face_ref(id).parent_shell_id;
                active(s, parent) && s.shell_relations(parent).face_ref_ids.contains(&id)
            })
        }
        RefKind::SHELL => {
            let id = ShellRefId::from_ref_id(reference);
            owned(storage, id, |s| {
                let parent = s.shell_ref(id).parent_solid_id;
                active(s, parent) && s.solid_relations(parent).shell_ref_ids.contains(&id)
            })
        }
        _ => is_container_ref_owned(storage, reference),
    }
}

// The solid, child and occurrence cases of `isRefOwnedByAnyParent`.
fn is_container_ref_owned(storage: &Storage, reference: RefId) -> bool {
    match reference.kind {
        RefKind::SOLID => {
            let id = SolidRefId::from_ref_id(reference);
            owned(storage, id, |s| {
                let parent = s.solid_ref(id).parent_comp_solid_id;
                active(s, parent) && s.comp_solid_relations(parent).solid_ref_ids.contains(&id)
            })
        }
        RefKind::CHILD => {
            let id = ChildRefId::from_ref_id(reference);
            owned(storage, id, |s| {
                let parent = s.child_ref(id).parent_compound_id;
                active(s, parent) && s.compound_relations(parent).child_ref_ids.contains(&id)
            })
        }
        RefKind::OCCURRENCE => {
            let id = OccurrenceRefId::from_ref_id(reference);
            owned(storage, id, |s| {
                let parent = s.occurrence_ref(id).parent_product_id;
                active(s, parent) && s.product_relations(parent).occurrence_ref_ids.contains(&id)
            })
        }
        _ => false,
    }
}

// An active reference in range whose parent test holds.
fn owned<T: crate::brep_graph_inc::RefTag>(
    storage: &Storage,
    id: TypedId<T>,
    parent_lists: impl FnOnce(&Storage) -> bool,
) -> bool {
    id.is_valid_for(storage.nb::<T>()) && !storage.is_removed(id) && parent_lists(storage)
}

// An active definition in range.
fn active<T: crate::brep_graph_inc::NodeTag>(storage: &Storage, id: TypedId<T>) -> bool {
    id.is_valid_for(storage.nb::<T>()) && !storage.is_removed(id)
}

/// `detachRefRelations(theStorage, theRef)`: detaches the reference from its parent's
/// ordered list and the relation tables (a vertex reference has none: true).
pub(crate) fn detach_ref_relations(storage: &mut Storage, reference: RefId) -> bool {
    match reference.kind {
        RefKind::WIRE => {
            let id = WireRefId::from_ref_id(reference);
            let parent = storage.wire_ref(id).parent_face_id;
            storage.detach_wire_from_face(parent, id)
        }
        RefKind::FACE => {
            let id = FaceRefId::from_ref_id(reference);
            let parent = storage.face_ref(id).parent_shell_id;
            storage.detach_face_from_shell(parent, id)
        }
        RefKind::SHELL => {
            let id = ShellRefId::from_ref_id(reference);
            let parent = storage.shell_ref(id).parent_solid_id;
            storage.detach_shell_from_solid(parent, id)
        }
        _ => detach_container_ref_relations(storage, reference),
    }
}

fn detach_container_ref_relations(storage: &mut Storage, reference: RefId) -> bool {
    match reference.kind {
        RefKind::SOLID => {
            let id = SolidRefId::from_ref_id(reference);
            let parent = storage.solid_ref(id).parent_comp_solid_id;
            storage.detach_solid_from_comp_solid(parent, id)
        }
        RefKind::CHILD => {
            let id = ChildRefId::from_ref_id(reference);
            let parent = storage.child_ref(id).parent_compound_id;
            storage.detach_child_from_compound(parent, id)
        }
        RefKind::OCCURRENCE => {
            let id = OccurrenceRefId::from_ref_id(reference);
            let parent = storage.occurrence_ref(id).parent_product_id;
            storage.detach_occurrence_from_product(parent, id)
        }
        _ => true,
    }
}

/// `findOrderedRef(<parent relations>.<RefIds>, theRef).IsValid()`: the parent's
/// ordered list holds the reference (the parent kind matches the reference kind: the
/// caller checked it; a vertex reference has no ordered list).
pub(crate) fn is_listed_by_parent(storage: &Storage, parent: NodeId, reference: RefId) -> bool {
    match reference.kind {
        RefKind::SHELL => storage
            .solid_relations(SolidId::from_node_id(parent))
            .shell_ref_ids
            .contains(&ShellRefId::from_ref_id(reference)),
        RefKind::FACE => storage
            .shell_relations(ShellId::from_node_id(parent))
            .face_ref_ids
            .contains(&FaceRefId::from_ref_id(reference)),
        RefKind::WIRE => storage
            .face_relations(FaceId::from_node_id(parent))
            .wire_ref_ids
            .contains(&WireRefId::from_ref_id(reference)),
        RefKind::SOLID => storage
            .comp_solid_relations(CompSolidId::from_node_id(parent))
            .solid_ref_ids
            .contains(&SolidRefId::from_ref_id(reference)),
        RefKind::CHILD => storage
            .compound_relations(CompoundId::from_node_id(parent))
            .child_ref_ids
            .contains(&ChildRefId::from_ref_id(reference)),
        RefKind::OCCURRENCE => storage
            .product_relations(ProductId::from_node_id(parent))
            .occurrence_ref_ids
            .contains(&OccurrenceRefId::from_ref_id(reference)),
        _ => false,
    }
}

/// `detachOrderedParentRef(theGraph, theStorage, theRefId, theParentRefIds,
/// theChildNode, theToPruneOrphanedChild)`: when the parent lists the reference,
/// detaches it, removes it unless another parent still owns it, and removes the
/// child subgraph when pruning is asked and no active use remains. `listed` is the
/// native `findOrderedRef` test, made by the caller on the parent's ordered list.
pub(crate) fn detach_ordered_parent_ref(
    graph: &mut BRepGraph,
    reference: RefId,
    listed: bool,
    child: NodeId,
    prune_orphaned_child: bool,
) -> Result<bool, Failure> {
    if !listed {
        return Ok(false);
    }
    detach_ref_relations(graph.inc_storage_mut(), reference);
    if !is_ref_owned_by_any_parent(graph.inc_storage(), reference)
        && !graph.editor().generic().remove_ref(reference)?
    {
        return Ok(false);
    }
    if prune_orphaned_child && child.is_valid() && !has_any_active_usage(graph, child)? {
        graph.editor().generic().remove_subgraph(child)?;
    }
    Ok(true)
}
