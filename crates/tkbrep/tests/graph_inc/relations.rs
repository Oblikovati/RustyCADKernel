fn inc_atomic_compound(shape: &Shape) -> BRepGraph {
    let graph = inc_populate(&inc_compound(std::slice::from_ref(shape)), false);
    assert_eq!(graph.topo().compounds().nb(), 1);
    graph
}
#[test]
fn Relations_Validate_CompoundWithAtomicWire() {
    let edge = support::make_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0));
    let graph = inc_atomic_compound(&inc_wire(&[edge.into_shape()]));
    assert!(graph.topo().wires().nb() >= 1);
    inc_lightweight(&graph);
}
#[test]
fn Relations_Validate_CompoundWithAtomicEdge() {
    let graph = inc_atomic_compound(&support::make_edge(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(5.0, 0.0, 0.0),
    ));
    assert!(graph.topo().edges().nb() >= 1);
    inc_lightweight(&graph);
}
#[test]
fn Relations_Validate_CompoundWithAtomicVertex() {
    let graph = inc_atomic_compound(&support::make_vertex(Point3::new(1.0, 2.0, 3.0)));
    assert!(graph.topo().vertices().nb() >= 1);
    inc_lightweight(&graph);
}
#[test]
fn Relations_CoEdgeToWire_IsPopulated() {
    let graph = inc_populate(&support::make_box(5.0, 5.0, 5.0), false);
    assert!(graph.topo().co_edges().nb() > 0);
    for i in 0..graph.topo().co_edges().nb() {
        assert!(
            graph
                .topo()
                .co_edges()
                .definition(CoEdgeId::new(i))
                .parent_wire_id
                .is_valid()
        );
    }
    inc_lightweight(&graph);
}
fn inc_compsolid(children: &[Shape]) -> Shape {
    let mut shape = CompSolid::default();
    BRepBuilder::default().make_comp_solid(&mut shape);
    for child in children {
        BRepBuilder::default().add(&mut shape, child).unwrap();
    }
    shape.into_shape()
}
#[test]
fn Relations_CompSolid_IncomingMaintained_AfterBuild() {
    let shape = inc_compsolid(&[
        support::make_box(4.0, 4.0, 4.0).into_shape(),
        support::make_box(2.0, 2.0, 2.0).into_shape(),
    ]);
    let graph = inc_populate(&shape, false);
    assert_eq!(graph.topo().comp_solids().nb(), 1);
    assert!(graph.topo().solids().nb() >= 2);
    for i in 0..graph.topo().solids().nb() {
        let refs = &graph
            .topo()
            .solids()
            .relations(SolidId::new(i))
            .parent_solid_ref_ids;
        assert_eq!(refs.len(), 1);
    }
    inc_lightweight(&graph);
}
#[test]
fn Relations_CompSolid_IncomingMaintained_AfterBuildDelta() {
    let graph = inc_populate(
        &inc_compsolid(&[support::make_box(3.0, 3.0, 3.0).into_shape()]),
        false,
    );
    assert_eq!(graph.topo().comp_solids().nb(), 1);
    assert!(graph.topo().solids().nb() >= 1);
    inc_lightweight(&graph);
    assert!(
        !graph
            .topo()
            .solids()
            .relations(SolidId::start())
            .parent_solid_ref_ids
            .is_empty()
    );
}
#[test]
fn Relations_Validate_Box_FullConsistency() {
    inc_lightweight(&inc_populate(&inc_box(), false));
}
#[test]
fn Relations_Validate_Compound_FullConsistency() {
    let shape = inc_compound(&[
        support::make_box(5.0, 5.0, 5.0).into_shape(),
        support::make_box(3.0, 3.0, 3.0).into_shape(),
    ]);
    inc_lightweight(&inc_populate(&shape, false));
}
#[test]
fn Relations_BulkBuild_TwiceProducesEqualState() {
    let shape = support::make_box(7.0, 11.0, 13.0);
    let a = inc_populate(&shape, false);
    let b = inc_populate(&shape, false);
    assert_eq!(a.topo().edges().nb(), b.topo().edges().nb());
    for i in 0..a.topo().edges().nb() {
        let id = EdgeId::new(i);
        assert_eq!(
            a.topo().edges().wires_of(id).collect::<Vec<_>>(),
            b.topo().edges().wires_of(id).collect::<Vec<_>>()
        );
        assert_eq!(
            a.topo().edges().faces_of(id).collect::<Vec<_>>(),
            b.topo().edges().faces_of(id).collect::<Vec<_>>()
        );
    }
}
#[test]
fn Relations_SeamEdge_NoDuplicateEdgeToWireEntries() {
    let graph = inc_populate(&inc_sphere(), false);
    for i in 0..graph.topo().edges().nb() {
        let mut seen = Vec::new();
        for id in graph.topo().edges().wires_of(EdgeId::new(i)) {
            assert!(!seen.contains(&id));
            seen.push(id);
        }
    }
    inc_lightweight(&graph);
}
#[test]
fn Relations_FaceSupplementVertex_NoPersistedRefUpdate() {
    let mut graph = inc_add(&inc_box());
    graph
        .editor()
        .vertices()
        .add(Point3::new(7.0, 7.0, 7.0), 1e-7);
    assert!(graph.validate_relations());
    let layer = graph
        .layer_registry()
        .ensure::<LayerTopoSupplement>()
        .unwrap();
    assert!(!layer.is_null());
    let attached = layer.get().unwrap().attached_to(FaceId::start().into());
    assert_eq!(attached.len(), 0);
}
