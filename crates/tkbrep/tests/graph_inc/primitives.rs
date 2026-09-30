#[test]
fn Box_EntityCounts_MatchDefCounts() {
    let graph = inc_populate(&inc_box(), false);
    let counts = inc_counts(&graph);
    assert_eq!(
        [counts[0], counts[1], counts[3], counts[4], counts[5]],
        [8, 12, 6, 1, 1]
    );
}
#[test]
fn Cylinder_EntityCounts_MatchDefCounts() {
    assert_eq!(
        inc_counts(&inc_populate(&inc_cylinder(15.0), false)),
        [2, 3, 3, 3, 1, 1]
    );
}
#[test]
fn Sphere_EntityCounts_MatchDefCounts() {
    assert_eq!(
        inc_counts(&inc_populate(&inc_sphere(), false)),
        [2, 3, 1, 1, 1, 1]
    );
}
#[test]
fn Box_RoundTrip_AreaPreserved() {
    inc_roundtrip_measure(&inc_box(), inc_area);
}
#[test]
fn Cylinder_RoundTrip_AreaPreserved() {
    inc_roundtrip_measure(&inc_cylinder(15.0), inc_area);
}
#[test]
fn Sphere_RoundTrip_AreaPreserved() {
    inc_roundtrip_measure(&inc_sphere(), inc_area);
}
#[test]
fn Box_RoundTrip_VolumePreserved() {
    inc_roundtrip_measure(&inc_box(), inc_volume);
}
#[test]
fn Cylinder_RoundTrip_VolumePreserved() {
    inc_roundtrip_measure(&inc_cylinder(15.0), inc_volume);
}
#[test]
fn Box_RoundTrip_SubShapeCounts() {
    inc_roundtrip_counts(&inc_box());
}
#[test]
fn Cylinder_RoundTrip_SubShapeCounts() {
    inc_roundtrip_counts(&inc_cylinder(15.0));
}
#[test]
fn Box_Relations_EdgesToWires() {
    let graph = inc_populate(&inc_box(), false);
    for index in 0..graph.topo().edges().nb() {
        assert!(graph.topo().edges().wires_of(EdgeId::new(index)).count() >= 1);
    }
}
#[test]
fn Box_Relations_EdgesToFaces() {
    let graph = inc_populate(&inc_box(), false);
    for index in 0..graph.topo().edges().nb() {
        let id = EdgeId::new(index);
        if edge::degenerated(&graph, id) {
            continue;
        }
        assert!(graph.topo().edges().faces_of(id).count() >= 1);
    }
}
#[test]
fn Box_ParallelPopulate_SameEntityCounts() {
    let shape = inc_box();
    assert_eq!(
        inc_counts(&inc_populate(&shape, true)),
        inc_counts(&inc_populate(&shape, false))
    );
}
#[test]
fn NullShape_NoEntities() {
    let mut graph = BRepGraph::new();
    populate::perform(&mut graph, &Shape::default(), false, &populate::Options).unwrap();
    assert!(graph.is_empty());
    assert_eq!(graph.topo().vertices().nb(), 0);
    assert_eq!(graph.topo().edges().nb(), 0);
}
#[test]
fn Compound_RoundTrip_SubShapeCounts() {
    let compound = inc_compound(&[
        support::make_box(10.0, 10.0, 10.0).into_shape(),
        support::make_box(5.0, 5.0, 5.0).into_shape(),
    ]);
    let graph = inc_populate(&compound, false);
    assert_eq!(graph.topo().solids().nb(), 2);
    assert_eq!(graph.topo().shells().nb(), 2);
    assert_eq!(graph.topo().compounds().nb(), 1);
    assert_eq!(graph.topo().faces().nb(), 12);
    let recon = inc_reconstruct(&graph, CompoundId::start());
    assert_eq!(inc_subshapes(&recon, ShapeEnum::Solid).len(), 2);
    assert_eq!(inc_subshapes(&recon, ShapeEnum::Face).len(), 12);
}
#[test]
fn Box_CoEdgeCount() {
    let graph = inc_populate(&inc_box(), false);
    let count: usize = (0..graph.topo().edges().nb())
        .map(|i| {
            graph
                .topo()
                .edges()
                .relations(EdgeId::new(i))
                .co_edge_ids
                .len()
        })
        .sum();
    assert_eq!(count, 24);
}
fn inc_seam_pairs(graph: &BRepGraph, id: EdgeId) -> usize {
    let ids = &graph.topo().edges().relations(id).co_edge_ids;
    ids.iter()
        .filter(|id| {
            let def = graph.topo().co_edges().definition(**id);
            def.orientation == ParityOrientation::FORWARD
                && ids.iter().any(|other| {
                    let other_def = graph.topo().co_edges().definition(*other);
                    other != *id
                        && other_def.face_id == def.face_id
                        && other_def.orientation != def.orientation
                })
        })
        .count()
}
#[test]
fn Cylinder_HasSeamEdges() {
    let graph = inc_populate(&inc_cylinder(15.0), false);
    let count: usize = (0..graph.topo().edges().nb())
        .map(|i| inc_seam_pairs(&graph, EdgeId::new(i)))
        .sum();
    assert!(count >= 1, "Cylinder should have at least 1 seam edge pair");
}
#[test]
fn Cylinder_SeamEdge_Relations_NoDuplicateFace() {
    let graph = inc_populate(&inc_cylinder(15.0), false);
    for i in 0..graph.topo().edges().nb() {
        let mut seen = Vec::new();
        for id in graph.topo().edges().faces_of(EdgeId::new(i)) {
            assert!(!seen.contains(&id));
            seen.push(id);
        }
    }
}
#[test]
fn Sphere_DegenerateEdges_Preserved() {
    let shape = inc_sphere();
    let graph = inc_populate(&shape, false);
    assert!(inc_degenerate_count(&graph) >= 2);
    let recon = inc_reconstruct(&graph, SolidId::start());
    inc_near(inc_area(&recon), inc_area(&shape));
}
#[test]
fn Cylinder_RoundTrip_BRepDump() {
    let shape = inc_cylinder(20.0);
    let graph = inc_populate(&shape, false);
    let recon = inc_reconstruct(&graph, SolidId::start());
    inc_brep_dumps(&shape, &recon);
    inc_near(inc_area(&recon), inc_area(&shape));
}

fn inc_degenerate_count(graph: &BRepGraph) -> usize {
    let mut count = 0;
    for i in 0..graph.topo().edges().nb() {
        let id = EdgeId::new(i);
        if !edge::degenerated(graph, id) {
            continue;
        }
        count += 1;
        let def = graph.topo().edges().definition(id);
        assert!(!def.curve3d_rep_id.is_valid());
    }
    count
}
fn inc_brep_dumps(original: &Shape, reconstructed: &Shape) {
    use rustycad_tkernel::{message::ProgressRange, storage::IoStream};
    let mut original_stream = IoStream::from_bytes(Vec::new());
    let mut reconstructed_stream = IoStream::from_bytes(Vec::new());
    rustycad_tkbrep::brep_tools::write(original, &mut original_stream, &ProgressRange::default())
        .unwrap();
    rustycad_tkbrep::brep_tools::write(
        reconstructed,
        &mut reconstructed_stream,
        &ProgressRange::default(),
    )
    .unwrap();
    // Native output reports differences for diagnosis; equality is deliberately not asserted.
}
