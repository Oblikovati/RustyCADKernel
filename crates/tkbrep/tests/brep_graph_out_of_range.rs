//! Regression tests for ids past their store (docs/topology-design.md, section 11.14).
//!
//! The native `Definition()`, `Relations()`, `Entry()` and `BRepGraph_Tool` members,
//! and the editor setters, index an `NCollection_DynamicArray` without a range check:
//! an id past the store is undefined behaviour. The fast Rust accessors panic; each
//! family has a checked form that returns Standard_OutOfRange with the offending id and
//! the valid range, and the library members that take a caller id and write or copy
//! through it use the checked form, so no caller input panics them.
mod support;
use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use support::make_box;

const FORWARD: ParityOrientation = ParityOrientation::FORWARD;

// A box added as a part: 8 vertices, 12 edges, 24 coedges, 6 wires and faces, one
// shell and solid, one product and one occurrence, no compound.
fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&make_box(1.0, 2.0, 3.0)).unwrap();
    graph
}

// The failure is Standard_OutOfRange and names the id text and the valid range.
fn assert_out_of_range<T>(result: Result<T, Failure>, id_text: &str, nb: u32) {
    let Err(failure) = result else {
        panic!("{id_text} was accepted");
    };
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    let expected = format!("{id_text} is out of range; expected 0..{nb}");
    let message = failure.message();
    assert!(
        message.contains(&expected),
        "{message:?} lacks {expected:?}"
    );
}

// The call was refused with Standard_OutOfRange.
fn refused<T>(result: Result<T, Failure>) -> bool {
    result.is_err_and(|failure| failure.exception_type() == "Standard_OutOfRange")
}

#[test]
fn views_have_checked_definition_relations_and_entry() {
    let graph = box_graph();
    let topo = graph.topo();
    let edge = topo.edges().try_definition(EdgeId::new(3)).unwrap();
    assert_eq!(
        edge.tolerance,
        topo.edges().definition(EdgeId::new(3)).tolerance
    );
    let co_edge = topo.co_edges().try_definition(CoEdgeId::new(24));
    assert_out_of_range(co_edge, "BRepGraph_CoEdgeId index 24", 24);
    assert!(refused(topo.products().try_definition(ProductId::new(1))));
    assert!(refused(
        topo.occurrences().try_relations(OccurrenceId::new(1))
    ));
    let shell = topo.shells().try_relations(ShellId::new(0)).unwrap();
    assert_eq!(shell.face_ref_ids.len(), 6);
    assert!(refused(topo.wires().try_relations(WireId::new(6))));
    let refs = graph.refs();
    assert!(refs.faces().try_entry(FaceRefId::new(0)).is_ok());
    assert!(refused(refs.vertices().try_entry(VertexRefId::new(9999))));
    assert!(refused(refs.children().try_entry(ChildRefId::new(0))));
}

#[test]
fn untyped_ids_are_checked_by_kind() {
    let graph = box_graph();
    let nodes = graph.topo().generic();
    assert_eq!(nodes.check_in_range(EdgeId::new(11).into()).unwrap(), 11);
    let past = nodes.check_in_range(EdgeId::new(12).into());
    assert_out_of_range(past, "BRepGraph_NodeId of kind 4 index 12", 12);
    assert!(refused(nodes.check_in_range(NodeId::default())));
    let reference = RefId::from(FaceRefId::new(6));
    assert!(refused(graph.refs().generic().check_in_range(reference)));
}

#[test]
fn tool_members_have_one_checked_form() {
    let graph = box_graph();
    let vertex = VertexId::new(7);
    let point = tool::checked_member(&graph, vertex, tool::vertex::pnt).unwrap();
    assert_eq!(point, tool::vertex::pnt(&graph, vertex));
    let past = tool::checked_member(&graph, VertexId::new(8), tool::vertex::pnt);
    assert_out_of_range(past, "BRepGraph_VertexId index 8", 8);
    assert!(refused(tool::checked_member(
        &graph,
        EdgeId::new(12),
        tool::edge::range
    )));
    let co_edge = CoEdgeId::new(24);
    assert!(refused(tool::checked_member(
        &graph,
        co_edge,
        tool::co_edge::orientation
    )));
    assert!(refused(tool::checked_member(
        &graph,
        FaceId::new(6),
        tool::face::nb_wires
    )));
    assert!(refused(tool::checked_member(
        &graph,
        WireId::new(6),
        tool::wire::face_of
    )));
    assert!(refused(tool::checked_member(
        &graph,
        ShellId::new(1),
        tool::shell::nb_faces
    )));
    let outer = tool::checked_member(&graph, FaceId::new(0), tool::face::outer_wire);
    assert!(outer.unwrap().unwrap().is_valid());
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn fast_accessors_keep_the_unchecked_contract() {
    let graph = box_graph();
    let _ = tool::vertex::pnt(&graph, VertexId::new(8));
}

#[test]
fn node_setters_refuse_ids_past_the_store() {
    let mut graph = box_graph();
    let mut editor = graph.editor();
    let (edge, co_edge, face) = (EdgeId::new(12), CoEdgeId::new(24), FaceId::new(6));
    let point = Point3::default();
    assert!(refused(
        editor
            .reborrow()
            .vertices()
            .set_point(VertexId::new(8), point)
    ));
    assert!(refused(editor.reborrow().edges().set_tolerance(edge, 1.0)));
    assert!(refused(
        editor.reborrow().edges().set_param_range(edge, 0.0, 1.0)
    ));
    assert!(refused(editor.reborrow().edges().clear_curve(edge)));
    let start = VertexRefId::new(0);
    assert!(refused(
        editor
            .reborrow()
            .edges()
            .set_start_vertex_ref_id(edge, start)
    ));
    assert!(refused(
        editor
            .reborrow()
            .co_edges()
            .set_param_range(co_edge, 0.0, 1.0)
    ));
    assert!(refused(
        editor
            .reborrow()
            .co_edges()
            .set_orientation(co_edge, FORWARD)
    ));
    assert!(refused(
        editor
            .reborrow()
            .co_edges()
            .set_face_id(co_edge, FaceId::new(0))
    ));
    let past = editor.reborrow().faces().set_tolerance(face, 1.0);
    assert_out_of_range(past, "BRepGraph_FaceId index 6", 6);
    assert!(refused(editor.reborrow().faces().clear_surface(face)));
    // A setter that checks its id natively keeps the native answer: ignored.
    let mut occurrences = editor.reborrow().occurrences();
    let child = NodeId::from(SolidId::new(0));
    assert!(
        occurrences
            .set_child_node_id(OccurrenceId::new(1), child)
            .is_ok()
    );
    // An id in range is still written.
    editor.faces().set_tolerance(FaceId::new(5), 0.5).unwrap();
    assert_eq!(
        graph.topo().faces().definition(FaceId::new(5)).tolerance,
        0.5
    );
}

#[test]
fn reference_setters_refuse_ids_past_the_store() {
    let mut graph = box_graph();
    let mut editor = graph.editor();
    let location = Location::default();
    let (vertex, child) = (VertexRefId::new(99), ChildRefId::new(0));
    assert!(refused(
        editor
            .reborrow()
            .vertices()
            .set_ref_orientation(vertex, FORWARD)
    ));
    let face = FaceId::new(0);
    assert!(refused(
        editor
            .reborrow()
            .faces()
            .set_ref_face_id(FaceRefId::new(6), face)
    ));
    let wire = WireId::new(0);
    assert!(refused(
        editor
            .reborrow()
            .wires()
            .set_ref_child_wire_id(WireRefId::new(6), wire)
    ));
    let shell_ref = ShellRefId::new(1);
    assert!(refused(
        editor
            .reborrow()
            .shells()
            .set_ref_orientation(shell_ref, FORWARD)
    ));
    let (solid_ref, solid) = (SolidRefId::new(0), SolidId::new(0));
    assert!(refused(
        editor
            .reborrow()
            .solids()
            .set_ref_child_solid_id(solid_ref, solid)
    ));
    let occurrence_ref = OccurrenceRefId::new(1);
    let mut occurrences = editor.reborrow().occurrences();
    assert!(refused(
        occurrences.set_ref_local_location(occurrence_ref, &location)
    ));
    assert!(refused(
        editor
            .reborrow()
            .generic()
            .set_child_ref_orientation(child, FORWARD)
    ));
    let vertex_node = NodeId::from(VertexId::new(0));
    assert!(refused(
        editor
            .reborrow()
            .compounds()
            .replace_child(child, vertex_node)
    ));
    assert!(refused(
        editor
            .reborrow()
            .comp_solids()
            .replace_solid(solid_ref, solid)
    ));
    let removed = editor.generic().remove_ref(RefId::from(FaceRefId::new(6)));
    assert_out_of_range(removed, "BRepGraph_RefId of kind 1 index 6", 6);
}

#[test]
fn copy_and_transform_refuse_assembly_ids_past_the_store() {
    let graph = box_graph();
    let (geom, mesh, cache) = (GeomPolicy::Copy, MeshPolicy::Copy, CachePolicy::Drop);
    let mut target = BRepGraph::new();
    let product = NodeId::from(ProductId::new(3));
    let copied = copy::copy_node(&graph, &mut target, product, geom, mesh, cache);
    assert_out_of_range(copied, "BRepGraph_ProductId index 3", 1);
    assert!(target.is_empty(), "a refused copy appends nothing");
    let occurrence = NodeId::from(OccurrenceId::new(1));
    assert!(refused(copy::copy_node(
        &graph,
        &mut target,
        occurrence,
        geom,
        mesh,
        cache
    )));
    let mut in_place = box_graph();
    assert!(refused(copy::copy_node_in_place(
        &mut in_place,
        product,
        geom,
        mesh,
        cache
    )));
    assert_eq!(in_place.topo().products().nb(), 1);
    let (share, drop) = (GeomPolicy::Share, MeshPolicy::Drop);
    let identity = Transform3::default();
    let moved = transform::transform_node(&graph, &mut target, occurrence, &identity, share, drop);
    assert!(refused(moved));
    // Other kinds keep the native answer: the invalid id, no failure.
    let face = NodeId::from(FaceId::new(6));
    let copied = copy::copy_node(&graph, &mut target, face, geom, mesh, cache);
    assert!(!copied.unwrap().is_valid());
}
