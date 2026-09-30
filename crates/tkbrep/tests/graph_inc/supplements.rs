fn inc_wire(children: &[Shape]) -> Wire {
    let mut wire = Wire::default();
    BRepBuilder::default().make_wire(&mut wire);
    for child in children {
        BRepBuilder::default().add(&mut wire, child).unwrap();
    }
    wire
}
fn inc_internal_edge() -> Edge {
    let mut edge = support::make_edge(Point3::new(0.0, 0.0, 0.0), Point3::new(10.0, 0.0, 0.0));
    let vertex = support::make_vertex(Point3::new(5.0, 0.0, 0.0));
    BRepBuilder::default()
        .add(&mut edge, &vertex.oriented(Orientation::Internal))
        .unwrap();
    edge
}
fn inc_plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::from_pln(Plane3::default())).into_cast()
}
fn inc_wrapped_edge() -> Shape {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &inc_plane(), precision::confusion())
        .unwrap();
    BRepBuilder::default()
        .add(&mut face, &inc_wire(&[inc_internal_edge().into_shape()]))
        .unwrap();
    face.into_shape()
}
fn inc_plain_face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 10.0, 0.0, 10.0))
        .unwrap()
        .face()
        .unwrap()
}
fn inc_shell(children: &[Shape]) -> Shell {
    let mut shell = Shell::default();
    BRepBuilder::default().make_shell(&mut shell);
    for child in children {
        BRepBuilder::default().add(&mut shell, child).unwrap();
    }
    shell
}
fn inc_solid(children: &[Shape]) -> Solid {
    let mut solid = Solid::default();
    BRepBuilder::default().make_solid(&mut solid);
    for child in children {
        BRepBuilder::default().add(&mut solid, child).unwrap();
    }
    solid
}
fn inc_plain_shell() -> Shell {
    inc_shell(&[inc_plain_face().into_shape()])
}
fn inc_plain_solid() -> Solid {
    inc_solid(&[inc_plain_shell().into_shape()])
}
fn inc_direct_count(shape: &Shape, kind: ShapeEnum, orientation: Orientation) -> usize {
    topods::ShapeIterator::new(shape, false, false)
        .filter(|child| child.shape_type().unwrap() == kind && child.orientation() == orientation)
        .count()
}
fn inc_supplement_graph(shape: &Shape) -> (BRepGraph, Handle<LayerTopoSupplement>) {
    let mut graph = BRepGraph::new();
    let layer = graph
        .layer_registry()
        .ensure::<LayerTopoSupplement>()
        .unwrap();
    populate::perform(&mut graph, shape, false, &populate::Options).unwrap();
    assert!(!graph.is_empty());
    (graph, layer)
}
fn inc_attachment(
    layer: &Handle<LayerTopoSupplement>,
    owner: NodeId,
    kind: AttachmentKind,
    shape_kind: ShapeEnum,
) {
    let layer = layer.get().unwrap();
    let attached = layer.attached_to(owner);
    assert_eq!(attached.len(), 1);
    let entry = layer.find_by_uid(attached[0]).unwrap();
    assert_eq!(entry.kind, kind);
    assert_eq!(entry.shape.shape_type().unwrap(), shape_kind);
    assert_eq!(entry.shape.orientation(), Orientation::Internal);
}
#[test]
fn Populate_RegistersSupplementEdgeVerticesIntoLayer() {
    let (graph, layer) = inc_supplement_graph(&inc_wrapped_edge());
    assert!(graph.topo().edges().nb() > 0);
    inc_attachment(
        &layer,
        EdgeId::start().into(),
        AttachmentKind::EdgeInternalVertex,
        ShapeEnum::Vertex,
    );
}
#[test]
fn Reconstruct_ReplaysSupplementEdgeVerticesFromLayer() {
    let (graph, _layer) = inc_supplement_graph(&inc_wrapped_edge());
    let recon = inc_reconstruct(&graph, FaceId::start());
    assert!(
        inc_subshapes(&recon, ShapeEnum::Edge)
            .iter()
            .any(|edge| inc_direct_count(edge, ShapeEnum::Vertex, Orientation::Internal) > 0)
    );
}
#[test]
fn Reconstruct_WithoutSupplementLayer_DropsSupplementEdgeVerticesButKeepsCore() {
    let graph = inc_populate(&inc_wrapped_edge(), false);
    let recon = inc_reconstruct(&graph, FaceId::start());
    let edges = inc_subshapes(&recon, ShapeEnum::Edge);
    assert_eq!(edges.len(), 1);
    let boundaries: usize = edges
        .iter()
        .map(|edge| {
            inc_direct_count(edge, ShapeEnum::Vertex, Orientation::Forward)
                + inc_direct_count(edge, ShapeEnum::Vertex, Orientation::Reversed)
        })
        .sum();
    let internal: usize = edges
        .iter()
        .map(|edge| inc_direct_count(edge, ShapeEnum::Vertex, Orientation::Internal))
        .sum();
    assert_eq!(boundaries, 2);
    assert_eq!(internal, 0);
}
#[test]
fn Reconstruct_EdgeNode_ReplaysSupplementEdgeVerticesFromLayer() {
    let (graph, _layer) = inc_supplement_graph(&inc_internal_edge());
    assert!(graph.topo().edges().nb() > 0);
    let recon = inc_reconstruct(&graph, EdgeId::start());
    let internal = inc_direct_count(&recon, ShapeEnum::Vertex, Orientation::Internal);
    assert_eq!(internal, 1);
}
#[test]
fn Reconstruct_WireNode_ReplaysSupplementEdgeVerticesFromLayer() {
    let (graph, _layer) = inc_supplement_graph(&inc_wire(&[inc_internal_edge().into_shape()]));
    assert!(graph.topo().wires().nb() > 0);
    let recon = inc_reconstruct(&graph, WireId::start());
    let count: usize = inc_subshapes(&recon, ShapeEnum::Edge)
        .iter()
        .map(|edge| inc_direct_count(edge, ShapeEnum::Vertex, Orientation::Internal))
        .sum();
    assert_eq!(count, 1);
}
fn inc_bare_edge(first: &topods::Vertex, last: &topods::Vertex) -> Shape {
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    BRepBuilder::default()
        .add(&mut edge, &first.oriented(Orientation::Forward))
        .unwrap();
    BRepBuilder::default()
        .add(&mut edge, &last.oriented(Orientation::Reversed))
        .unwrap();
    edge.into_shape()
}
fn inc_direct_vertex_face() -> Face {
    let vertices = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]
        .map(|(x, y)| support::make_vertex(Point3::new(x, y, 0.0)));
    let edges: Vec<Shape> = (0..4)
        .map(|i| inc_bare_edge(&vertices[i], &vertices[(i + 1) % 4]))
        .collect();
    let wire = inc_wire(&edges);
    wire.set_closed(true).unwrap();
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    BRepBuilder::default().add(&mut face, &wire).unwrap();
    let vertex = support::make_vertex(Point3::new(5.0, 5.0, 0.0));
    BRepBuilder::default()
        .add(&mut face, &vertex.oriented(Orientation::Internal))
        .unwrap();
    face
}
#[test]
fn Reconstruct_FaceNode_ReplaysSupplementFaceVerticesFromLayer() {
    let (graph, layer) = inc_supplement_graph(&inc_direct_vertex_face());
    assert!(graph.topo().faces().nb() > 0);
    inc_attachment(
        &layer,
        FaceId::start().into(),
        AttachmentKind::FaceDirectVertex,
        ShapeEnum::Vertex,
    );
    let recon = inc_reconstruct(&graph, FaceId::start());
    assert_eq!(
        topods::ShapeIterator::new(&recon, false, false)
            .filter(|s| s.shape_type().unwrap() == ShapeEnum::Wire)
            .count(),
        1
    );
    let internal = inc_direct_count(&recon, ShapeEnum::Vertex, Orientation::Internal);
    assert_eq!(internal, 1);
}
#[test]
fn Populate_ShellRoutesInternalFaceToSupplement() {
    let shape = inc_shell(&[
        inc_plain_face().oriented(Orientation::Forward),
        inc_plain_face().oriented(Orientation::Internal),
    ]);
    let (graph, layer) = inc_supplement_graph(&shape);
    let id = ShellId::start();
    assert_eq!(graph.topo().shells().nb(), 1);
    assert_eq!(graph.topo().shells().relations(id).face_ref_ids.len(), 1);
    inc_attachment(
        &layer,
        id.into(),
        AttachmentKind::ShellAuxShape,
        ShapeEnum::Face,
    );
    inc_replay_children(&graph, id.into(), ShapeEnum::Face);
}
#[test]
fn Populate_SolidRoutesInternalShellToSupplement() {
    let shape = inc_solid(&[
        inc_plain_shell().oriented(Orientation::Forward),
        inc_plain_shell().oriented(Orientation::Internal),
    ]);
    let (graph, layer) = inc_supplement_graph(&shape);
    let id = SolidId::start();
    assert_eq!(graph.topo().solids().nb(), 1);
    assert_eq!(graph.topo().solids().relations(id).shell_ref_ids.len(), 1);
    inc_attachment(
        &layer,
        id.into(),
        AttachmentKind::SolidAuxShape,
        ShapeEnum::Shell,
    );
    inc_replay_children(&graph, id.into(), ShapeEnum::Shell);
}
#[test]
fn Populate_CompSolidRoutesInternalSolidToSupplement() {
    let shape = inc_compsolid(&[
        inc_plain_solid().oriented(Orientation::Forward),
        inc_plain_solid().oriented(Orientation::Internal),
    ]);
    let (graph, layer) = inc_supplement_graph(&shape);
    let id = CompSolidId::start();
    assert_eq!(graph.topo().comp_solids().nb(), 1);
    assert_eq!(
        graph.topo().comp_solids().relations(id).solid_ref_ids.len(),
        1
    );
    inc_attachment(
        &layer,
        id.into(),
        AttachmentKind::CompSolidAuxShape,
        ShapeEnum::Solid,
    );
    inc_replay_children(&graph, id.into(), ShapeEnum::Solid);
}
#[test]
fn Populate_SolidInvalidOrderChildRoutesToSupplement() {
    let shape = inc_solid(&[inc_internal_edge().oriented(Orientation::Forward)]);
    let (graph, layer) = inc_supplement_graph(&shape);
    let id = SolidId::start();
    assert_eq!(graph.topo().solids().nb(), 1);
    assert_eq!(graph.topo().solids().relations(id).shell_ref_ids.len(), 0);
    let layer = layer.get().unwrap();
    let attached = layer.attached_to(id.into());
    assert_eq!(attached.len(), 1);
    let entry = layer.find_by_uid(attached[0]).unwrap();
    assert_eq!(entry.kind, AttachmentKind::SolidAuxShape);
    assert_eq!(entry.shape.shape_type().unwrap(), ShapeEnum::Edge);
    let recon = inc_reconstruct(&graph, id);
    let count = inc_direct_count(&recon, ShapeEnum::Edge, Orientation::Forward);
    assert_eq!(count, 1);
}

fn inc_replay_children(graph: &BRepGraph, id: NodeId, kind: ShapeEnum) {
    let recon = inc_reconstruct(graph, id);
    assert_eq!(inc_direct_count(&recon, kind, Orientation::Forward), 1);
    assert_eq!(inc_direct_count(&recon, kind, Orientation::Internal), 1);
}
