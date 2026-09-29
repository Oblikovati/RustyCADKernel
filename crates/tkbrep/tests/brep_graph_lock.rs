//! Behaviour of BRepGraph_LayerLock and of the layer copies on a box graph: root
//! owners and their propagated bits, overlaps, absorption and conflicts, removal and
//! rebuild, owner scopes, registry dispatch, and CopyLayersTo into a second graph or in
//! place.
mod support;
use rustycad_tkbrep::brep_graph::child_explorer::ChildExplorer;
use rustycad_tkbrep::brep_graph::copy_remap::{ItemMap, MappingKind, Mode};
use rustycad_tkbrep::brep_graph::layer::BRepGraphLayer;
use rustycad_tkbrep::brep_graph::layer_deferred::{LayerDeferred, RepresentationKind};
use rustycad_tkbrep::brep_graph::layer_history::{HistoryKind, LayerHistory};
use rustycad_tkbrep::brep_graph::layer_lock::{LayerLock, ScopedOwnerEdit};
use rustycad_tkbrep::brep_graph::layer_registry::LayerRegistry;
use rustycad_tkbrep::brep_graph::layer_topo_supplement::{AttachmentKind, LayerTopoSupplement};
use rustycad_tkbrep::brep_graph::{
    BRepGraph, EdgeId, FaceId, FaceRefId, ItemId, NodeId, NodeKind, RefId, ShellId, SolidId,
};
use rustycad_tkbrep::brep_graph_inc::populate::{self, Options};
use rustycad_tkernel::standard::{Guid, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use support::{make_box, make_vertex};

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    populate::perform(&mut graph, &make_box(10.0, 20.0, 30.0), false, &Options).unwrap();
    graph
}

// A box added through the shapes view, which gives every node a UID (the history
// copies replay item UIDs).
fn added_box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph
        .shapes_mut()
        .add(&make_box(10.0, 20.0, 30.0).into())
        .unwrap();
    graph
}

fn owner(index: u8) -> Guid {
    Guid::from_components(1, 2, 3, 4, 5, 6, 7, 8, 9, index)
}

fn lock_of(graph: &BRepGraph) -> Handle<LayerLock> {
    graph.layer_registry().ensure::<LayerLock>().unwrap()
}

fn node(id: impl Into<NodeId>) -> ItemId {
    ItemId::from(id.into())
}

fn reference(id: impl Into<RefId>) -> ItemId {
    ItemId::from(id.into())
}

#[test]
fn a_shell_root_owns_its_subtree() {
    let graph = box_graph();
    let lock = lock_of(&graph);
    let shell = node(ShellId::new(0));
    assert!(
        lock.set_owner_with_revision(&graph, shell, &owner(1), true)
            .unwrap()
    );
    assert!(lock.has_owner(&graph, node(FaceId::new(3))));
    assert!(lock.has_owner(&graph, reference(FaceRefId::new(2))));
    assert!(!lock.has_owner(&graph, node(SolidId::new(0))));
    let resolved = lock.find_owner_id(&graph, node(EdgeId::new(5))).unwrap();
    assert_eq!(resolved, Some(owner(1)));
    let face_ref = lock
        .find_owner_id(&graph, reference(FaceRefId::new(1)))
        .unwrap();
    assert_eq!(face_ref, Some(owner(1)));
    assert_eq!(lock.revision(), 1);
}

#[test]
fn overlapping_roots_are_rejected_or_reaffirmed() {
    let graph = box_graph();
    let lock = lock_of(&graph);
    let (shell, face) = (node(ShellId::new(0)), node(FaceId::new(0)));
    lock.set_owner(&graph, shell, &owner(1)).unwrap();
    assert!(
        !lock
            .set_owner_with_revision(&graph, face, &owner(2), true)
            .unwrap()
    );
    let revision = lock.revision();
    assert!(
        !lock
            .set_owner_with_revision(&graph, face, &owner(1), true)
            .unwrap()
    );
    assert_eq!(lock.revision(), revision + 1);
    let error = lock.set_owner(&graph, face, &owner(2)).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert!(
        !lock
            .set_owner_with_revision(&graph, ItemId::default(), &owner(1), true)
            .unwrap()
    );
}

#[test]
fn an_ancestor_root_absorbs_same_owner_descendants_only() {
    let graph = box_graph();
    let lock = lock_of(&graph);
    lock.set_owner(&graph, node(FaceId::new(0)), &owner(1))
        .unwrap();
    lock.set_owner(&graph, node(FaceId::new(1)), &owner(2))
        .unwrap();
    let solid = node(SolidId::new(0));
    assert!(
        !lock
            .set_owner_with_revision(&graph, solid, &owner(1), true)
            .unwrap()
    );
    lock.unset_owner(&graph, node(FaceId::new(1))).unwrap();
    assert!(
        lock.set_owner_with_revision(&graph, solid, &owner(1), true)
            .unwrap()
    );
    let face = lock.find_owner_id(&graph, node(FaceId::new(0))).unwrap();
    assert_eq!(face, Some(owner(1)));
    lock.unset_owner(&graph, node(FaceId::new(0))).unwrap();
    assert!(lock.has_owner(&graph, node(FaceId::new(0))));
}

#[test]
fn unsetting_a_root_rebuilds_the_other_roots() {
    let graph = box_graph();
    let lock = lock_of(&graph);
    lock.set_owner(&graph, node(FaceId::new(1)), &owner(3))
        .unwrap();
    lock.set_owner(&graph, node(FaceId::new(0)), &owner(1))
        .unwrap();
    lock.unset_owner_matching(&graph, node(FaceId::new(0)), &owner(2))
        .unwrap();
    assert!(lock.has_owner(&graph, node(FaceId::new(0))));
    lock.unset_owner_matching(&graph, node(FaceId::new(0)), &owner(1))
        .unwrap();
    assert!(!lock.has_owner(&graph, node(FaceId::new(0))));
    assert!(lock.has_owner(&graph, node(FaceId::new(1))));
    // The edges face 0 shares with face 1 lost their bit and got it back.
    let edges = ChildExplorer::with_target(&graph, FaceId::new(1), NodeKind::EDGE).unwrap();
    for edge in edges {
        assert!(lock.has_owner(&graph, ItemId::from(edge.unwrap().def_id)));
    }
    lock.clear(Some(&graph));
    assert!(!lock.has_owners() && !lock.has_owner(&graph, node(FaceId::new(1))));
}

#[test]
fn an_owner_scope_lifts_one_bit() {
    let graph = box_graph();
    let lock = lock_of(&graph);
    let face = node(FaceId::new(2));
    lock.set_owner(&graph, node(ShellId::new(0)), &owner(1))
        .unwrap();
    {
        let _scope = ScopedOwnerEdit::new(&lock, &graph, face, &owner(1)).unwrap();
        assert!(!lock.has_owner(&graph, face));
    }
    assert!(lock.has_owner(&graph, face));
    let mismatch = ScopedOwnerEdit::new(&lock, &graph, face, &owner(2));
    assert_eq!(
        mismatch.err().unwrap().exception_type(),
        "Standard_ProgramError"
    );
    let invalid = ScopedOwnerEdit::new(&lock, &graph, ItemId::default(), &owner(1));
    assert!(invalid.is_err());
}

#[test]
fn registry_events_drop_and_move_roots() {
    let graph = box_graph();
    let lock = lock_of(&graph);
    lock.set_owner(&graph, node(FaceId::new(0)), &owner(1))
        .unwrap();
    lock.set_owner(&graph, reference(FaceRefId::new(3)), &owner(2))
        .unwrap();
    let registry = graph.layer_registry();
    registry.dispatch_on_node_replaced(Some(&graph), FaceId::new(0).into(), FaceId::new(4).into());
    assert!(lock.has_owner(&graph, node(FaceId::new(4))));
    assert!(!lock.has_owner(&graph, node(FaceId::new(0))));
    registry.dispatch_on_ref_removed(Some(&graph), FaceRefId::new(3).into());
    assert!(!lock.has_owner(&graph, reference(FaceRefId::new(3))));
    registry.dispatch_on_node_removed(Some(&graph), FaceId::new(4).into());
    assert!(!lock.has_owners());
}

#[test]
fn a_detached_lock_keeps_entries_without_bits() {
    let graph = box_graph();
    let lock = LayerLock::new();
    let face = node(FaceId::new(0));
    assert!(
        lock.set_owner_with_revision(&graph, face, &owner(1), true)
            .unwrap()
    );
    assert!(!lock.has_owner(&graph, face));
    assert_eq!(lock.find_owner_id(&graph, face).unwrap(), Some(owner(1)));
    lock.reserve_owners(8);
    lock.touch_owners();
    assert_eq!(lock.revision(), 2);
}

#[test]
fn layers_copy_into_an_identical_graph() {
    let source = added_box_graph();
    lock_of(&source)
        .set_owner(&source, node(ShellId::new(0)), &owner(1))
        .unwrap();
    let history = source.layer_registry().ensure::<LayerHistory>().unwrap();
    let label = AsciiString::new("Op");
    history.record(
        &source,
        &label,
        EdgeId::new(0).into(),
        &[EdgeId::new(1).into()],
        HistoryKind::Modified,
    );
    let deferred = source.layer_registry().ensure::<LayerDeferred>().unwrap();
    let text = AsciiString::new("x");
    deferred
        .register_deferred_node(
            &source,
            EdgeId::new(3).into(),
            &text,
            &text,
            RepresentationKind::Mesh,
            &text,
            1,
        )
        .unwrap();
    let supplement = source
        .layer_registry()
        .ensure::<LayerTopoSupplement>()
        .unwrap();
    let vertex = make_vertex(Point3::new(1.0, 2.0, 3.0));
    let uid = supplement.add_attachment(
        &source,
        FaceId::new(1).into(),
        AttachmentKind::FaceDirectVertex,
        &vertex.into(),
    );
    let mut target = added_box_graph();
    LayerRegistry::copy_layers_to_with_mapping(
        &source,
        &mut target,
        MappingKind::Identity,
        Mode::Copy,
    )
    .unwrap();
    let copied = target.layer_registry().find::<LayerLock>();
    assert!(copied.has_owner(&target, node(FaceId::new(5))));
    let target_history = target.layer_registry().find::<LayerHistory>();
    assert_eq!(
        target_history.find_original(EdgeId::new(1).into()),
        NodeId::from(EdgeId::new(0))
    );
    let target_deferred = target.layer_registry().find::<LayerDeferred>();
    assert!(target_deferred.has_deferred_node(EdgeId::new(3).into()));
    let target_supplement = target.layer_registry().find::<LayerTopoSupplement>();
    assert_eq!(
        target_supplement.attached_to(FaceId::new(1).into()),
        vec![uid]
    );
}

#[test]
fn an_explicit_map_drops_unmapped_items() {
    let source = box_graph();
    lock_of(&source)
        .set_owner(&source, node(FaceId::new(0)), &owner(1))
        .unwrap();
    lock_of(&source)
        .set_owner(&source, node(FaceId::new(1)), &owner(2))
        .unwrap();
    let mut items = ItemMap::default();
    items.bind(node(FaceId::new(0)), node(FaceId::new(3)));
    let mut target = box_graph();
    LayerRegistry::copy_layers_to(&source, &mut target, &items, Mode::Copy).unwrap();
    let copied = target.layer_registry().find::<LayerLock>();
    let face3 = copied.find_owner_id(&target, node(FaceId::new(3))).unwrap();
    assert_eq!(face3, Some(owner(1)));
    assert_eq!(
        copied.find_owner_id(&target, node(FaceId::new(1))).unwrap(),
        None
    );
}

#[test]
fn an_in_place_compaction_migrates_new_layer_instances() {
    let mut graph = box_graph();
    let history = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    let label = AsciiString::new("Op");
    history.record(
        &graph,
        &label,
        EdgeId::new(0).into(),
        &[EdgeId::new(1).into()],
        HistoryKind::Generated,
    );
    lock_of(&graph)
        .set_owner(&graph, node(FaceId::new(0)), &owner(1))
        .unwrap();
    let mut items = ItemMap::default();
    items.bind(node(EdgeId::new(0)), node(EdgeId::new(2)));
    items.bind(node(EdgeId::new(1)), node(EdgeId::new(3)));
    items.bind(node(FaceId::new(0)), node(FaceId::new(1)));
    LayerRegistry::copy_layers_in_place(&mut graph, &items, Mode::Compact).unwrap();
    let migrated = graph.layer_registry().find::<LayerHistory>();
    assert!(migrated != history);
    assert_eq!(
        migrated.find_generated(EdgeId::new(2).into()),
        Some(vec![EdgeId::new(3).into()])
    );
    let lock = graph.layer_registry().find::<LayerLock>();
    assert_eq!(
        lock.find_owner_id(&graph, node(FaceId::new(1))).unwrap(),
        Some(owner(1))
    );
    assert!(!history.layer_base().is_attached());
}
