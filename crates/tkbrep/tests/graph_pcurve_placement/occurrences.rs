// SPDX-License-Identifier: GPL-2.0-only
use super::fixtures::{imported_v2, single_use_face, translated};
use rustycad_tkbrep::{
    brep_graph::{BRepGraph, CoEdgeId, CompoundId, tool::co_edge},
    topods::{Builder, Compound, Shell},
};
#[test]
fn cumulative_v2_preserves_distinct_shell_face_definitions() {
    let face = single_use_face();
    let moved = translated(&face);
    let mut shell = Shell::default();
    Builder.make_shell(&mut shell);
    Builder.add(&mut shell, &face).unwrap();
    Builder.add(&mut shell, &moved).unwrap();
    let graph = imported_v2(&shell, false);
    assert_eq!(graph.topo().faces().nb(), 2);
    assert_eq!(graph.topo().edges().nb(), 2);
    assert_eq!(graph.topo().co_edges().nb(), 2);
    assert_ne!(
        graph.shapes().find_node(&face),
        graph.shapes().find_node(&moved)
    );
    assert_pcurves(&graph);
}
fn assert_pcurves(graph: &BRepGraph) {
    for index in 0..graph.topo().co_edges().nb() {
        assert!(co_edge::has_pcurve(graph, CoEdgeId::new(index)));
        assert_eq!(co_edge::range(graph, CoEdgeId::new(index)), (0., 1.));
    }
}
#[test]
fn cumulative_v2_preserves_compound_definition_sharing_and_placed_uses() {
    let face = single_use_face();
    let moved = translated(&face);
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder.add(&mut compound, &face).unwrap();
    Builder.add(&mut compound, &moved).unwrap();
    let graph = imported_v2(&compound, false);
    assert_eq!(graph.topo().faces().nb(), 1);
    let refs = &graph
        .topo()
        .compounds()
        .relations(CompoundId::new(0))
        .child_ref_ids;
    assert_eq!(refs.len(), 2);
    let [first, second] = [refs[0], refs[1]].map(|id| graph.refs().children().entry(id));
    assert_eq!(first.child_node_id, second.child_node_id);
    assert_eq!(&first.local_location, face.location());
    assert_eq!(&second.local_location, moved.location());
    assert_pcurves(&graph);
}
