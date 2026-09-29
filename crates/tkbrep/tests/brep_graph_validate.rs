//! Behaviour of `BRepGraph_Validate` on a box graph broken on purpose through mutation
//! guards: each audit check reports its issue text on the expected node, the
//! lightweight mode sees only the boundary, isolation and document-root checks, and the
//! reports keep the native order. The OCCT comparison of the same reports is the
//! `br_bedit` probe (tools/tkbrep_brepgraph_algo_corpus.py).
mod support;
use rustycad_tkbrep::brep_graph::validate::{self, Issue, Mode, Options, Severity};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkmath::toploc::Location;
use support::make_box;

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&make_box(10.0, 20.0, 30.0)).unwrap();
    graph
}

fn audit(graph: &BRepGraph) -> Vec<Issue> {
    validate::perform_with_mode(graph, Mode::Audit).issues
}

fn texts(issues: &[Issue]) -> Vec<String> {
    issues
        .iter()
        .map(|issue| String::from_utf8_lossy(issue.description.as_bytes()).into_owned())
        .collect()
}

fn has_issue(issues: &[Issue], node: NodeId, text: &str) -> bool {
    issues
        .iter()
        .any(|issue| issue.node_id == node && issue.description.is_equal(text))
}

#[test]
fn a_built_box_is_valid_in_both_modes() {
    let graph = box_graph();
    assert!(validate::perform(&graph).issues.is_empty());
    let result = validate::perform_with_options(&graph, &Options::audit());
    assert!(result.is_valid(), "{:?}", texts(&result.issues));
    assert_eq!(result.nb_issues(Severity::Warning), 0);
}

#[test]
fn a_missing_curve_is_an_edge_error_and_an_ownerless_record() {
    let mut graph = box_graph();
    let edge = EdgeId::new(0);
    let mut guard = graph.editor().edges().mutate(edge).unwrap();
    guard.internal().curve3d_rep_id = EdgeCurve3DRepId::default();
    drop(guard);
    let issues = audit(&graph);
    let text = "Non-degenerate EdgeDef has no Curve3D representation";
    assert!(has_issue(&issues, edge.into(), text));
    assert!(has_issue(
        &issues,
        NodeId::default(),
        "Active EdgeCurve3DRep has no owner"
    ));
    // The lightweight checks do not look at representations.
    assert!(validate::perform(&graph).is_valid());
}

#[test]
fn a_shared_curve_record_has_a_mismatched_parent_and_two_owners() {
    let mut graph = box_graph();
    let shared = graph
        .topo()
        .edges()
        .definition(EdgeId::new(0))
        .curve3d_rep_id;
    let mut guard = graph.editor().edges().mutate(EdgeId::new(1)).unwrap();
    guard.internal().curve3d_rep_id = shared;
    drop(guard);
    let issues = audit(&graph);
    let edge1 = NodeId::from(EdgeId::new(1));
    assert!(has_issue(
        &issues,
        edge1,
        "EdgeCurve3DRep.ParentEdgeId mismatch"
    ));
    let edge0 = NodeId::from(EdgeId::new(0));
    assert!(has_issue(
        &issues,
        edge0,
        "Active EdgeCurve3DRep has multiple owners"
    ));
}

#[test]
fn a_coedge_on_an_out_of_range_face_is_reported_before_the_owner_checks() {
    let mut graph = box_graph();
    let co_edge = CoEdgeId::new(0);
    let mut guard = graph.editor().co_edges().mutate(co_edge).unwrap();
    guard.internal().face_id = FaceId::new(999);
    drop(guard);
    let issues = audit(&graph);
    let bounds = issues
        .iter()
        .position(|issue| issue.description.is_equal("CoEdgeDef.FaceId out of bounds"));
    assert!(bounds.is_some());
    assert_eq!(issues[bounds.unwrap()].node_id, NodeId::from(co_edge));
}

#[test]
fn a_vertex_reference_on_a_removed_vertex_fails_the_lightweight_mode() {
    let mut graph = box_graph();
    let spare = graph
        .editor()
        .vertices()
        .add(rustycad_tkmath::gp::Point3::new(99.0, 0.0, 0.0), 1e-7);
    graph.editor().generic().remove_node(spare.into()).unwrap();
    let reference = graph
        .topo()
        .edges()
        .definition(EdgeId::new(0))
        .end_vertex_ref_id;
    let mut guard = graph.editor().vertices().mutate_ref(reference).unwrap();
    guard.internal().child_vertex_id = spare;
    drop(guard);
    let light = validate::perform(&graph);
    let text = "Non-removed EdgeDef references removed EndVertexEntity";
    assert!(has_issue(&light.issues, EdgeId::new(0).into(), text));
    assert!(!light.is_valid());
}

#[test]
fn a_compound_that_contains_itself_is_a_direct_cycle() {
    let mut graph = box_graph();
    let compound = graph.editor().compounds().add(&[SolidId::new(0).into()]);
    let reference = graph.refs().children().ids_of(compound)[0];
    let mut guard = graph
        .editor()
        .generic()
        .mutate_child_ref(reference)
        .unwrap();
    guard.internal().child_node_id = compound.into();
    drop(guard);
    let issues = audit(&graph);
    let text = "Compound cycle: Compound directly references itself via ChildRef";
    assert!(has_issue(&issues, compound.into(), text));
}

#[test]
fn a_solid_in_a_compound_and_a_compsolid_is_cross_owned() {
    let mut graph = box_graph();
    let solid = SolidId::new(0);
    graph.editor().compounds().add(&[solid.into()]);
    graph.editor().comp_solids().add(&[solid]);
    let issues = audit(&graph);
    let text = "Cross-container ownership: Solid is referenced by both a Compound (ChildRef) \
                and a CompSolid (SolidRef)";
    assert_eq!(texts(&issues).last().map(String::as_str), Some(text));
    assert_eq!(issues.last().unwrap().node_id, NodeId::from(solid));
}

#[test]
fn a_root_product_placed_by_an_occurrence_is_reported_in_both_modes() {
    let mut graph = box_graph();
    let part = ProductId::new(0);
    let assembly = graph.editor().products().add_empty();
    let default = OccurrenceId::default();
    let mut products = graph.editor().products();
    products
        .append(assembly, part, &Location::default(), default)
        .unwrap();
    products.append_document_root(part).unwrap();
    let text = "Document root Product is referenced by an active Occurrence";
    assert!(has_issue(
        &validate::perform(&graph).issues,
        part.into(),
        text
    ));
    assert!(has_issue(&audit(&graph), part.into(), text));
}

#[test]
fn an_occurrence_of_an_occurrence_is_refused_by_the_bounds_check() {
    let mut graph = box_graph();
    let occurrence = OccurrenceId::new(0);
    let mut guard = graph.editor().occurrences().mutate(occurrence).unwrap();
    guard.internal().child_node_id = occurrence.into();
    drop(guard);
    let issues = audit(&graph);
    let text = "OccurrenceDef.ChildNodeId cannot reference an Occurrence";
    assert!(has_issue(&issues, occurrence.into(), text));
}

#[test]
fn a_face_without_wires_is_only_a_warning() {
    let mut graph = box_graph();
    let reference = graph.refs().wires().ids_of(FaceId::new(0))[0];
    graph
        .editor()
        .faces()
        .remove_wire(FaceId::new(0), reference)
        .unwrap();
    let result = validate::perform_with_mode(&graph, Mode::Audit);
    let warning = "Surfaced face has no wire refs";
    assert!(has_issue(&result.issues, FaceId::new(0).into(), warning));
    assert_eq!(result.nb_issues(Severity::Warning), 1);
}

#[test]
fn a_reversed_second_coedge_breaks_the_wire_chain() {
    let mut graph = box_graph();
    let co_edge = graph.topo().wires().relations(WireId::new(0)).co_edge_ids[1];
    let mut guard = graph.editor().co_edges().mutate(co_edge).unwrap();
    let orientation = guard.internal().orientation;
    guard.internal().orientation = ParityOrientation {
        is_reversed: !orientation.is_reversed,
    };
    drop(guard);
    let issues = audit(&graph);
    let text = "Wire edges not connected: edge[0] end != edge[1] start";
    assert!(has_issue(&issues, WireId::new(0).into(), text));
}
