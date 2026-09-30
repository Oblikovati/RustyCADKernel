//! Original BRepGraph_Convenience_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::{
    BRepGraph, CoEdgeId, CompSolidId, CompoundId, EdgeId, FaceId, NodeId, NodeKind, ShellId,
    SolidId, VertexId, WireId,
    iterator::{EdgeIterator, FaceIterator},
    tool::{co_edge, edge, vertex},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn convenience_graph() -> BRepGraph {
    let shape = BRepPrimAPIMakeBox::from_sizes(10.0, 20.0, 30.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(&shape).unwrap();
    graph
}
fn first_edge_vertices(graph: &BRepGraph) -> (VertexId, VertexId) {
    assert!(graph.topo().edges().nb() > 0);
    let first = vertex::usage(graph, edge::start_vertex_id(graph, EdgeId::new(0)));
    let last = vertex::usage(graph, edge::end_vertex_id(graph, EdgeId::new(0)));
    assert!(first.is_valid());
    assert!(last.is_valid());
    (first.def_id, last.def_id)
}
#[test]
fn BRepGraph_ConvenienceTest__NodeId_Factories_CorrectKindAndIndex() {
    let _graph = convenience_graph();
    let cases: [(NodeId, NodeKind, u32); 8] = [
        (SolidId::new(3).into(), NodeKind::SOLID, 3),
        (ShellId::new(5).into(), NodeKind::SHELL, 5),
        (FaceId::start().into(), NodeKind::FACE, 0),
        (WireId::new(2).into(), NodeKind::WIRE, 2),
        (EdgeId::new(7).into(), NodeKind::EDGE, 7),
        (VertexId::new(1).into(), NodeKind::VERTEX, 1),
        (CompoundId::start().into(), NodeKind::COMPOUND, 0),
        (CompSolidId::start().into(), NodeKind::COMP_SOLID, 0),
    ];
    for (id, kind, index) in cases {
        assert_eq!(id.kind, kind);
        assert_eq!(id.index, index);
    }
}
#[test]
fn BRepGraph_ConvenienceTest__NodeId_Factories_EqualToConstructor() {
    let _graph = convenience_graph();
    assert_eq!(NodeId::from(FaceId::new(3)), NodeId::new(NodeKind::FACE, 3));
    assert_eq!(
        NodeId::from(EdgeId::start()),
        NodeId::new(NodeKind::EDGE, 0)
    );
}
#[test]
fn BRepGraph_ConvenienceTest__EdgeDef_StartVertex_Valid() {
    let graph = convenience_graph();
    assert!(graph.topo().edges().nb() > 0);
    assert!(edge::start_vertex_id(&graph, EdgeId::new(0)).is_valid());
}
#[test]
fn BRepGraph_ConvenienceTest__EdgeDef_EndVertex_Valid() {
    let graph = convenience_graph();
    assert!(graph.topo().edges().nb() > 0);
    assert!(edge::end_vertex_id(&graph, EdgeId::new(0)).is_valid());
}
#[test]
fn BRepGraph_ConvenienceTest__EdgeDef_StartEnd_DifferForNonClosed() {
    let graph = convenience_graph();
    assert!(graph.topo().edges().nb() > 0);
    let edge_id = EdgeId::new(0);
    if !edge::is_closed(&graph, edge_id) {
        assert_ne!(
            edge::start_vertex_id(&graph, edge_id),
            edge::end_vertex_id(&graph, edge_id)
        );
    }
}
#[test]
fn BRepGraph_ConvenienceTest__EdgeDef_RefIds_AreValid() {
    let graph = convenience_graph();
    assert!(graph.topo().edges().nb() > 0);
    let edge = graph.topo().edges().definition(EdgeId::new(0));
    assert!(edge.start_vertex_ref_id.is_valid());
    assert!(edge.end_vertex_ref_id.is_valid());
}
#[test]
fn BRepGraph_ConvenienceTest__EdgeOps_FindByVertices_FindsDirectedEdge() {
    let graph = convenience_graph();
    let (first, last) = first_edge_vertices(&graph);
    assert_eq!(
        edge::find_by_vertices(&graph, first, last, false),
        EdgeId::new(0)
    );
}
#[test]
fn BRepGraph_ConvenienceTest__EdgeOps_FindByVertices_ReverseRequiresExplicitFlag() {
    let graph = convenience_graph();
    let (first, last) = first_edge_vertices(&graph);
    assert_ne!(first, last);
    assert!(!edge::find_by_vertices(&graph, last, first, false).is_valid());
    assert_eq!(
        edge::find_by_vertices(&graph, last, first, true),
        EdgeId::new(0)
    );
}
#[test]
fn BRepGraph_ConvenienceTest__FaceSurface_Valid() {
    let graph = convenience_graph();
    assert!(graph.topo().faces().nb() > 0);
    assert!(
        graph
            .topo()
            .faces()
            .definition(FaceId::start())
            .surface_rep_id
            .is_valid()
    );
}
#[test]
fn BRepGraph_ConvenienceTest__FaceSurface_AllBoxFaces() {
    let graph = convenience_graph();
    for face in FaceIterator::new(&graph) {
        assert!(face.surface_rep_id.is_valid());
    }
}
fn first_pcurve_of_face(graph: &BRepGraph, face: FaceId) -> Option<CoEdgeId> {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let coedge = edge::find_pcurve_co_edge_id(graph, edges.current_id(), face);
        if coedge.is_valid() {
            return Some(coedge);
        }
        edges.next();
    }
    None
}
#[test]
fn BRepGraph_ConvenienceTest__FindPCurveCoEdgeId_ValidPair() {
    let graph = convenience_graph();
    let mut faces = FaceIterator::new(&graph);
    while faces.more() {
        if let Some(coedge) = first_pcurve_of_face(&graph, faces.current_id()) {
            assert!(
                graph
                    .topo()
                    .co_edges()
                    .definition(coedge)
                    .curve2d_rep_id
                    .is_valid()
            );
            return;
        }
        faces.next();
    }
    // Upstream deliberately has no assertion when no pair is found.
}
#[test]
fn BRepGraph_ConvenienceTest__FindPCurveCoEdgeId_InvalidPair_ReturnsNull() {
    let graph = convenience_graph();
    assert!(!edge::find_pcurve_co_edge_id(&graph, EdgeId::start(), FaceId::new(9999)).is_valid());
}
#[test]
fn BRepGraph_ConvenienceTest__ShellFaceRefs_Box_SixFaces() {
    let graph = convenience_graph();
    assert_eq!(graph.topo().shells().nb(), 1);
    assert_eq!(graph.refs().faces().ids_of(ShellId::start()).len(), 6);
}
#[test]
fn BRepGraph_ConvenienceTest__ShellFaceRefs_AllValid() {
    let graph = convenience_graph();
    for id in graph.refs().faces().ids_of(ShellId::start()) {
        assert!(graph.refs().faces().entry(*id).child_face_id.is_valid());
    }
}
#[test]
fn BRepGraph_ConvenienceTest__ShellFaceRefs_InvalidShell_Empty() {
    let graph = convenience_graph();
    assert!(graph.refs().faces().ids_of(ShellId::new(100)).is_empty());
}
#[test]
fn BRepGraph_ConvenienceTest__ShapesView_RemoveShape_RemovesFoundNode() {
    let mut graph = convenience_graph();
    assert!(graph.topo().faces().nb() > 0);
    let id = FaceId::start().into();
    let shape = graph.shapes().original(id);
    assert!(!shape.is_null());
    assert!(graph.shapes().find_node(&shape).is_valid());
    assert!(graph.shapes_mut().remove_shape(&shape).unwrap());
    assert!(!graph.shapes().find_node(&shape).is_valid());
    assert!(graph.topo().generic().is_removed(id));
    assert!(!graph.shapes_mut().remove_shape(&shape).unwrap());
}
fn first_edge_seam(graph: &BRepGraph) -> Option<(EdgeId, FaceId)> {
    let mut edges = EdgeIterator::new(graph);
    while edges.more() {
        let edge = edges.current_id();
        if let Some(id) = graph
            .topo()
            .edges()
            .co_edges(edge)
            .iter()
            .find(|id| co_edge::is_seam(graph, **id))
        {
            return Some((edge, graph.topo().co_edges().definition(*id).face_id));
        }
        edges.next();
    }
    None
}
#[test]
fn BRepGraph_ConvenienceTest__FindPCurveCoEdgeId_WithOrientation_SeamEdge() {
    let _fixture = convenience_graph();
    let shape = BRepPrimAPIMakeCylinder::new(5.0, 10.0)
        .shape()
        .unwrap()
        .clone();
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    assert!(graph.shapes_mut().add(&shape).unwrap().is_ok());
    let Some((edge, face)) = first_edge_seam(&graph) else {
        return;
    };
    let forward = edge::find_pcurve_co_edge_id_oriented(&graph, edge, face, Orientation::Forward);
    let reversed = edge::find_pcurve_co_edge_id_oriented(&graph, edge, face, Orientation::Reversed);
    assert!(forward.is_valid());
    assert!(reversed.is_valid());
    assert_ne!(forward, reversed);
}
