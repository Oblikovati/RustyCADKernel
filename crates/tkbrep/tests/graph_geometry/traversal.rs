#[test]
fn BRepGraph_GeometryTest__SolidDef_CountMatchesNb() {
    let graph = geometry_box(true);
    let count = DefIterator::<SolidTag>::new(&graph).count();
    assert_eq!(count, graph.topo().solids().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__ShellDef_CountMatchesNb() {
    let graph = geometry_box(true);
    let count = DefIterator::<ShellTag>::new(&graph).count();
    assert_eq!(count, graph.topo().shells().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__FaceDef_CountMatchesNb() {
    let graph = geometry_box(true);
    let count = DefIterator::<FaceTag>::new(&graph).count();
    assert_eq!(count, graph.topo().faces().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__WireDef_CountMatchesNb() {
    let graph = geometry_box(true);
    let count = DefIterator::<WireTag>::new(&graph).count();
    assert_eq!(count, graph.topo().wires().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__EdgeDef_CountMatchesNb() {
    let graph = geometry_box(true);
    let count = DefIterator::<EdgeTag>::new(&graph).count();
    assert_eq!(count, graph.topo().edges().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__VertexDef_CountMatchesNb() {
    let graph = geometry_box(true);
    let count = DefIterator::<VertexTag>::new(&graph).count();
    assert_eq!(count, graph.topo().vertices().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__FaceDef_CountViaIterator_MatchesNb() {
    let graph = geometry_box(true);
    let count = DefIterator::<FaceTag>::new(&graph).count();
    assert_eq!(count, graph.topo().faces().nb() as usize);
}
#[test]
fn BRepGraph_GeometryTest__FaceDef_AllSurfacesNonNull() {
    let graph = geometry_box(true);
    let mut count = 0;
    for id in geometry_ids::<FaceTag>(&graph) {
        assert!(face::has_surface(&graph, id));
        count += 1;
    }
    assert_eq!(count, graph.topo().faces().nb());
}
#[test]
fn BRepGraph_GeometryTest__EdgeDef_AllCurves3dNonNull() {
    let graph = geometry_box(true);
    let mut count = 0;
    for id in geometry_ids::<EdgeTag>(&graph) {
        if !edge::degenerated(&graph, id) {
            assert!(edge::has_curve(&graph, id));
        }
        count += 1;
    }
    assert_eq!(count, graph.topo().edges().nb());
}
#[test]
fn BRepGraph_GeometryTest__ConnectedComponents_SingleBox_OneComponent() {
    let graph = geometry_box(true);
    let counts = geometry_component_counts(&graph);
    assert_eq!(counts.len(), 1);
    assert_eq!(*counts.values().next().unwrap(), 6);
}
#[test]
fn BRepGraph_GeometryTest__ConnectedComponents_TwoBoxCompound_TwoComponents() {
    let graph = geometry_two_components();
    let counts = geometry_component_counts(&graph);
    assert_eq!(counts.len(), 2);
}
#[test]
fn BRepGraph_GeometryTest__ConnectedComponents_TwoBoxCompound_FacesGroupedPerRoot() {
    let graph = geometry_two_components();
    let counts = geometry_component_counts(&graph);
    assert_eq!(counts.len(), 2);
    for count in counts.values() {
        assert_eq!(*count, 6);
    }
}
#[test]
fn BRepGraph_GeometryTest__ConnectedComponents_TwoBoxCompound_CoverAllFaces() {
    let graph = geometry_two_components();
    let counts = geometry_component_counts(&graph);
    assert_eq!(counts.values().sum::<u32>(), graph.topo().faces().nb());
}
