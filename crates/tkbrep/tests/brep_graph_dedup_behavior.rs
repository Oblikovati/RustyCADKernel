//! Behaviour of BRepGraph_Deduplicate on graphs built through the editor (the upstream
//! suite also needs BRepBuilderAPI, BRepCheck and TKPrim): geometry rewrites, the
//! vertex, edge, wire and face merges, analysis without changes, the history records
//! and switch, and the removed-edge raise of the vertex redirect.
mod support;
use rustycad_tkbrep::brep_graph::deduplicate::{self, Options};
use rustycad_tkbrep::brep_graph::layer_history::LayerHistory;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point3};

fn plane() -> Handle<dyn GeomSurface> {
    let normal = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let x = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let axis = Axis3::new(Point3::new(0.0, 0.0, 0.0), normal, x).unwrap();
    Handle::new(GeomPlane::new(axis)).into_cast()
}

fn line(from: (f64, f64), to: (f64, f64)) -> Handle<dyn rustycad_tkg3d::geom::GeomCurve> {
    let direction = Direction3::new(to.0 - from.0, to.1 - from.1, 0.0).unwrap();
    let axis = Axis1::new(Point3::new(from.0, from.1, 0.0), direction);
    Handle::new(GeomLine::new(axis)).into_cast()
}

const CORNERS: [(f64, f64); 3] = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];

// One triangle face on its own plane, lines, vertices, edges, coedges and wire.
fn add_triangle(graph: &mut BRepGraph) -> FaceId {
    let vertices: Vec<VertexId> = CORNERS
        .iter()
        .map(|&(x, y)| graph.editor().vertices().add(Point3::new(x, y, 0.0), 1e-7))
        .collect();
    let mut co_edges = Vec::new();
    for index in 0..3 {
        let (from, to) = (CORNERS[index], CORNERS[(index + 1) % 3]);
        let length = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
        let edge = graph.editor().edges().add(
            vertices[index],
            vertices[(index + 1) % 3],
            &line(from, to),
            0.0,
            length,
            1e-7,
        );
        co_edges.push(
            graph
                .editor()
                .co_edges()
                .add(edge, ParityOrientation::FORWARD),
        );
    }
    let wire = graph.editor().wires().add(&co_edges);
    graph.editor().faces().add(&plane(), wire, &[], 1e-7)
}

fn twin_triangles() -> BRepGraph {
    let mut graph = BRepGraph::new();
    add_triangle(&mut graph);
    add_triangle(&mut graph);
    graph
}

fn history(graph: &BRepGraph) -> Handle<LayerHistory> {
    graph.layer_registry().ensure::<LayerHistory>().unwrap()
}

fn labels(graph: &BRepGraph) -> Vec<String> {
    let history = history(graph);
    let history = history.get().unwrap();
    (0..history.nb_records())
        .map(|index| {
            history
                .record_at(index, |event| {
                    String::from_utf8_lossy(event.operation_name.as_bytes()).into_owned()
                })
                .unwrap()
        })
        .collect()
}

fn merge_options() -> Options {
    Options {
        merge_entities_when_safe: true,
        ..Options::default()
    }
}

#[test]
fn equal_geometry_is_rewritten_to_the_first_owner() {
    let mut graph = twin_triangles();
    let result = deduplicate::perform(&mut graph).unwrap();
    assert_eq!(
        (result.nb_canonical_surfaces, result.nb_canonical_curves),
        (1, 3)
    );
    assert_eq!(
        (result.nb_surface_rewrites, result.nb_curve_rewrites),
        (1, 3)
    );
    assert_eq!(result.affected_faces, vec![FaceId::new(1)]);
    assert_eq!(result.affected_edges.len(), 3);
    assert!(!result.is_entity_merge_applied);
    let (first, second) = (
        graph.topo().faces().surface(FaceId::new(0)),
        graph.topo().faces().surface(FaceId::new(1)),
    );
    assert_eq!(first.object_address(), second.object_address());
    assert_eq!(result.nb_history_records, 4);
    let labels = labels(&graph);
    assert_eq!(labels[0], "Dedup:CanonicalizeSurface");
    assert_eq!(labels[1..], ["Dedup:CanonicalizeCurve"; 3]);
}

#[test]
fn analysis_counts_without_changing_the_graph() {
    let mut graph = twin_triangles();
    let options = Options {
        analyze_only: true,
        ..merge_options()
    };
    let result = deduplicate::perform_with(&mut graph, &options).unwrap();
    assert_eq!(result.nb_canonical_surfaces, 1);
    assert_eq!(result.nb_surface_rewrites, 0);
    assert_eq!(result.nb_merged_vertices, 3);
    assert!(!result.is_entity_merge_applied);
    assert!((0..6).all(|index| !VertexId::new(index).is_removed(&graph)));
    assert_eq!(history(&graph).get().unwrap().nb_records(), 0);
}

#[test]
fn the_merges_collapse_the_twin_triangle() {
    let mut graph = twin_triangles();
    let result = deduplicate::perform_with(&mut graph, &merge_options()).unwrap();
    assert!(result.is_entity_merge_applied);
    let merged = [
        result.nb_merged_vertices,
        result.nb_merged_edges,
        result.nb_merged_wires,
        result.nb_merged_faces,
    ];
    assert_eq!(merged, [3, 3, 1, 1]);
    assert_eq!(result.nb_history_records, 4 + 8);
    assert!(FaceId::new(1).is_removed(&graph));
    assert!(WireId::new(1).is_removed(&graph));
    assert!((3..6).all(|index| EdgeId::new(index).is_removed(&graph)));
    let labels = labels(&graph);
    assert_eq!(labels.last().unwrap(), "Dedup:MergeFace");
    assert!(labels.contains(&"Dedup:MergeVertex".to_string()));
    assert!(graph.validate_relations());
}

#[test]
fn without_history_mode_nothing_is_recorded_but_the_count_is_kept() {
    let mut graph = twin_triangles();
    let options = Options {
        history_mode: false,
        ..Options::default()
    };
    let result = deduplicate::perform_with(&mut graph, &options).unwrap();
    assert_eq!(result.nb_history_records, 4);
    let history = history(&graph);
    assert_eq!(history.get().unwrap().nb_records(), 0);
    assert!(history.get().unwrap().is_enabled());
}

#[test]
fn a_removed_edge_makes_the_vertex_redirect_raise() {
    let mut graph = twin_triangles();
    graph
        .editor()
        .generic()
        .remove_node(EdgeId::new(5).into())
        .unwrap();
    history(&graph).get().unwrap().set_enabled(false);
    let options = Options {
        history_mode: true,
        ..merge_options()
    };
    let error = deduplicate::perform_with(&mut graph, &options).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    // The raise skips the restore: the history keeps HistoryMode.
    assert!(history(&graph).get().unwrap().is_enabled());
    assert!(!graph.editor_state().is_deferred_mode());
}

#[test]
fn an_empty_graph_is_left_alone() {
    let result = deduplicate::perform(&mut BRepGraph::new()).unwrap();
    assert_eq!(result, Default::default());
}
