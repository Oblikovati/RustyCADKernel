#[test]
fn BRepGraph_DeduplicateTest__HistoryFindOriginal_TracesBackToCanonical() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    let options = Options {
        history_mode: true,
        ..Options::default()
    };
    let result = deduplicate::perform_with(&mut graph, &options).unwrap();
    assert_eq!(result.nb_history_records, 5);
    let handle = dedup_history(&graph);
    let history = handle.get().unwrap();
    for (original, replacements) in dedup_history_mappings(history) {
        for replacement in replacements {
            assert!(history.find_original(replacement).is_valid());
            assert!(original.is_valid());
        }
    }
}

#[test]
fn BRepGraph_DeduplicateTest__HistoryFindDerived_ContainsCanonicalNode() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    let options = Options {
        history_mode: true,
        ..Options::default()
    };
    let result = deduplicate::perform_with(&mut graph, &options).unwrap();
    assert_eq!(result.nb_history_records, 5);
    let handle = dedup_history(&graph);
    let history = handle.get().unwrap();
    let mut count = 0;
    for (original, _) in dedup_history_mappings(history) {
        assert_eq!(history.find_derived(original).len(), 1);
        count += 1;
    }
    assert_eq!(count, 5);
}

#[test]
fn BRepGraph_DeduplicateTest__HistoryRecordSequenceNumbers_AreMonotonic() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    let options = Options {
        history_mode: true,
        ..Options::default()
    };
    deduplicate::perform_with(&mut graph, &options).unwrap();
    let handle = dedup_history(&graph);
    let history = handle.get().unwrap();
    dedup_assert_monotonic_sequences(history);
}

fn dedup_history_mappings(history: &LayerHistory) -> Vec<(NodeId, Vec<NodeId>)> {
    let mut mappings = Vec::new();
    for index in 0..history.nb_records() {
        history
            .record_at(index, |event| {
                mappings.extend(
                    event
                        .mapping
                        .items()
                        .map(|(&key, value)| (key, value.clone())),
                )
            })
            .unwrap();
    }
    mappings
}

#[test]
fn BRepGraph_DeduplicateTest__AnalyzeOnly_NoBackRefChangesOrNullification() {
    let mut graph = dedup_graph(&makeTwoCopiedIdenticalFaces());
    assert!(!graph.is_empty());
    let surfaces: Vec<usize> = (0..graph.topo().faces().nb())
        .map(|index| dedup_surface_pointer(&graph, index))
        .collect();
    let curves: Vec<usize> = (0..graph.topo().edges().nb())
        .map(|index| dedup_curve_pointer(&graph, index))
        .collect();
    let options = Options {
        analyze_only: true,
        ..Options::default()
    };
    let result = deduplicate::perform_with(&mut graph, &options).unwrap();
    assert_eq!(result.nb_nullified_surfaces, 0);
    assert_eq!(result.nb_nullified_curves, 0);
    dedup_assert_pointer_snapshots(&graph, &surfaces, &curves);
    dedup_assert_face_handles(&graph);
    dedup_assert_edge_handles(&graph);
}

fn dedup_assert_pointer_snapshots(graph: &BRepGraph, surfaces: &[usize], curves: &[usize]) {
    for index in 0..graph.topo().faces().nb() {
        assert_eq!(
            dedup_surface_pointer(graph, index),
            surfaces[index as usize]
        );
    }
    for index in 0..graph.topo().edges().nb() {
        assert_eq!(dedup_curve_pointer(graph, index), curves[index as usize]);
    }
}

#[test]
fn BRepGraph_DeduplicateTest__ParallelBuild_SameResultAsSequential() {
    let compound = makeTwoCopiedIdenticalFaces();
    let mut sequential = dedup_parallel_graph(&compound, false);
    assert!(!sequential.is_empty());
    let mut parallel = dedup_parallel_graph(&compound, true);
    assert!(!parallel.is_empty());
    let first = deduplicate::perform(&mut sequential).unwrap();
    let second = deduplicate::perform(&mut parallel).unwrap();
    assert_eq!(first.nb_canonical_surfaces, 1);
    assert_eq!(second.nb_canonical_surfaces, 1);
    assert_eq!(first.nb_canonical_curves, 4);
    assert_eq!(second.nb_canonical_curves, 4);
    assert_eq!(first.nb_surface_rewrites, 1);
    assert_eq!(second.nb_surface_rewrites, 1);
    assert_eq!(first.nb_curve_rewrites, 4);
    assert_eq!(second.nb_curve_rewrites, 4);
}

fn dedup_parallel_graph(shape: &Shape, parallel: bool) -> BRepGraph {
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    let options = ShapeOptions {
        create_auto_product: true,
        flatten: false,
        parallel,
        ..ShapeOptions::default()
    };
    graph
        .shapes_mut()
        .add_with_options(shape, &options)
        .unwrap();
    graph
}

// OCCT 3d097a0 marks this original assertion DISABLED_: inline pcurve entries
// are retained. Kept as a reference assertion, not counted as a passing port.
#[allow(dead_code)]
fn BRepGraph_DeduplicateTest__DISABLED_PCurveDedup_RewritesReduceUniquePCurveNodes() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    addDuplicatePCurvesToAllEdges(&mut graph);
    let before = nbUniquePCurveNodes(&graph);
    deduplicate::perform(&mut graph).unwrap();
    assert!(nbUniquePCurveNodes(&graph) < before);
}

#[test]
fn compatibility_disabled_pcurve_dedup_retains_inline_entries() {
    let mut graph = dedup_graph(&dedup_copy(&dedup_box_face()));
    assert!(!graph.is_empty());
    addDuplicatePCurvesToAllEdges(&mut graph);
    let before = nbUniquePCurveNodes(&graph);
    deduplicate::perform(&mut graph).unwrap();
    assert_eq!(before, 8);
    assert_eq!(nbUniquePCurveNodes(&graph), before);
}

fn dedup_assert_monotonic_sequences(history: &LayerHistory) {
    let mut previous = None;
    for index in 0..history.nb_records() {
        let sequence = history
            .record_at(index, |event| event.sequence_number)
            .unwrap();
        if let Some(before) = previous {
            assert!(sequence > before);
        }
        previous = Some(sequence);
    }
}
