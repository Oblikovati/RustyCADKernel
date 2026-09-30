fn geometry_surface_state(graph: &BRepGraph, id: FaceId) -> (FaceSurfaceRepId, u32) {
    let rep = graph.topo().faces().definition(id).surface_rep_id;
    let active = graph.topo().geometry().nb_active_face_surfaces();
    (rep, active)
}
fn geometry_curve3d_state(graph: &BRepGraph, id: EdgeId) -> (EdgeCurve3DRepId, u32) {
    let rep = graph.topo().edges().definition(id).curve3d_rep_id;
    let active = graph.topo().geometry().nb_active_edge_curves3d();
    (rep, active)
}
fn geometry_pcurve_state(graph: &BRepGraph, id: CoEdgeId) -> (CoEdgeCurve2DRepId, u32) {
    let rep = graph.topo().co_edges().definition(id).curve2d_rep_id;
    let active = graph.topo().geometry().nb_active_co_edge_curves2d();
    (rep, active)
}
fn geometry_triangulation_state(graph: &BRepGraph, id: FaceId) -> (FaceTriangulationRepId, u32) {
    let rep = graph.topo().faces().definition(id).triangulation_rep_id;
    let active = graph.mesh().poly().nb_active_triangulations();
    (rep, active)
}
fn geometry_polygon3d_state(graph: &BRepGraph, id: EdgeId) -> (EdgePolygon3DRepId, u32) {
    let rep = graph.topo().edges().definition(id).polygon3d_rep_id;
    let active = graph.mesh().poly().nb_active_polygons3d();
    (rep, active)
}
fn geometry_polygon2d_state(graph: &BRepGraph, id: CoEdgeId) -> (CoEdgePolygon2DRepId, u32) {
    let rep = graph.topo().co_edges().definition(id).polygon2d_rep_id;
    let active = graph.mesh().poly().nb_active_polygons2d();
    (rep, active)
}
fn geometry_polygon_on_tri_state(
    graph: &BRepGraph,
    id: CoEdgeId,
) -> (CoEdgePolygonOnTriRepId, u32) {
    let rep = graph.topo().co_edges().definition(id).polygon_on_tri_rep_id;
    let active = graph.mesh().poly().nb_active_polygons_on_tri();
    (rep, active)
}
fn geometry_set_triangulation(graph: &mut BRepGraph, id: FaceId, mesh: &Handle<Triangulation>) {
    let editor = graph.editor();
    let mut entries = editor.faces();
    entries.set_persistent_triangulation(id, mesh).unwrap();
}
fn geometry_clear_triangulation(graph: &mut BRepGraph, id: FaceId) {
    let editor = graph.editor();
    let mut entries = editor.faces();
    entries.clear_persistent_triangulation(id).unwrap();
}
fn geometry_set_polygon3d(graph: &mut BRepGraph, id: EdgeId, mesh: &Handle<Polygon3D>) {
    let editor = graph.editor();
    let mut entries = editor.edges();
    entries.set_persistent_polygon3d(id, mesh).unwrap();
}
fn geometry_clear_polygon3d(graph: &mut BRepGraph, id: EdgeId) {
    let editor = graph.editor();
    let mut entries = editor.edges();
    entries.clear_persistent_polygon3d(id).unwrap();
}
fn geometry_set_polygon2d(graph: &mut BRepGraph, id: CoEdgeId, mesh: &Handle<Polygon2D>) {
    let editor = graph.editor();
    let mut entries = editor.co_edges();
    entries.set_persistent_polygon2d(id, mesh).unwrap();
}
fn geometry_set_polygon_on_tri(
    graph: &mut BRepGraph,
    id: CoEdgeId,
    mesh: &Handle<PolygonOnTriangulation>,
) {
    let editor = graph.editor();
    let mut entries = editor.co_edges();
    entries.set_persistent_polygon_on_tri(id, mesh).unwrap();
}
