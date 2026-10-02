fn triangulation() -> Handle<Triangulation> {
    Handle::new(Triangulation::new())
}

fn mesh_handles() -> (
    Handle<Triangulation>,
    Handle<Polygon3D>,
    Handle<Polygon2D>,
    Handle<PolygonOnTriangulation>,
) {
    (
        triangulation_with_test_data(),
        polygon3d_with_test_data(),
        polygon2d_with_test_data(),
        polygon_on_tri_with_test_data(),
    )
}

fn triangulation_with_test_data() -> Handle<Triangulation> {
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
    let points = [
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(4.0, 5.0, 6.0),
        Point3::new(7.0, 8.0, 9.0),
    ];
    for (index, point) in (1..).zip(points) {
        mesh.set_node(index, point).unwrap();
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    mesh
}

fn polygon3d_with_test_data() -> Handle<Polygon3D> {
    let mesh = Handle::new(Polygon3D::new(2, false).unwrap());
    mesh.change_nodes(|nodes| {
        nodes.set_value(1, Point3::new(10.0, 11.0, 12.0)).unwrap();
        nodes.set_value(2, Point3::new(13.0, 14.0, 15.0)).unwrap();
    });
    mesh
}

fn polygon2d_with_test_data() -> Handle<Polygon2D> {
    let mesh = Handle::new(Polygon2D::new(2).unwrap());
    mesh.change_nodes(|nodes| {
        nodes.set_value(1, Point2::new(16.0, 17.0)).unwrap();
        nodes.set_value(2, Point2::new(18.0, 19.0)).unwrap();
    });
    mesh
}

fn polygon_on_tri_with_test_data() -> Handle<PolygonOnTriangulation> {
    let mesh = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    mesh.set_node(1, 2).unwrap();
    mesh.set_node(2, 3).unwrap();
    mesh
}

fn first_face_mesh_ids(graph: &BRepGraph) -> (FaceId, EdgeId, CoEdgeId) {
    let face = FaceId::start();
    let co_edge = first_co_edge_of_face(graph, face);
    let edge = graph.topo().co_edges().definition(co_edge).child_edge_id;
    (face, edge, co_edge)
}

fn first_co_edge_of_face(graph: &BRepGraph, face: FaceId) -> CoEdgeId {
    graph
        .refs()
        .wires()
        .ids_of(face)
        .iter()
        .flat_map(|wire_ref| {
            let wire = graph.refs().wires().entry(*wire_ref).child_wire_id;
            graph
                .topo()
                .wires()
                .relations(wire)
                .co_edge_ids
                .iter()
                .copied()
        })
        .find(|co_edge| graph.topo().co_edges().definition(*co_edge).face_id == face)
        .expect("the box face has a coedge")
}

fn install_mesh_handles(
    graph: &mut BRepGraph,
    ids: (FaceId, EdgeId, CoEdgeId),
    handles: &(
        Handle<Triangulation>,
        Handle<Polygon3D>,
        Handle<Polygon2D>,
        Handle<PolygonOnTriangulation>,
    ),
) {
    let (face, edge, co_edge) = ids;
    install_face_mesh(graph, face, &handles.0);
    install_edge_mesh(graph, edge, &handles.1);
    install_coedge_mesh(graph, co_edge, &handles.2, &handles.3);
}

fn install_face_mesh(graph: &mut BRepGraph, face: FaceId, mesh: &Handle<Triangulation>) {
    graph
        .editor()
        .faces()
        .set_persistent_triangulation(face, mesh)
        .unwrap();
}

fn install_edge_mesh(graph: &mut BRepGraph, edge: EdgeId, mesh: &Handle<Polygon3D>) {
    graph
        .editor()
        .edges()
        .set_persistent_polygon3d(edge, mesh)
        .unwrap();
}

fn install_coedge_mesh(
    graph: &mut BRepGraph,
    co_edge: CoEdgeId,
    polygon2d: &Handle<Polygon2D>,
    polygon_on_tri: &Handle<PolygonOnTriangulation>,
) {
    graph
        .editor()
        .co_edges()
        .set_persistent_polygon2d(co_edge, polygon2d)
        .unwrap();
    graph
        .editor()
        .co_edges()
        .set_persistent_polygon_on_tri(co_edge, polygon_on_tri)
        .unwrap();
}

fn assert_persistent_mesh_handles_copied(
    graph: &BRepGraph,
    ids: (FaceId, EdgeId, CoEdgeId),
    originals: &(
        Handle<Triangulation>,
        Handle<Polygon3D>,
        Handle<Polygon2D>,
        Handle<PolygonOnTriangulation>,
    ),
) {
    let (face, edge, co_edge) = ids;
    let copied = copied_mesh_handles(graph, (face, edge, co_edge));
    assert_mesh_handles_are_detached(&copied, originals);
    assert_mesh_handle_values_match(&copied, originals);
}

type MeshHandles = (
    Handle<Triangulation>,
    Handle<Polygon3D>,
    Handle<Polygon2D>,
    Handle<PolygonOnTriangulation>,
);

fn copied_mesh_handles(graph: &BRepGraph, ids: (FaceId, EdgeId, CoEdgeId)) -> MeshHandles {
    let (face, edge, co_edge) = ids;
    let mesh = graph.mesh().persistent();
    (
        mesh.faces().triangulation(face),
        mesh.edges().polygon3d(edge),
        mesh.co_edges().polygon_on_surface(co_edge),
        mesh.co_edges().polygon_on_triangulation(co_edge),
    )
}

fn assert_mesh_handles_are_detached(copied: &MeshHandles, originals: &MeshHandles) {
    assert!(!copied.0.is_null());
    assert!(!copied.1.is_null());
    assert!(!copied.2.is_null());
    assert!(!copied.3.is_null());
    assert_ne!(copied.0.object_address(), originals.0.object_address());
    assert_ne!(copied.1.object_address(), originals.1.object_address());
    assert_ne!(copied.2.object_address(), originals.2.object_address());
    assert_ne!(copied.3.object_address(), originals.3.object_address());
}

fn assert_mesh_handle_values_match(copied: &MeshHandles, originals: &MeshHandles) {
    assert_triangulation_values_match(&copied.0, &originals.0);
    assert_polygon3d_values_match(&copied.1, &originals.1);
    assert_polygon2d_values_match(&copied.2, &originals.2);
    assert_polygon_on_tri_values_match(&copied.3, &originals.3);
}

fn assert_triangulation_values_match(copied: &Triangulation, original: &Triangulation) {
    assert_eq!(copied.nb_nodes(), original.nb_nodes());
    assert_eq!(copied.nb_triangles(), original.nb_triangles());
    assert_eq!(copied.node(2).unwrap(), original.node(2).unwrap());
    assert_eq!(copied.triangle(1).unwrap(), original.triangle(1).unwrap());
}

fn assert_polygon3d_values_match(copied: &Polygon3D, original: &Polygon3D) {
    assert_eq!(copied.nb_nodes(), original.nb_nodes());
    assert_eq!(
        copied.nodes().value(2).unwrap(),
        original.nodes().value(2).unwrap()
    );
}

fn assert_polygon2d_values_match(copied: &Polygon2D, original: &Polygon2D) {
    assert_eq!(copied.nb_nodes(), original.nb_nodes());
    assert_eq!(
        copied.nodes().value(2).unwrap(),
        original.nodes().value(2).unwrap()
    );
}

fn assert_polygon_on_tri_values_match(
    copied: &PolygonOnTriangulation,
    original: &PolygonOnTriangulation,
) {
    assert_eq!(copied.nb_nodes(), original.nb_nodes());
    assert_eq!(copied.node(2).unwrap(), original.node(2).unwrap());
}

fn attach_runtime_face_mesh(graph: &BRepGraph, face: FaceId, mesh: &Handle<Triangulation>) {
    graph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(face, mesh)
        .unwrap();
}

fn attach_runtime_edge_mesh(graph: &BRepGraph, edge: EdgeId) {
    graph
        .mesh()
        .editor()
        .edges()
        .set_cached_polygon3d(edge, &Handle::new(Polygon3D::new(2, false).unwrap()))
        .unwrap();
}

fn attach_runtime_coedge_mesh(graph: &BRepGraph, co_edge: CoEdgeId) {
    graph
        .mesh()
        .editor()
        .co_edges()
        .set_cached_polygon2d(co_edge, &Handle::new(Polygon2D::new(2).unwrap()))
        .unwrap();
    graph
        .mesh()
        .editor()
        .co_edges()
        .append_cached_polygon_on_tri(
            co_edge,
            &Handle::new(PolygonOnTriangulation::new(2, false).unwrap()),
        )
        .unwrap();
}

fn attach_runtime_edge_and_coedge_mesh(graph: &BRepGraph, edge: EdgeId, co_edge: CoEdgeId) {
    attach_runtime_edge_mesh(graph, edge);
    attach_runtime_coedge_mesh(graph, co_edge);
}

fn set_minimal_persistent_mesh(graph: &mut BRepGraph, face: FaceId, edge: EdgeId) {
    graph
        .editor()
        .faces()
        .set_persistent_triangulation(face, &triangulation())
        .unwrap();
    graph
        .editor()
        .edges()
        .set_persistent_polygon3d(edge, &Handle::new(Polygon3D::new(2, false).unwrap()))
        .unwrap();
}

fn attach_runtime_meshes(
    graph: &BRepGraph,
    ids: (FaceId, EdgeId, CoEdgeId),
    handles: &MeshHandles,
) {
    let (face, edge, co_edge) = ids;
    attach_runtime_face_mesh(graph, face, &handles.0);
    attach_runtime_edge_mesh_handle(graph, edge, &handles.1);
    attach_runtime_coedge_mesh_handles(graph, co_edge, &handles.2, &handles.3);
}

fn attach_runtime_edge_mesh_handle(graph: &BRepGraph, edge: EdgeId, mesh: &Handle<Polygon3D>) {
    graph
        .mesh()
        .editor()
        .edges()
        .set_cached_polygon3d(edge, mesh)
        .unwrap();
}

fn attach_runtime_coedge_mesh_handles(
    graph: &BRepGraph,
    co_edge: CoEdgeId,
    polygon2d: &Handle<Polygon2D>,
    polygon_on_tri: &Handle<PolygonOnTriangulation>,
) {
    graph
        .mesh()
        .editor()
        .co_edges()
        .set_cached_polygon2d(co_edge, polygon2d)
        .unwrap();
    graph
        .mesh()
        .editor()
        .co_edges()
        .append_cached_polygon_on_tri(co_edge, polygon_on_tri)
        .unwrap();
}

fn mesh_ids_after_append(
    counts: (u32, u32, u32),
    source: (FaceId, EdgeId, CoEdgeId),
) -> (FaceId, EdgeId, CoEdgeId) {
    (
        FaceId::new(counts.0 + source.0.index),
        EdgeId::new(counts.1 + source.1.index),
        CoEdgeId::new(counts.2 + source.2.index),
    )
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyGraph_DropsRuntimeFaceCacheMesh() {
    let source = box_graph();
    let face = FaceId::start();
    attach_runtime_face_mesh(&source, face, &triangulation());
    assert!(source.mesh().cache().faces().has(face));
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert!(!target.mesh().cache().faces().has(face));
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyGraph_CanCopyFreshRuntimeFaceCacheMesh() {
    let source = box_graph();
    let face = FaceId::start();
    let source_mesh = triangulation();
    attach_runtime_face_mesh(&source, face, &source_mesh);
    let mut target = BRepGraph::new();
    copy::perform(
        &source,
        &mut target,
        GeomPolicy::Copy,
        MeshPolicy::Copy,
        CachePolicy::CopyFresh,
    )
    .unwrap();
    let copied_mesh = target.mesh().cache().faces().triangulation(face);
    assert!(!copied_mesh.is_null());
    assert_eq!(copied_mesh.object_address(), source_mesh.object_address());
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyGraph_DropsRuntimeEdgeAndCoEdgeCacheMesh() {
    let mut source = box_graph();
    let (face, edge, co_edge) = first_face_mesh_ids(&source);
    source
        .editor()
        .faces()
        .set_persistent_triangulation(face, &triangulation())
        .unwrap();
    attach_runtime_edge_and_coedge_mesh(&source, edge, co_edge);
    assert!(source.mesh().cache().edges().entry(edge).is_some());
    assert!(source.mesh().cache().co_edges().has(co_edge));
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert!(target.mesh().cache().edges().entry(edge).is_none());
    assert!(!target.mesh().cache().co_edges().has(co_edge));
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyGraph_CopiesPersistentMeshHandles() {
    let mut source = box_graph();
    let ids = first_face_mesh_ids(&source);
    let originals = mesh_handles();
    install_mesh_handles(&mut source, ids, &originals);
    let mut target = BRepGraph::new();
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    assert_persistent_mesh_handles_copied(&target, ids, &originals);
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyNode_CopiesPersistentMeshAndDropsCache() {
    let mut source = box_graph();
    let ids = first_face_mesh_ids(&source);
    let handles = mesh_handles();
    install_mesh_handles(&mut source, ids, &handles);
    attach_runtime_meshes(&source, ids, &handles);
    let mut target = BRepGraph::new();
    copy_face_node(&source, &mut target, ids.0);
    let copied_ids = first_face_mesh_ids(&target);
    assert_persistent_mesh_handles_copied(&target, copied_ids, &handles);
    assert_runtime_meshes_absent(&target, copied_ids);
}

fn copy_face_node(source: &BRepGraph, target: &mut BRepGraph, face: FaceId) {
    copy::copy_node(
        source,
        target,
        face.into(),
        GeomPolicy::Copy,
        MeshPolicy::Copy,
        CachePolicy::Drop,
    )
    .unwrap();
}

fn assert_runtime_meshes_absent(graph: &BRepGraph, ids: (FaceId, EdgeId, CoEdgeId)) {
    assert!(!graph.mesh().cache().faces().has(ids.0));
    assert!(graph.mesh().cache().edges().entry(ids.1).is_none());
    assert!(!graph.mesh().cache().co_edges().has(ids.2));
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyGraph_NonEmptyTargetUsesSetterPathForPersistentMesh() {
    let mut source = box_graph();
    let ids = first_face_mesh_ids(&source);
    let originals = mesh_handles();
    install_mesh_handles(&mut source, ids, &originals);
    let mut target = box_graph();
    let counts = (
        target.topo().faces().nb(),
        target.topo().edges().nb(),
        target.topo().co_edges().nb(),
    );
    copy_graph(&source, &mut target, GeomPolicy::Copy);
    let copied_ids = mesh_ids_after_append(counts, ids);
    assert_persistent_mesh_handles_copied(&target, copied_ids, &originals);
}

#[test]
#[allow(non_snake_case)]
fn BRepGraph_CopyTest__CopyNode_DoesNotReviveRemovedPersistentMeshReps() {
    let (source, face) = graph_with_removed_persistent_meshes();
    let mut target = BRepGraph::new();
    copy_face_node(&source, &mut target, face);
    assert_copied_persistent_meshes_absent(&target);
}

fn graph_with_removed_persistent_meshes() -> (BRepGraph, FaceId) {
    let mut graph = box_graph();
    let (face, edge, _) = first_face_mesh_ids(&graph);
    set_minimal_persistent_mesh(&mut graph, face, edge);
    graph
        .editor()
        .faces()
        .clear_persistent_triangulation(face)
        .unwrap();
    graph
        .editor()
        .edges()
        .clear_persistent_polygon3d(edge)
        .unwrap();
    assert!(!graph.mesh().persistent().faces().has(face));
    assert!(!graph.mesh().persistent().edges().has(edge));
    (graph, face)
}

fn assert_copied_persistent_meshes_absent(graph: &BRepGraph) {
    let copied_face = FaceId::start();
    let copied_co_edge = first_co_edge_of_face(graph, copied_face);
    let copied_edge = graph
        .topo()
        .co_edges()
        .definition(copied_co_edge)
        .child_edge_id;
    assert!(!graph.mesh().persistent().faces().has(copied_face));
    assert!(!graph.mesh().persistent().edges().has(copied_edge));
}

