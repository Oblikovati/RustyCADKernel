//! Behaviour of the editor's structural operations that the translated suites leave
//! out, on a box graph: container removals and reference retargeting, reference and
//! subgraph removal with cleanup, products and occurrences, wire checks and edits,
//! guard move assignment, the supplement editor and the mutation boundary. The OCCT
//! comparison of the same members is the `br_bedit` probe
//! (tools/tkbrep_brepgraph_edit_corpus.py).
mod support;
use rustycad_tkbrep::brep_graph::editor::{
    CoEdgeOrderStatus, FaceOps, OccurrenceOps, ReplaceEdgeStatus, ShellOps, SolidOps, WireOps,
};
use rustycad_tkbrep::brep_graph::layer_topo_supplement::{AttachmentKind, LayerTopoSupplement};
use rustycad_tkbrep::brep_graph::mut_guard::MutGuard;
use rustycad_tkbrep::brep_graph::supplement_editor::SupplementEditor;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use support::{make_box, make_vertex};

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&make_box(10.0, 20.0, 30.0)).unwrap();
    graph
}

fn wire_co_edges(graph: &BRepGraph, wire: WireId) -> Vec<CoEdgeId> {
    graph.topo().wires().relations(wire).co_edge_ids.clone()
}

#[test]
fn a_fresh_graph_passes_the_mutation_boundary() {
    let mut graph = box_graph();
    let mut issues = Vec::new();
    assert!(
        graph
            .editor_state()
            .validate_mutation_boundary(Some(&mut issues))
    );
    assert!(issues.is_empty());
    graph.editor().commit_mutation();
    assert!(graph.editor().validate_mutation_boundary(None));
}

#[test]
fn shell_and_solid_references_are_removed_once() {
    let mut graph = box_graph();
    let mut shells = graph.editor().shells();
    assert!(
        shells
            .remove_face(ShellId::new(0), FaceRefId::new(0))
            .unwrap()
    );
    assert!(
        !shells
            .remove_face(ShellId::new(0), FaceRefId::new(0))
            .unwrap()
    );
    let mut solids = graph.editor().solids();
    assert!(
        solids
            .remove_shell(SolidId::new(0), ShellRefId::new(0))
            .unwrap()
    );
    assert!(
        !solids
            .remove_shell(SolidId::new(0), ShellRefId::new(0))
            .unwrap()
    );
}

#[test]
fn a_compsolid_solid_reference_is_retargeted_and_removed() {
    let mut graph = box_graph();
    let comp_solid = graph.editor().comp_solids().add(&[SolidId::new(0)]);
    let spare = graph.editor().solids().add();
    let reference = SolidRefId::new(graph.refs().solids().nb() - 1);
    graph
        .editor()
        .solids()
        .set_ref_child_solid_id(reference, spare)
        .unwrap();
    assert_eq!(graph.refs().solids().entry(reference).child_solid_id, spare);
    let mut guard = graph.editor().solids().mutate_ref(reference).unwrap();
    SolidOps::set_ref_child_solid_id_guarded(&mut guard, SolidId::new(0));
    drop(guard);
    assert_eq!(
        graph.refs().solids().entry(reference).child_solid_id,
        SolidId::new(0)
    );
    let mut comp_solids = graph.editor().comp_solids();
    assert!(comp_solids.remove_solid(comp_solid, reference).unwrap());
    assert!(!comp_solids.remove_solid(comp_solid, reference).unwrap());
}

#[test]
fn a_compound_child_reference_is_removed_once() {
    let mut graph = box_graph();
    let children = [FaceId::new(0).into(), FaceId::new(1).into()];
    let compound = graph.editor().compounds().add(&children);
    let first = ChildRefId::new(graph.refs().children().nb() - 2);
    let mut compounds = graph.editor().compounds();
    assert!(compounds.remove_child(compound, first).unwrap());
    assert!(!compounds.remove_child(compound, first).unwrap());
}

#[test]
fn a_face_wire_use_is_removed_with_its_unshared_wire() {
    let mut graph = box_graph();
    let mut faces = graph.editor().faces();
    assert!(
        faces
            .remove_wire(FaceId::new(0), WireRefId::new(0))
            .unwrap()
    );
    assert!(
        !faces
            .remove_wire(FaceId::new(0), WireRefId::new(0))
            .unwrap()
    );
    assert!(WireId::new(0).is_removed(&graph));
}

#[test]
fn guarded_reference_retargets_rebind_their_children() {
    let mut graph = box_graph();
    let mut guard = graph
        .editor()
        .faces()
        .mutate_ref(FaceRefId::new(0))
        .unwrap();
    FaceOps::set_ref_face_id_guarded(&mut guard, FaceId::new(2));
    drop(guard);
    assert_eq!(
        graph.refs().faces().entry(FaceRefId::new(0)).child_face_id,
        FaceId::new(2)
    );
    let mut guard = graph
        .editor()
        .wires()
        .mutate_ref(WireRefId::new(1))
        .unwrap();
    WireOps::set_ref_child_wire_id_guarded(&mut guard, WireId::new(2));
    drop(guard);
    assert_eq!(
        graph.refs().wires().entry(WireRefId::new(1)).child_wire_id,
        WireId::new(2)
    );
    let extra = graph.editor().shells().add();
    let mut guard = graph
        .editor()
        .shells()
        .mutate_ref(ShellRefId::new(0))
        .unwrap();
    ShellOps::set_ref_child_shell_id_guarded(&mut guard, extra);
    drop(guard);
    assert_eq!(
        graph
            .refs()
            .shells()
            .entry(ShellRefId::new(0))
            .child_shell_id,
        extra
    );
}

#[test]
fn a_reference_is_removed_only_under_its_owner() {
    let mut graph = box_graph();
    let mut generic = graph.editor().generic();
    let face_ref: RefId = FaceRefId::new(1).into();
    assert!(
        !generic
            .remove_ref_of_parent(SolidId::new(0).into(), face_ref, true)
            .unwrap()
    );
    assert!(
        generic
            .remove_ref_of_parent(ShellId::new(0).into(), face_ref, true)
            .unwrap()
    );
    assert!(face_ref.is_removed(&graph));
    // The face had no other use, so the prune removed it.
    assert!(FaceId::new(1).is_removed(&graph));
}

#[test]
fn a_subgraph_is_removed_with_its_unshared_descendants() {
    let mut graph = box_graph();
    graph
        .editor()
        .generic()
        .remove_subgraph(ShellId::new(0).into())
        .unwrap();
    assert!(ShellId::new(0).is_removed(&graph));
    assert!((0..6).all(|i| FaceId::new(i).is_removed(&graph)));
    assert!(EdgeId::new(0).is_removed(&graph));
}

#[test]
fn cleanup_removes_the_references_to_a_removed_face() {
    let mut graph = box_graph();
    let face_ref: RefId = FaceRefId::new(0).into();
    graph
        .editor()
        .generic()
        .remove_node(FaceId::new(0).into())
        .unwrap();
    graph.editor().generic().cleanup_removed_references();
    assert!(face_ref.is_removed(&graph));
    assert!(graph.validate_relations());
}

#[test]
fn occurrences_are_retargeted_and_removed() {
    let mut graph = BRepGraph::new();
    let mut products = graph.editor().products();
    let (part, other, assembly) = (
        products.add_empty(),
        products.add_empty(),
        products.add_empty(),
    );
    let (occurrence, reference) = products
        .append(
            assembly,
            part,
            &Location::default(),
            OccurrenceId::default(),
        )
        .unwrap();
    let mut occurrences = graph.editor().occurrences();
    occurrences
        .set_child_node_id(occurrence, other.into())
        .unwrap();
    assert_eq!(
        graph
            .topo()
            .occurrences()
            .definition(occurrence)
            .child_node_id,
        other
    );
    let mut guard = graph.editor().occurrences().mutate(occurrence).unwrap();
    OccurrenceOps::set_child_node_id_guarded(&mut guard, part.into());
    drop(guard);
    assert_eq!(
        graph
            .topo()
            .occurrences()
            .definition(occurrence)
            .child_node_id,
        part
    );
    // The occurrence is already owned by its reference: rebinding it to itself is a no-op.
    graph
        .editor()
        .occurrences()
        .set_ref_child_occurrence_id(reference, occurrence)
        .unwrap();
    let mut guard = graph.editor().occurrences().mutate_ref(reference).unwrap();
    OccurrenceOps::set_ref_child_occurrence_id_guarded(&mut guard, occurrence);
    assert!(!guard.is_dirty() || guard.is_active());
    drop(guard);
    let mut products = graph.editor().products();
    assert!(products.remove_occurrence(assembly, reference).unwrap());
    assert!(!products.remove_occurrence(assembly, reference).unwrap());
}

#[test]
fn a_shape_root_is_removed_once() {
    let mut graph = box_graph();
    let product = graph
        .editor()
        .products()
        .add(SolidId::new(0).into(), &Location::default());
    let mut products = graph.editor().products();
    assert!(products.remove_shape_root(product).unwrap());
    assert!(!products.remove_shape_root(product).unwrap());
}

/// A free triangle wire built through the editor: three vertices, three edges without
/// curves and three FORWARD coedges chained v0 -> v1 -> v2 -> v0. (A face of the test
/// support box lists one coedge against its chain, so the box wires check as
/// disconnected.)
fn triangle(graph: &mut BRepGraph) -> (WireId, Vec<CoEdgeId>, Vec<EdgeId>) {
    let mut e = graph.editor();
    let corners = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
    let v: Vec<VertexId> = corners
        .iter()
        .map(|&(x, y)| e.reborrow().vertices().add(Point3::new(x, y, 0.0), 1.0e-7))
        .collect();
    let edges: Vec<EdgeId> = (0..3)
        .map(|i| {
            let curve = Handle::default();
            e.reborrow()
                .edges()
                .add(v[i], v[(i + 1) % 3], &curve, 0.0, 1.0, 1.0e-7)
        })
        .collect();
    let chain: Vec<CoEdgeId> = edges
        .iter()
        .map(|&edge| {
            e.reborrow()
                .co_edges()
                .add(edge, ParityOrientation::FORWARD)
        })
        .collect();
    (e.wires().add(&chain), chain, edges)
}

#[test]
fn wire_orders_are_checked_against_the_stored_chain() {
    let mut graph = BRepGraph::new();
    let (wire, chain, edges) = triangle(&mut graph);
    let wires = graph.editor().wires();
    assert_eq!(
        wires.check_wire_co_edge_order(wire, &chain),
        CoEdgeOrderStatus::AlreadyCurrent
    );
    assert_eq!(
        wires.check_append_co_edge(wire, chain[0]),
        CoEdgeOrderStatus::AlreadyContained
    );
    assert_eq!(wires.check_co_edge_order(&[]), CoEdgeOrderStatus::Empty);
    assert_eq!(
        wires.check_co_edge_order(&[chain[0], chain[0]]),
        CoEdgeOrderStatus::DuplicateCoEdge
    );
    assert_eq!(
        wires.check_replace_edge(wire, edges[0], edges[0], false),
        ReplaceEdgeStatus::AlreadyCurrent
    );
    assert_eq!(
        wires.check_replace_edge(wire, edges[0], EdgeId::new(99), false),
        ReplaceEdgeStatus::InvalidNewEdge
    );
}

#[test]
fn a_wire_is_reordered_and_loses_a_coedge() {
    let mut graph = BRepGraph::new();
    let (wire, chain, _) = triangle(&mut graph);
    let rotated = [chain[1], chain[2], chain[0]];
    assert!(graph.editor().wires().set_co_edge_order(wire, &rotated));
    assert_eq!(wire_co_edges(&graph, wire), rotated);
    let mut wires = graph.editor().wires();
    assert!(wires.remove_co_edge(wire, chain[2]).unwrap());
    assert!(!wires.remove_co_edge(wire, chain[2]).unwrap());
    assert_eq!(wire_co_edges(&graph, wire).len(), 2);
}

#[test]
fn an_edge_of_a_wire_is_replaced_by_a_parallel_edge() {
    let mut graph = BRepGraph::new();
    let (wire, chain, edges) = triangle(&mut graph);
    let start = graph
        .refs()
        .vertices()
        .entry(tool::edge::start_vertex_id(&graph, edges[0]))
        .child_vertex_id;
    let end = graph
        .refs()
        .vertices()
        .entry(tool::edge::end_vertex_id(&graph, edges[0]))
        .child_vertex_id;
    let new = graph
        .editor()
        .edges()
        .add(start, end, &Handle::default(), 0.0, 1.0, 1.0e-7);
    graph
        .editor()
        .wires()
        .replace_edge(wire, edges[0], new, false)
        .unwrap();
    assert_eq!(
        graph.topo().co_edges().definition(chain[0]).child_edge_id,
        new
    );
}

#[test]
fn a_moved_guard_is_moved_back_by_assignment() {
    let mut graph = box_graph();
    let mut first = graph.editor().vertices().mutate(VertexId::new(0)).unwrap();
    first.mark_dirty();
    let mut second = MutGuard::move_from(&mut first);
    assert!(!first.is_active() && second.is_active() && second.is_dirty());
    first.move_assign(&mut second);
    assert!(first.is_active() && first.is_dirty() && !second.is_active());
    assert_eq!(first.id(), VertexId::new(0));
}

#[test]
fn the_supplement_editor_attaches_with_each_owner_default_kind() {
    let mut graph = box_graph();
    let shape = make_vertex(Point3::new(1.0, 1.0, 1.0)).into();
    let compound = graph.editor().compounds().add(&[]);
    let comp_solid = graph.editor().comp_solids().add(&[]);
    let mut editor = SupplementEditor::new(&mut graph);
    let uids = [
        (
            editor
                .attach_to_vertex(VertexId::new(0), &shape, None)
                .unwrap(),
            AttachmentKind::VertexSupplementShape,
        ),
        (
            editor.attach_to_edge(EdgeId::new(0), &shape, None).unwrap(),
            AttachmentKind::EdgeInternalVertex,
        ),
        (
            editor.attach_to_face(FaceId::new(0), &shape, None).unwrap(),
            AttachmentKind::FaceDirectVertex,
        ),
        (
            editor
                .attach_to_solid(SolidId::new(0), &shape, None)
                .unwrap(),
            AttachmentKind::SolidAuxShape,
        ),
        (
            editor
                .attach_to_shell(ShellId::new(0), &shape, None)
                .unwrap(),
            AttachmentKind::ShellAuxShape,
        ),
        (
            editor.attach_to_compound(compound, &shape, None).unwrap(),
            AttachmentKind::CompoundAuxShape,
        ),
        (
            editor
                .attach_to_comp_solid(comp_solid, &shape, None)
                .unwrap(),
            AttachmentKind::CompSolidAuxShape,
        ),
    ];
    let generic = editor
        .attach(
            FaceId::new(1).into(),
            AttachmentKind::GenericSupplementShape,
            &shape,
        )
        .unwrap();
    assert_eq!(
        editor
            .attach(
                FaceId::new(99).into(),
                AttachmentKind::GenericSupplementShape,
                &shape
            )
            .unwrap(),
        0
    );
    let layer: Handle<LayerTopoSupplement> = graph.layer_registry().find::<LayerTopoSupplement>();
    for (uid, kind) in uids {
        assert_eq!(layer.find_by_uid(uid).unwrap().kind, kind);
    }
    assert!(graph.editor().supplement().remove_attachment(generic));
    assert!(!graph.editor().supplement().remove_attachment(generic));
}

#[test]
fn a_reborrowed_editor_serves_several_operation_groups() {
    let mut graph = BRepGraph::new();
    let mut editor = graph.editor();
    let a = editor
        .reborrow()
        .vertices()
        .add(Point3::new(0.0, 0.0, 0.0), 1.0e-7);
    let b = editor
        .reborrow()
        .vertices()
        .add(Point3::new(1.0, 0.0, 0.0), 1.0e-7);
    let edge = editor
        .edges()
        .add(a, b, &Handle::default(), 0.0, 1.0, 1.0e-7);
    assert!(edge.is_valid());
    assert_eq!(graph.topo().vertices().nb(), 2);
    let co_edge = graph
        .editor()
        .co_edges()
        .add(edge, ParityOrientation::REVERSED);
    assert!(co_edge.is_valid());
}
