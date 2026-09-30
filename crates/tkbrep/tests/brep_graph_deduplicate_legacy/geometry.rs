fn nbUniqueFaceSurfaceDefs(graph: &BRepGraph) -> usize {
    let mut surfaces = std::collections::HashSet::new();
    for index in 0..graph.topo().faces().nb() {
        let surface = tool::face::surface(graph, FaceId::new(index));
        if !surface.is_null() {
            surfaces.insert(surface.object_address());
        }
    }
    surfaces.len()
}

fn nbUniqueEdgeCurveDefs(graph: &BRepGraph) -> usize {
    let mut curves = std::collections::HashSet::new();
    for index in 0..graph.topo().edges().nb() {
        let curve = tool::edge::curve(graph, EdgeId::new(index));
        if !curve.is_null() {
            curves.insert(curve.object_address());
        }
    }
    curves.len()
}

fn nbPCurveEntries(graph: &BRepGraph) -> usize {
    let edges = (0..graph.topo().edges().nb()).map(EdgeId::new);
    edges
        .filter(|id| !id.is_removed(graph))
        .map(|edge| dedup_edge_pcurve_entries(graph, edge))
        .sum()
}

fn dedup_edge_pcurve_entries(graph: &BRepGraph, edge: EdgeId) -> usize {
    let ids = graph.topo().edges().co_edges(edge);
    ids.iter()
        .filter(|&&id| graph.topo().co_edges().definition(id).face_id.is_valid())
        .count()
}

fn nbUniquePCurveNodes(graph: &BRepGraph) -> usize {
    let edges = (0..graph.topo().edges().nb()).map(EdgeId::new);
    let counts = edges.map(|edge| graph.topo().edges().co_edges(edge).len());
    counts.sum()
}

fn addDuplicatePCurvesToAllEdges(graph: &mut BRepGraph) -> u32 {
    let mut count = 0;
    for index in 0..graph.topo().edges().nb() {
        count += dedup_duplicate_first_pcurve(graph, EdgeId::new(index));
    }
    count
}

fn dedup_duplicate_first_pcurve(graph: &mut BRepGraph, edge: EdgeId) -> u32 {
    let Some(&coedge) = graph.topo().edges().co_edges(edge).first() else {
        return 0;
    };
    let definition = graph.topo().co_edges().definition(coedge);
    if !definition.curve2d_rep_id.is_valid() {
        return 0;
    }
    let (face, orientation) = (definition.face_id, definition.orientation);
    let range = tool::co_edge::range(graph, coedge);
    let curve = tool::co_edge::pcurve(graph, coedge);
    graph
        .editor()
        .co_edges()
        .add_with_pcurve(edge, face, &curve, range.0, range.1, orientation);
    1
}

fn countHistoryRecordsByOp(graph: &BRepGraph, operation: &str) -> u32 {
    let handle = graph.layer_registry().find::<LayerHistory>();
    let Some(history) = handle.get() else {
        return 0;
    };
    (0..history.nb_records())
        .filter(|&index| {
            history
                .record_at(index, |event| {
                    event.operation_name.as_bytes() == operation.as_bytes()
                })
                .unwrap()
        })
        .count() as u32
}

fn countValidCoEdgePCurves(graph: &BRepGraph) -> usize {
    let ids = (0..graph.topo().co_edges().nb()).map(CoEdgeId::new);
    ids.filter(|&id| {
        !id.is_removed(graph)
            && graph
                .topo()
                .co_edges()
                .definition(id)
                .curve2d_rep_id
                .is_valid()
    })
    .filter(|&id| !tool::co_edge::pcurve(graph, id).is_null())
    .count()
}

fn countSeamCoEdges(graph: &BRepGraph) -> usize {
    let ids = (0..graph.topo().co_edges().nb()).map(CoEdgeId::new);
    ids.filter(|&id| dedup_has_seam_partner(graph, id)).count()
}

fn dedup_assert_face_representations(graph: &BRepGraph) {
    for index in 0..graph.topo().faces().nb() {
        let definition = graph.topo().faces().definition(FaceId::new(index));
        assert!(
            definition.surface_rep_id.is_valid(),
            "Face {index} has null Surface after dedup"
        );
    }
}

fn dedup_assert_edge_representations(graph: &BRepGraph) {
    for index in 0..graph.topo().edges().nb() {
        let edge = EdgeId::new(index);
        if tool::edge::degenerated(graph, edge) {
            continue;
        }
        assert!(
            graph
                .topo()
                .edges()
                .definition(edge)
                .curve3d_rep_id
                .is_valid(),
            "Edge {index} has null Curve3d after dedup"
        );
    }
}

fn dedup_assert_face_handles(graph: &BRepGraph) {
    for index in 0..graph.topo().faces().nb() {
        let face = FaceId::new(index);
        assert!(
            tool::face::has_surface(graph, face),
            "Face {index} has null Surface after dedup"
        );
    }
}

fn dedup_assert_edge_handles(graph: &BRepGraph) {
    for index in 0..graph.topo().edges().nb() {
        let edge = EdgeId::new(index);
        if tool::edge::degenerated(graph, edge) {
            continue;
        }
        assert!(
            tool::edge::has_curve(graph, edge),
            "Edge {index} has null Curve3d after dedup"
        );
    }
}

fn dedup_assert_pcurve_handles(graph: &BRepGraph) {
    for index in 0..graph.topo().co_edges().nb() {
        let id = CoEdgeId::new(index);
        if id.is_removed(graph)
            || !graph
                .topo()
                .co_edges()
                .definition(id)
                .curve2d_rep_id
                .is_valid()
        {
            continue;
        }
        assert!(
            !tool::co_edge::pcurve(graph, id).is_null(),
            "CoEdge {index} PCurve is null"
        );
    }
}
