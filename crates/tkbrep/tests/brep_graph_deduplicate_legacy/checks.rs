fn dedup_surface_pointer(graph: &BRepGraph, index: u32) -> usize {
    let id = FaceId::new(index);
    let surface = tool::face::surface(graph, id);
    surface.object_address() as usize
}

fn dedup_curve_pointer(graph: &BRepGraph, index: u32) -> usize {
    let id = EdgeId::new(index);
    let curve = tool::edge::curve(graph, id);
    curve.object_address() as usize
}

fn dedup_assert_pcurve_representations(graph: &BRepGraph) {
    for index in 0..graph.topo().edges().nb() {
        for (entry, &coedge) in graph
            .topo()
            .edges()
            .co_edges(EdgeId::new(index))
            .iter()
            .enumerate()
        {
            let representation = graph.topo().co_edges().definition(coedge).curve2d_rep_id;
            assert!(
                representation.is_valid(),
                "Edge {index} CoEdge {entry} has null Curve2d after dedup"
            );
        }
    }
}

fn dedup_reconstructed_faces(graph: &BRepGraph) -> Shape {
    let faces: Vec<Shape> = (0..graph.topo().faces().nb())
        .map(|index| {
            let shape = graph
                .shapes()
                .reconstruct(FaceId::new(index).into())
                .unwrap();
            assert!(!shape.is_null());
            shape
        })
        .collect();
    dedup_compound(&faces)
}

fn dedup_assert_reconstructed_root(graph: &BRepGraph) {
    assert_eq!(graph.root_product_ids().len(), 1);
    let root = graph.root_product_ids()[0];
    let shape = graph.shapes().reconstruct(root.into()).unwrap();
    assert!(!shape.is_null());
    assert!(
        Analyzer::new(&shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

fn dedup_assert_reconstructed_faces(graph: &BRepGraph) {
    for index in 0..graph.topo().faces().nb() {
        let id = FaceId::new(index);
        if id.is_removed(graph) {
            continue;
        }
        let shape = graph.shapes().reconstruct(id.into()).unwrap();
        assert!(!shape.is_null());
        assert!(
            Analyzer::new(&shape, true, false, false)
                .unwrap()
                .is_valid()
                .unwrap(),
            "Reconstructed face {index} is not valid"
        );
    }
}

fn dedup_assert_reconstructed_solids(graph: &BRepGraph, check_geometry: bool) {
    for index in 0..graph.topo().solids().nb() {
        let id = SolidId::new(index);
        if id.is_removed(graph) {
            continue;
        }
        let shape = graph.shapes().reconstruct(id.into()).unwrap();
        assert!(!shape.is_null());
        dedup_assert_solid_shape(&shape, index, check_geometry);
    }
}

fn dedup_assert_solid_shape(shape: &Shape, index: u32, check_geometry: bool) {
    if !check_geometry {
        assert_eq!(shape.shape_type().unwrap(), ShapeEnum::Solid);
        return;
    }
    assert!(
        Analyzer::new(shape, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap(),
        "Reconstructed solid {index} is not valid"
    );
}

fn dedup_assert_three_box_merges(result: &DeduplicateResult) {
    assert_eq!(result.nb_merged_vertices, 16);
    assert_eq!(result.nb_merged_edges, 24);
    assert_eq!(result.nb_merged_wires, 12);
    assert_eq!(result.nb_merged_faces, 12);
}

fn dedup_assert_outer_wires(graph: &BRepGraph) {
    for index in 0..graph.topo().faces().nb() {
        let face = FaceId::new(index);
        if face.is_removed(graph) {
            continue;
        }
        let refs = &graph.topo().faces().relations(face).wire_ref_ids;
        let count = refs
            .iter()
            .filter(|&&id| !graph.refs().generic().is_removed(id.into()))
            .count();
        assert_eq!(count, 1, "Face {index} should have exactly 1 wire");
    }
}

fn dedup_distinct_edge_faces(graph: &BRepGraph, edge: EdgeId) -> usize {
    let mut faces = std::collections::HashSet::new();
    for &id in graph.topo().edges().co_edges(edge) {
        let coedge = graph.topo().co_edges().definition(id);
        if !id.is_removed(graph) && coedge.face_id.is_valid() {
            faces.insert(coedge.face_id);
        }
    }
    faces.len()
}

fn dedup_rectangle_on_plane(x1: f64, y1: f64, x2: f64, y2: f64) -> Shape {
    let points = [
        Point3::new(x1, y1, 0.),
        Point3::new(x2, y1, 0.),
        Point3::new(x2, y2, 0.),
        Point3::new(x1, y2, 0.),
    ];
    let mut wire = Wire::default();
    let builder = BRepBuilder::default();
    builder.make_wire(&mut wire);
    for index in 0..4 {
        let edge = support::make_edge(points[index], points[(index + 1) % 4]);
        builder.add(&mut wire, &edge).unwrap();
    }
    dedup_face_on_default_plane(&wire)
}

fn dedup_has_seam_partner(graph: &BRepGraph, id: CoEdgeId) -> bool {
    let coedge = graph.topo().co_edges().definition(id);
    if id.is_removed(graph) || !coedge.face_id.is_valid() || !coedge.child_edge_id.is_valid() {
        return false;
    }
    graph
        .topo()
        .edges()
        .co_edges(coedge.child_edge_id)
        .iter()
        .any(|&other| {
            let sibling = graph.topo().co_edges().definition(other);
            other != id
                && !other.is_removed(graph)
                && sibling.face_id == coedge.face_id
                && sibling.orientation != coedge.orientation
        })
}

fn dedup_merge(graph: &mut BRepGraph) -> DeduplicateResult {
    let options = Options {
        merge_entities_when_safe: true,
        ..Options::default()
    };
    deduplicate::perform_with(graph, &options).unwrap()
}

fn dedup_face_on_default_plane(wire: &Wire) -> Shape {
    let plane = Plane3::new(Point3::default(), Direction3::Z).unwrap();
    BRepBuilderAPIMakeFace::from_plane_wire(plane, wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into()
}
