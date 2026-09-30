use rustycad_tkbrep::brep_graph::SolidId;
fn synthetic_frame(point: Point3, direction: Direction3) -> Axis3 {
    Axis3::from_axis2(Axis2::new(point, direction).unwrap())
}
fn synthetic_vertices<const N: usize>(graph: &mut BRepGraph, points: [Point3; N]) -> [VertexId; N] {
    points.map(|point| graph.editor().vertices().add(point, 1e-7))
}
fn synthetic_line(graph: &mut BRepGraph, first: VertexId, last: VertexId, length: f64) -> EdgeId {
    let start = graph.topo().vertices().definition(first).point;
    let end = graph.topo().vertices().definition(last).point;
    let direction =
        Direction3::from_vector(Vector3::between(start, end).normalized().unwrap()).unwrap();
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(start, direction))).into_cast();
    graph
        .editor()
        .edges()
        .add(first, last, &curve, 0.0, length, 1e-7)
}
fn synthetic_circle(
    graph: &mut BRepGraph,
    vertex: VertexId,
    center: Point3,
    normal: Direction3,
    radius: f64,
) -> EdgeId {
    let axis = Axis2::new(center, normal).unwrap();
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(axis, radius).unwrap()).into_cast();
    graph.editor().edges().add(
        vertex,
        vertex,
        &curve,
        0.0,
        2.0 * std::f64::consts::PI,
        1e-7,
    )
}
fn synthetic_wire(graph: &mut BRepGraph, entries: &[(EdgeId, ParityOrientation)]) -> WireId {
    let coedges: Vec<CoEdgeId> = entries
        .iter()
        .map(|(edge, orientation)| graph.editor().co_edges().add(*edge, *orientation))
        .collect();
    graph.editor().wires().add(&coedges)
}
fn synthetic_plane(point: Point3, normal: Direction3) -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(synthetic_frame(point, normal))).into_cast()
}
fn synthetic_face(
    graph: &mut BRepGraph,
    surface: &Handle<dyn GeomSurface>,
    wire: WireId,
) -> FaceId {
    graph.editor().faces().add(surface, wire, &[], 1e-7)
}
fn synthetic_solid(graph: &mut BRepGraph, faces: &[FaceId]) -> SolidId {
    let shell = graph.editor().shells().add();
    for face in faces {
        graph
            .editor()
            .shells()
            .append(shell, *face, ParityOrientation::FORWARD)
            .unwrap();
    }
    let solid = graph.editor().solids().add();
    graph
        .editor()
        .solids()
        .append(solid, shell, ParityOrientation::FORWARD)
        .unwrap();
    solid
}
fn synthetic_direction(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}
