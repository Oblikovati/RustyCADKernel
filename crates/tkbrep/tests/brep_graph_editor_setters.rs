//! Behaviour of the editor setters that the translated suites leave out, on a box
//! graph: coedge p-curves, polygons and bindings, edge ranges, curves and vertex
//! references, vertex points and reference rebinding, reference retargeting, the
//! generic child-reference setters and their guarded forms. The OCCT comparison of the
//! same members is the `br_bedit` probe (tools/tkbrep_brepgraph_edit_corpus.py).
mod support;
use rustycad_tkbrep::brep_graph::editor::{CoEdgeOps, EdgeOps, GenOps, VertexOps};
use rustycad_tkbrep::brep_graph::tool;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Direction2, Direction3, Point2, Point3, Transform3, Vector3,
};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation};
use rustycad_tkmath::toploc::Location;
use support::make_box;

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&make_box(10.0, 20.0, 30.0)).unwrap();
    graph
}

fn line2d() -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Handle::new(Geom2dLine::new(axis)).cast()
}

fn line3d() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(Point3::default(), Direction3::X))).into_cast()
}

fn own_gen_of_co_edge(graph: &BRepGraph, co_edge: CoEdgeId) -> u32 {
    graph.topo().co_edges().definition(co_edge).base.own_gen
}

#[test]
fn a_pcurve_is_set_with_or_without_its_range_then_cleared() {
    let mut graph = box_graph();
    let co_edge = CoEdgeId::new(0);
    let curve = line2d();
    graph
        .editor()
        .co_edges()
        .set_pcurve(co_edge, &curve, 0.25, 0.75)
        .unwrap();
    assert!(tool::co_edge::pcurve(&graph, co_edge) == curve);
    assert_eq!(tool::co_edge::range(&graph, co_edge), (0.25, 0.75));
    let other = line2d();
    graph
        .editor()
        .co_edges()
        .set_pcurve_keep_range(co_edge, &other)
        .unwrap();
    assert!(tool::co_edge::pcurve(&graph, co_edge) == other);
    assert_eq!(tool::co_edge::range(&graph, co_edge), (0.25, 0.75));
    graph.editor().co_edges().clear_pcurve(co_edge).unwrap();
    assert!(!tool::co_edge::has_pcurve(&graph, co_edge));
}

#[test]
fn a_coedge_range_is_set_directly_and_through_a_guard() {
    let mut graph = box_graph();
    let co_edge = CoEdgeId::new(1);
    graph
        .editor()
        .co_edges()
        .set_param_range(co_edge, 1.0, 2.0)
        .unwrap();
    assert_eq!(tool::co_edge::range(&graph, co_edge), (1.0, 2.0));
    let before = own_gen_of_co_edge(&graph, co_edge);
    let mut guard = graph.editor().co_edges().mutate(co_edge).unwrap();
    CoEdgeOps::set_param_range_guarded(&mut guard, 3.0, 4.0);
    assert!(guard.is_dirty());
    drop(guard);
    assert_eq!(tool::co_edge::range(&graph, co_edge), (3.0, 4.0));
    assert!(own_gen_of_co_edge(&graph, co_edge) > before);
}

#[test]
fn a_pcurve_binding_reset_unbinds_the_record() {
    let mut graph = box_graph();
    let co_edge = CoEdgeId::new(2);
    assert!(tool::co_edge::has_pcurve(&graph, co_edge));
    let before = own_gen_of_co_edge(&graph, co_edge);
    graph
        .editor()
        .co_edges()
        .reset_pcurve_binding(co_edge)
        .unwrap();
    let definition = graph.topo().co_edges().definition(co_edge);
    assert!(!definition.curve2d_rep_id.is_valid());
    assert!(!tool::co_edge::has_pcurve(&graph, co_edge));
    assert!(own_gen_of_co_edge(&graph, co_edge) > before);
    let mut guard = graph.editor().co_edges().mutate(CoEdgeId::new(3)).unwrap();
    CoEdgeOps::reset_pcurve_binding_guarded(&mut guard);
    assert!(guard.is_dirty());
    drop(guard);
    assert!(!tool::co_edge::has_pcurve(&graph, CoEdgeId::new(3)));
}

#[test]
fn coedge_polygons_become_persistent_representations() {
    let mut graph = box_graph();
    let co_edge = CoEdgeId::new(0);
    let polygon = Handle::new(Polygon2D::new(2).unwrap());
    let on_tri = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    let mut ops = graph.editor().co_edges();
    ops.set_persistent_polygon2d(co_edge, &polygon).unwrap();
    ops.set_persistent_polygon_on_tri(co_edge, &on_tri).unwrap();
    let persistent = graph.mesh().persistent().co_edges();
    assert!(persistent.polygon_on_surface(co_edge) == polygon);
    assert!(persistent.polygon_on_triangulation(co_edge) == on_tri);
}

#[test]
fn a_coedge_is_retargeted_to_another_edge_and_face() {
    let mut graph = box_graph();
    let co_edge = CoEdgeId::new(0);
    let old_face = graph.topo().co_edges().definition(co_edge).face_id;
    let (edge, face) = (EdgeId::new(11), FaceId::new(5));
    let mut ops = graph.editor().co_edges();
    ops.set_child_edge_id(co_edge, edge).unwrap();
    ops.set_face_id(co_edge, face).unwrap();
    let definition = graph.topo().co_edges().definition(co_edge);
    assert_eq!((definition.child_edge_id, definition.face_id), (edge, face));
    let mut guard = graph.editor().co_edges().mutate(co_edge).unwrap();
    CoEdgeOps::set_face_id_guarded(&mut guard, old_face);
    CoEdgeOps::set_child_edge_id_guarded(&mut guard, EdgeId::new(0));
    drop(guard);
    let definition = graph.topo().co_edges().definition(co_edge);
    assert_eq!(
        (definition.child_edge_id, definition.face_id),
        (EdgeId::new(0), old_face)
    );
}

#[test]
fn an_edge_range_curve_and_polygon_are_rewritten() {
    let mut graph = box_graph();
    let edge = EdgeId::new(0);
    let curve = line3d();
    let polygon = Handle::new(Polygon3D::new(2, false).unwrap());
    let mut ops = graph.editor().edges();
    ops.set_param_range(edge, 0.5, 1.5).unwrap();
    ops.set_curve(EdgeId::new(1), &curve, -1.0, 1.0).unwrap();
    ops.set_persistent_polygon3d(edge, &polygon).unwrap();
    assert_eq!(tool::edge::range(&graph, edge), (0.5, 1.5));
    assert!(tool::edge::curve(&graph, EdgeId::new(1)) == curve);
    assert_eq!(tool::edge::range(&graph, EdgeId::new(1)), (-1.0, 1.0));
    assert!(graph.mesh().persistent().edges().polygon3d(edge) == polygon);
    let mut guard = graph.editor().edges().mutate(edge).unwrap();
    EdgeOps::set_param_range_guarded(&mut guard, 2.0, 3.0);
    drop(guard);
    assert_eq!(tool::edge::range(&graph, edge), (2.0, 3.0));
}

#[test]
fn a_boundary_reference_is_swapped_for_an_unowned_one() {
    let mut graph = box_graph();
    let edge = EdgeId::new(0);
    let old = tool::edge::start_vertex_id(&graph, edge);
    // ReplaceVertex puts a fresh reference in the slot; the replaced one is removed.
    let fresh = graph
        .editor()
        .edges()
        .replace_vertex(edge, old, VertexId::new(7))
        .unwrap();
    assert!(fresh.is_valid() && fresh != old);
    assert_eq!(tool::edge::start_vertex_id(&graph, edge), fresh);
    assert!(RefId::from(old).is_removed(&graph));
    // An owned reference (the edge's own end) is not rebound as the start.
    let end = tool::edge::end_vertex_id(&graph, edge);
    graph
        .editor()
        .edges()
        .set_start_vertex_ref_id(edge, end)
        .unwrap();
    assert_eq!(tool::edge::start_vertex_id(&graph, edge), fresh);
}

#[test]
fn a_boundary_vertex_reference_is_detached_once() {
    let mut graph = box_graph();
    let edge = EdgeId::new(3);
    let end = tool::edge::end_vertex_id(&graph, edge);
    assert!(graph.editor().edges().remove_vertex(edge, end).unwrap());
    assert!(!graph.editor().edges().remove_vertex(edge, end).unwrap());
    assert!(RefId::from(end).is_removed(&graph));
    let mut guard = graph.editor().edges().mutate(edge).unwrap();
    EdgeOps::set_end_vertex_ref_id_guarded(&mut guard, VertexRefId::invalid()).unwrap();
    drop(guard);
    assert!(!tool::edge::end_vertex_id(&graph, edge).is_valid());
    graph
        .editor()
        .edges()
        .set_end_vertex_ref_id(edge, VertexRefId::invalid())
        .unwrap();
}

#[test]
fn a_vertex_point_and_reference_child_are_rewritten() {
    let mut graph = box_graph();
    let point = Point3::new(1.0, 2.0, 3.0);
    graph
        .editor()
        .vertices()
        .set_point(VertexId::new(0), point)
        .unwrap();
    assert_eq!(
        graph.topo().vertices().definition(VertexId::new(0)).point,
        point
    );
    let reference = VertexRefId::new(0);
    graph
        .editor()
        .vertices()
        .set_ref_child_vertex_id(reference, VertexId::new(5))
        .unwrap();
    assert_eq!(
        graph.refs().vertices().entry(reference).child_vertex_id,
        VertexId::new(5)
    );
    let mut guard = graph.editor().vertices().mutate_ref(reference).unwrap();
    VertexOps::set_ref_child_vertex_id_guarded(&mut guard, VertexId::new(6));
    drop(guard);
    assert_eq!(
        graph.refs().vertices().entry(reference).child_vertex_id,
        VertexId::new(6)
    );
}

#[test]
fn references_are_retargeted_to_other_children() {
    let mut graph = box_graph();
    let mut faces = graph.editor().faces();
    faces
        .set_ref_face_id(FaceRefId::new(0), FaceId::new(1))
        .unwrap();
    assert_eq!(
        graph.refs().faces().entry(FaceRefId::new(0)).child_face_id,
        FaceId::new(1)
    );
    graph
        .editor()
        .wires()
        .set_ref_child_wire_id(WireRefId::new(0), WireId::new(1))
        .unwrap();
    assert_eq!(
        graph.refs().wires().entry(WireRefId::new(0)).child_wire_id,
        WireId::new(1)
    );
    let extra = graph.editor().shells().add();
    graph
        .editor()
        .shells()
        .set_ref_child_shell_id(ShellRefId::new(0), extra)
        .unwrap();
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
fn child_references_take_a_location_orientation_and_child() {
    let mut graph = box_graph();
    let compound = graph.editor().compounds().add(&[SolidId::new(0).into()]);
    let reference = ChildRefId::new(graph.refs().children().nb() - 1);
    let mut moved = Transform3::default();
    moved.set_translation(Vector3::new(1.0, 0.0, 0.0));
    let location = Location::new(moved);
    let mut generic = graph.editor().generic();
    generic
        .set_child_ref_local_location(reference, &location)
        .unwrap();
    generic
        .set_child_ref_orientation(reference, ParityOrientation::REVERSED)
        .unwrap();
    generic
        .set_child_ref_child_node_id(reference, FaceId::new(0).into())
        .unwrap();
    let entry = graph.refs().children().entry(reference);
    assert_eq!(entry.parent_compound_id, compound);
    assert!(entry.local_location.is_equal(&location));
    assert_eq!(entry.orientation, ParityOrientation::REVERSED);
    assert_eq!(entry.child_node_id, FaceId::new(0));
    let mut guard = graph
        .editor()
        .generic()
        .mutate_child_ref(reference)
        .unwrap();
    GenOps::set_child_ref_local_location_guarded(&mut guard, &Location::default());
    GenOps::set_child_ref_child_node_id_guarded(&mut guard, SolidId::new(0).into());
    drop(guard);
    let entry = graph.refs().children().entry(reference);
    assert!(entry.local_location.is_identity());
    assert_eq!(entry.child_node_id, SolidId::new(0));
}
