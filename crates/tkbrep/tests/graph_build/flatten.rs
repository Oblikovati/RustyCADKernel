#[test]
fn BRepGraph_BuildTest__AddFlatten_OnEmptyGraph_BuildsFlattenedGraph() {
    let shape = build_box(10.0, 20.0, 30.0);
    let mut graph = BRepGraph::new();
    flat_add(&mut graph, &shape);
    assert_eq!(graph.topo().solids().nb(), 0);
    assert_eq!(graph.topo().shells().nb(), 0);
    assert_eq!(graph.topo().faces().nb(), 6);
    for i in 0..graph.topo().faces().nb() {
        assert!(!graph.topo().generic().is_removed(FaceId::new(i).into()));
    }
    assert!(graph.editor().validate_mutation_boundary(None));
}
#[test]
fn BRepGraph_BuildTest__Build_MutationBoundary_IsValid() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    assert!(graph.editor().validate_mutation_boundary(None));
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_SameFaceTwice_DedupsDefinition() {
    let face = first_build_face(&build_box(10.0, 20.0, 30.0));
    let mut graph = BRepGraph::new();
    flat_add(&mut graph, &face);
    flat_add(&mut graph, &face);
    assert_eq!(graph.topo().faces().nb(), 1);
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_AfterBuild_DoesNotCreateNewSolidDefs() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    let solids = graph.topo().solids().nb();
    let faces = graph.topo().faces().nb();
    flat_add(&mut graph, &build_box(15.0, 25.0, 35.0));
    assert_eq!(graph.topo().solids().nb(), solids);
    assert_eq!(graph.topo().faces().nb(), faces + 6);
    assert_eq!(graph.root_product_ids().len(), 1);
    assert!(graph.editor().validate_mutation_boundary(None));
}
#[test]
fn BRepGraph_BuildTest__AddFull_MutationBoundary_IsValid() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    graph.shapes_mut().add(&build_sphere(5.0)).unwrap();
    assert!(graph.editor().validate_mutation_boundary(None));
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_AppendedFaceHasNoParentShell() {
    let face = first_build_face(&build_box(10.0, 20.0, 30.0));
    let mut graph = BRepGraph::new();
    flat_add(&mut graph, &face);
    assert_eq!(graph.topo().faces().nb(), 1);
    assert_eq!(graph.topo().shells().nb(), 0);
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_PreservesExistingUIDs() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    assert!(graph.topo().edges().nb() > 0);
    let uid = graph.uids().of_node(EdgeId::start().into());
    assert!(uid.is_valid());
    flat_add(&mut graph, &build_sphere(5.0));
    assert_eq!(uid, graph.uids().of_node(EdgeId::start().into()));
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_StandaloneVertex_AppendsIntoNonEmptyGraph() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    let before = graph.topo().vertices().nb();
    let shape = build_standalone_vertex(100.0);
    flat_add(&mut graph, &shape);
    assert_eq!(graph.topo().vertices().nb(), before + 1);
    assert!(graph.shapes().find_node(&shape).is_valid());
    assert!(
        !graph
            .shapes()
            .shape(VertexId::new(before).into())
            .unwrap()
            .is_null()
    );
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_StandaloneEdge_AppendsIntoNonEmptyGraph() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    let before = graph.topo().edges().nb();
    let shape = build_standalone_edge(100.0, 120.0);
    flat_add(&mut graph, &shape);
    assert_eq!(graph.topo().edges().nb(), before + 1);
    assert!(graph.shapes().find_node(&shape).is_valid());
    assert!(
        !graph
            .shapes()
            .shape(EdgeId::new(before).into())
            .unwrap()
            .is_null()
    );
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_StandaloneWire_AppendsIntoNonEmptyGraph() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    let before = graph.topo().wires().nb();
    let shape = build_standalone_wire();
    flat_add(&mut graph, &shape);
    assert_eq!(graph.topo().wires().nb(), before + 1);
    assert!(graph.shapes().find_node(&shape).is_valid());
    assert!(
        !graph
            .shapes()
            .shape(WireId::new(before).into())
            .unwrap()
            .is_null()
    );
}
#[test]
fn BRepGraph_BuildTest__AddFlatten_CompoundWithStandaloneShapes() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    let shape = build_compound(&[
        build_standalone_vertex(50.0),
        build_standalone_edge(0.0, 10.0),
    ]);
    let vertices = graph.topo().vertices().nb();
    let edges = graph.topo().edges().nb();
    flat_add(&mut graph, &shape);
    assert!(graph.topo().vertices().nb() > vertices);
    assert!(graph.topo().edges().nb() > edges);
}
#[test]
fn BRepGraph_BuildTest__Build_BasicQueriesAndAlgorithmsWork() {
    let shape = build_box(10.0, 20.0, 30.0);
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph
        .shapes_mut()
        .add_with_options(
            &shape,
            &Options {
                parallel: false,
                ..Options::default()
            },
        )
        .unwrap();
    assert_eq!(graph.topo().faces().nb(), 6);
    assert_eq!(graph.topo().edges().nb(), 12);
    assert_eq!(graph.topo().vertices().nb(), 8);
}
#[test]
fn BRepGraph_BuildTest__RootProductIds_Box_ReturnsOneProduct() {
    let graph = build_graph(&build_box(10.0, 20.0, 30.0));
    let roots = graph.root_product_ids();
    assert_eq!(roots.len(), 1);
    assert!(roots[0].is_valid());
    let shape_root = graph.topo().products().shape_root(roots[0]);
    assert!(shape_root.is_valid());
    assert_eq!(shape_root.kind, NodeKind::SOLID);
}
#[test]
fn BRepGraph_BuildTest__BuildOptions_DisableAutoProduct_DoesNotCreateProducts() {
    let shape = build_box(10.0, 20.0, 30.0);
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let options = Options {
        create_auto_product: false,
        parallel: false,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(&shape, &options)
        .unwrap();
    assert_eq!(graph.topo().products().nb(), 0);
    assert!(graph.root_product_ids().is_empty());
    assert_eq!(graph.topo().solids().nb(), 1);
}
#[test]
fn BRepGraph_BuildTest__RootProductIds_AddFlatten_ProductCountUnchanged() {
    let mut graph = build_graph(&build_box(10.0, 20.0, 30.0));
    assert_eq!(graph.root_product_ids().len(), 1);
    let faces = graph.topo().faces().nb();
    let face = first_build_face(&build_box(5.0, 5.0, 5.0));
    flat_add(&mut graph, &face);
    let roots = graph.root_product_ids();
    assert_eq!(roots.len(), 1);
    assert_eq!(graph.topo().faces().nb(), faces + 1);
    assert!(roots[0].is_valid());
}
