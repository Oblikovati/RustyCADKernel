#[test]
fn BRepGraph_ShapesViewImportTest__AddVertex_ReturnsValidId() {
    let mut graph = BRepGraph::new();
    let vertex = graph
        .editor()
        .vertices()
        .add(Point3::new(1., 2., 3.), 0.001);
    assert!(vertex.is_valid());
    assert_eq!(vertex.index, 0);
    let definition = graph.topo().vertices().definition(VertexId::start());
    assert!((definition.point.x() - 1.).abs() <= precision::confusion());
    assert!((definition.point.y() - 2.).abs() <= precision::confusion());
    assert!((definition.point.z() - 3.).abs() <= precision::confusion());
    assert!((definition.tolerance - 0.001).abs() <= 1e-10);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddEdge_WithCurve() {
    let mut graph = BRepGraph::new();
    let (first, last, edge) = import_single_edge(&mut graph);
    assert!(edge.is_valid());
    let definition = graph.topo().edges().definition(EdgeId::start());
    let start = tool::edge::start_vertex_id(&graph, EdgeId::start());
    let end = tool::edge::end_vertex_id(&graph, EdgeId::start());
    assert_eq!(graph.refs().vertices().entry(start).child_vertex_id, first);
    assert_eq!(graph.refs().vertices().entry(end).child_vertex_id, last);
    assert!(definition.curve3d_rep_id.is_valid());
    let range = tool::edge::range(&graph, EdgeId::start());
    assert!((range.0 - 0.).abs() <= 1e-10);
    assert!((range.1 - 10.).abs() <= 1e-10);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddWire_ClosedRectangle() {
    let mut graph = BRepGraph::new();
    let (_, edges) = import_rectangle_edges(&mut graph);
    let wire = import_rectangle_wire(&mut graph, &edges);
    assert!(wire.is_valid());
    assert_eq!(count_co_edges_of_wire(&graph, WireId::start()), 4);
    assert!(tool::wire::is_closed(&graph, WireId::start()));
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddFace_WithSurface() {
    let mut graph = BRepGraph::new();
    // Build a simple rectangular face programmatically.
    let (face, _, _, _) = import_rectangle_face(&mut graph);
    assert!(face.is_valid());
    assert_eq!(graph.topo().faces().nb(), 1);
    assert_eq!(graph.topo().faces().nb(), 1);
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
fn BRepGraph_ShapesViewImportTest__AddWireToFace_AppendsInnerWireRefAndRelations() {
    let mut graph = BRepGraph::new();
    let (_, edges) = import_rectangle_edges(&mut graph);
    let outer = import_rectangle_wire(&mut graph, &edges);
    let inner = import_rectangle_wire(&mut graph, &edges);
    let face = graph
        .editor()
        .faces()
        .add(&import_plane(), outer, &[], 0.001);
    let reference = graph.editor().faces().append(face, inner, FORWARD).unwrap();
    assert!(reference.is_valid());
    let relations = graph.topo().faces().relations(face);
    assert_eq!(relations.wire_ref_ids.len(), 2);
    assert_eq!(relations.wire_ref_ids[1], reference);
    assert_eq!(graph.refs().wires().entry(reference).child_wire_id, inner);
    let parents = &graph.topo().wires().relations(inner).parent_wire_ref_ids;
    assert_eq!(parents.len(), 1);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddEdge_InvalidVertex_ReturnsInvalidAndDoesNotAppend() {
    let mut graph = BRepGraph::new();
    let vertex = graph.editor().vertices().add(Point3::default(), 0.001);
    let edge =
        graph
            .editor()
            .edges()
            .add(vertex, VertexId::new(42), &Handle::default(), 0., 1., 0.001);
    assert!(!edge.is_valid());
    assert_eq!(graph.topo().edges().nb(), 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddWire_InvalidEdge_ReturnsInvalidAndDoesNotAppend() {
    let mut graph = BRepGraph::new();
    let coedge = graph.editor().co_edges().add(EdgeId::new(17), FORWARD);
    let wire = graph.editor().wires().add(&[coedge]);
    assert!(!wire.is_valid());
    assert_eq!(graph.topo().wires().nb(), 0);
    assert_eq!(graph.topo().co_edges().nb(), 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddFace_InvalidOuterWire_ReturnsInvalidAndDoesNotAppend() {
    let mut graph = BRepGraph::new();
    let face = graph
        .editor()
        .faces()
        .add(&import_plane(), WireId::new(9), &[], 0.001);
    assert!(!face.is_valid());
    assert_eq!(graph.topo().faces().nb(), 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddShellAndSolid() {
    let mut graph = BRepGraph::new();
    let shell = graph.editor().shells().add();
    assert!(shell.is_valid());
    let solid = graph.editor().solids().add();
    assert!(solid.is_valid());
}

#[test]
fn BRepGraph_ShapesViewImportTest__AppendTwoBoxFaces() {
    let (mut first_copy, mut second_copy) = import_copied_box_faces();
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(first_copy.shape().unwrap()).unwrap();
    assert_eq!(graph.topo().faces().nb(), 1);
    let options = Options {
        create_auto_product: false,
        flatten: true,
        track_added_nodes: false,
        ..Options::default()
    };
    graph
        .shapes_mut()
        .add_with_options(second_copy.shape().unwrap(), &options)
        .unwrap();
    assert_eq!(graph.topo().faces().nb(), 2);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddFace_CreatesUsage() {
    let mut graph = BRepGraph::new();
    let (face, _, _, _) = import_rectangle_face(&mut graph);
    let shell = graph.editor().shells().add();
    let reference = graph
        .editor()
        .shells()
        .append(shell, face, FORWARD)
        .unwrap();
    assert!(reference.is_valid());
    assert_eq!(count_face_refs_of_shell(&graph, ShellId::start()), 1);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddFace_InvalidNodes_NoMutation() {
    let mut graph = BRepGraph::new();
    let shell = graph.editor().shells().add();
    let reference = graph
        .editor()
        .shells()
        .append(shell, FaceId::new(4), FORWARD)
        .unwrap();
    assert!(!reference.is_valid());
    assert_eq!(count_face_refs_of_shell(&graph, shell), 0);
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddShell_CreatesUsage() {
    let mut graph = BRepGraph::new();
    let shell = graph.editor().shells().add();
    let solid = graph.editor().solids().add();
    let reference = graph
        .editor()
        .solids()
        .append(solid, shell, FORWARD)
        .unwrap();
    assert!(reference.is_valid());
    assert_eq!(count_shell_refs_of_solid(&graph, SolidId::start()), 1);
}

#[test]
fn BRepGraph_ShapesViewImportTest__MutInvalidTopologyDefs_ThrowProgramError() {
    let mut graph = BRepGraph::new();
    import_program_error(graph.editor().vertices().mutate(VertexId::new(7)));
    import_program_error(graph.editor().edges().mutate(EdgeId::new(7)));
    import_program_error(graph.editor().wires().mutate(WireId::new(7)));
    import_program_error(graph.editor().faces().mutate(FaceId::new(7)));
    import_program_error(graph.editor().shells().mutate(ShellId::new(7)));
    import_program_error(graph.editor().solids().mutate(SolidId::new(7)));
    import_program_error(graph.editor().co_edges().mutate(CoEdgeId::new(7)));
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddCompound_WithChildren() {
    let mut graph = BRepGraph::new();
    let first = graph.editor().solids().add();
    let second = graph.editor().solids().add();
    let compound = graph
        .editor()
        .compounds()
        .add(&[first.into(), second.into()]);
    assert!(compound.is_valid());
    assert_eq!(graph.topo().compounds().nb(), 1);
    assert_eq!(
        count_child_refs_of_parent(&graph, CompoundId::start().into()),
        2
    );
}

#[test]
fn BRepGraph_ShapesViewImportTest__AddCompSolid_WithSolids() {
    let mut graph = BRepGraph::new();
    let first = graph.editor().solids().add();
    let second = graph.editor().solids().add();
    let composite = graph.editor().comp_solids().add(&[first, second]);
    assert!(composite.is_valid());
    assert_eq!(graph.topo().comp_solids().nb(), 1);
    let _definition = graph.topo().comp_solids().definition(CompSolidId::start());
    assert_eq!(
        count_solid_refs_of_comp_solid(&graph, CompSolidId::start()),
        2
    );
}

#[test]
fn BRepGraph_ShapesViewImportTest__FullSolid_ProgrammaticConstruction() {
    let mut graph = BRepGraph::new();
    let (face, _, _, _) = import_rectangle_face(&mut graph);
    let shell = graph.editor().shells().add();
    graph
        .editor()
        .shells()
        .append(shell, face, FORWARD)
        .unwrap();
    let solid = graph.editor().solids().add();
    graph
        .editor()
        .solids()
        .append(solid, shell, FORWARD)
        .unwrap();
    assert_eq!(graph.topo().solids().nb(), 1);
    assert_eq!(graph.topo().shells().nb(), 1);
    assert_eq!(graph.topo().faces().nb(), 1);
    assert_eq!(count_shell_refs_of_solid(&graph, SolidId::start()), 1);
}

fn import_single_edge(graph: &mut BRepGraph) -> (VertexId, VertexId, EdgeId) {
    let first = graph
        .editor()
        .vertices()
        .add(Point3::new(0., 0., 0.), 0.001);
    let last = graph
        .editor()
        .vertices()
        .add(Point3::new(10., 0., 0.), 0.001);
    let line = import_line(Point3::default(), Direction3::X);
    let edge = graph
        .editor()
        .edges()
        .add(first, last, &line, 0., 10., 0.001);
    (first, last, edge)
}

fn import_copied_box_faces() -> (BRepBuilderAPICopy, BRepBuilderAPICopy) {
    let box_shape = support::make_box(10., 20., 30.);
    let mut explorer =
        rustycad_tkbrep::topexp::Explorer::new(&box_shape, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap();
    assert!(explorer.more());
    let first_copy = BRepBuilderAPICopy::with_shape(explorer.current(), true, false).unwrap();
    explorer.next().unwrap();
    assert!(explorer.more());
    let second_copy = BRepBuilderAPICopy::with_shape(explorer.current(), true, false).unwrap();
    (first_copy, second_copy)
}
