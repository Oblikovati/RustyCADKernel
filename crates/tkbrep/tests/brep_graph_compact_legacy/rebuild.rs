//! Bounds, topology and history mirrors from BRepGraph_Compact_Test.cxx.
use super::*;
use rustycad_tkbrep::brep_graph::tool;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

fn graph_vertex_bounds(graph: &BRepGraph) -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    for index in 0..graph.topo().vertices().nb() {
        let vertex = VertexId::new(index);
        if !vertex.is_removed(graph) {
            bounds.add_point(graph.topo().vertices().definition(vertex).point);
        }
    }
    bounds
}

fn assert_compacted_bounds(before: &BoundingBox3, after: &BoundingBox3) {
    assert!(!before.is_void());
    assert!(!after.is_void());
    for (left, right) in after.get().unwrap().into_iter().zip(before.get().unwrap()) {
        assert!((left - right).abs() <= rustycad_tkernel::precision::confusion());
    }
}

#[test]
fn BRepGraph_CompactTest__RemovalCompact_PreservesBounds_AndDoesNotGrowTopology() {
    let mut graph = compact_graph(&make_box(10.0, 20.0, 30.0));
    assert!(!graph.is_empty());
    assert!(graph.topo().co_edges().nb() > 0);
    assert!(graph.topo().faces().nb() > 2);
    graph
        .editor()
        .generic()
        .remove_node(FaceId::new(2).into())
        .unwrap();
    let coedges = graph.topo().co_edges().nb();
    let before = graph_vertex_bounds(&graph);
    let result = compact::perform(&mut graph).unwrap();
    assert!(result.nb_nodes_before >= result.nb_nodes_after);
    assert!(graph.topo().co_edges().nb() <= coedges);
    assert_compacted_bounds(&before, &graph_vertex_bounds(&graph));
    assert!(validate::perform(&graph).is_valid());
}

fn remove_loose_edge(graph: &mut BRepGraph) {
    let edge = (0..graph.topo().edges().nb())
        .map(EdgeId::new)
        .find(|edge| graph.topo().edges().faces_of(*edge).count() == 0)
        .unwrap();
    assert!(edge.is_valid());
    graph.editor().generic().remove_node(edge.into()).unwrap();
}

fn assert_rebuilt_closed_shape(graph: &BRepGraph) {
    assert_eq!(graph.topo().shells().nb(), 1);
    for index in 0..graph.topo().wires().nb() {
        assert!(tool::wire::is_closed(graph, WireId::new(index)));
    }
    assert!(tool::shell::is_closed(graph, ShellId::start()));
    let shape = graph
        .shapes()
        .reconstruct(graph.root_product_ids()[0].into())
        .unwrap();
    assert!(!shape.is_null());
    assert!(
        BRepCheckAnalyzer::new(&shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

#[test]
fn BRepGraph_CompactTest__RemovalCompact_PreservesClosedTopologyAndValidShape() {
    let mut graph = compact_graph(&box_with_loose_edge());
    assert!(!graph.is_empty());
    remove_loose_edge(&mut graph);
    let result = compact::perform(&mut graph).unwrap();
    assert!(result.nb_nodes_before > result.nb_nodes_after);
    assert_rebuilt_closed_shape(&graph);
    assert!(validate::perform(&graph).is_valid());
}

fn compound_boxes(shapes: &[Shape]) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape).unwrap();
    }
    compound.into()
}

fn three_copied_boxes() -> Shape {
    let original = make_box(10.0, 20.0, 30.0);
    let copies: Vec<Shape> = (0..3)
        .map(|_| {
            BRepBuilderAPICopy::with_shape(&original, true, false)
                .unwrap()
                .shape()
                .unwrap()
                .clone()
        })
        .collect();
    compound_boxes(&copies)
}

fn deduplicate_topology(graph: &mut BRepGraph) {
    let options = DeduplicateOptions {
        merge_entities_when_safe: true,
        ..DeduplicateOptions::default()
    };
    deduplicate::perform_with(graph, &options).unwrap();
}

#[test]
fn BRepGraph_CompactTest__Compact_AfterDedupMerge_NoBoundsErrors() {
    let mut graph = compact_graph(&three_copied_boxes());
    assert!(!graph.is_empty());
    deduplicate_topology(&mut graph);
    let result = compact::perform(&mut graph).unwrap();
    assert_ne!(result.nb_nodes_after, 0);
    assert_eq!(graph.topo().vertices().nb(), 8);
    assert_eq!(graph.topo().edges().nb(), 12);
    assert_eq!(graph.topo().faces().nb(), 6);
    assert!(validate::perform(&graph).is_valid());
}

fn record_vertex_modification(graph: &BRepGraph) -> usize {
    let history = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    history.set_enabled(true);
    let vertex = VertexId::start().into();
    history.record(
        graph,
        &AsciiString::new("Test:Modify"),
        vertex,
        &[vertex],
        HistoryKind::Modified,
    );
    let count = history.nb_records();
    assert!(count > 0);
    count
}

#[test]
fn BRepGraph_CompactTest__HistorySurvivesCompactWithRemappedIds() {
    let boxes = [
        make_box(10.0, 20.0, 30.0).into(),
        make_box(10.0, 20.0, 30.0).into(),
    ];
    let mut graph = compact_graph(&compound_boxes(&boxes));
    assert!(!graph.is_empty());
    let before = record_vertex_modification(&graph);
    deduplicate_topology(&mut graph);
    let options = compact::Options {
        history_mode: true,
        ..compact::Options::default()
    };
    compact::perform_with(&mut graph, &options).unwrap();
    let history = graph.layer_registry().ensure::<LayerHistory>().unwrap();
    assert!(history.nb_records() > before);
}
