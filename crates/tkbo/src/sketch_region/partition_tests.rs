// SPDX-License-Identifier: GPL-2.0-only
//! The seam observes real split data before its local mapped filler is destroyed.
use super::{outcome::*, partition_observation::observe_owned_partition, partition_types::*};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};

#[test]
fn crossing_edges_retain_original_parameter_partitions_after_filler_retirement() {
    let edges = crossing_partition_edges();
    let inputs = partition_inputs(&edges);
    let observed = observe_owned_partition(&inputs, &mut |_| false);
    assert!(matches!(
        observed.completion,
        ArrangementCompletion::Arranged(_)
    ));
    let PartitionObservation::Captured(snapshot) = observed.partition else {
        panic!("arrangement completed; expected owned original-edge partition observations");
    };
    assert_eq!(snapshot.sources.len(), 2);
    for source in snapshot.sources {
        assert_eq!(source.range, [0., 2.]);
        let spans: Vec<_> = source.blocks.iter().map(|block| block.own_range).collect();
        assert_eq!(spans, [[0., 1.], [1., 2.]]);
    }
}
fn crossing_partition_edges() -> Vec<Shape> {
    [([-1., 0., 0.], [1., 0., 0.]), ([0., -1., 0.], [0., 1., 0.])]
        .into_iter()
        .map(|(a, b)| {
            BRepBuilderAPIMakeEdge::from_points(
                Point3::new(a[0], a[1], a[2]),
                Point3::new(b[0], b[1], b[2]),
            )
            .unwrap()
            .shape()
            .unwrap()
            .clone()
        })
        .collect()
}
fn partition_inputs(shapes: &[Shape]) -> Vec<ArrangementSource<'_>> {
    shapes
        .iter()
        .enumerate()
        .map(|(index, shape)| ArrangementSource {
            identity: RegionSourceId(index as u64 + 1),
            shape,
        })
        .collect()
}

#[test]
fn reversed_occurrences_preserve_both_original_parent_orientations() {
    let mut edges = crossing_partition_edges();
    edges.push(edges[0].reversed());
    let observed = observe_owned_partition(&partition_inputs(&edges), &mut |_| false);
    let PartitionObservation::Captured(snapshot) = observed.partition else {
        panic!("expected captured partition for actual opposite-orientation aliases");
    };
    let shared = snapshot
        .sources
        .iter()
        .find(|row| row.parents.len() == 2)
        .unwrap();
    assert_ne!(shared.parents[0].orientation, shared.parents[1].orientation);
    assert_eq!(
        shared.parents[0].topology_address,
        shared.parents[1].topology_address
    );
    assert_eq!(shared.parents[0].source, RegionSourceId(1));
    assert_eq!(shared.parents[1].source, RegionSourceId(3));
}

#[test]
fn history_cancellation_retains_partition_but_preserves_original_completion() {
    let edges = crossing_partition_edges();
    let observed = observe_owned_partition(&partition_inputs(&edges), &mut |phase| {
        phase == ArrangementPhase::History
    });
    assert!(matches!(
        observed.completion,
        ArrangementCompletion::Cancelled(ArrangementPhase::History)
    ));
    assert!(matches!(
        observed.partition,
        PartitionObservation::Captured(_)
    ));
}

#[test]
fn interference_cancellation_never_accesses_uncreated_filler_context() {
    let edges = crossing_partition_edges();
    let observed = observe_owned_partition(&partition_inputs(&edges), &mut |phase| {
        phase == ArrangementPhase::Interference
    });
    assert!(matches!(
        observed.completion,
        ArrangementCompletion::Cancelled(ArrangementPhase::Interference)
    ));
    assert!(matches!(
        observed.partition,
        PartitionObservation::NotReached
    ));
}

#[test]
fn observation_preserves_default_progress_callback_order() {
    let edges = crossing_partition_edges();
    let inputs = partition_inputs(&edges);
    let mut mapped_phases = Vec::new();
    let mut observed_phases = Vec::new();
    super::arrangement::arrange_owned_sources(&inputs, &mut |phase| {
        mapped_phases.push(phase);
        false
    });
    observe_owned_partition(&inputs, &mut |phase| {
        observed_phases.push(phase);
        false
    });
    assert_eq!(mapped_phases, observed_phases);
    assert_eq!(observed_phases, expected_partition_phases());
}

#[test]
fn uninitialized_actual_filler_reports_missing_context_without_success() {
    let edges = crossing_partition_edges();
    let copied = super::copy_inputs::copy_sources(&partition_inputs(&edges))
        .unwrap_or_else(|_| panic!("valid crossing edges; expected successful owned batch copy"));
    let filler = crate::bop_algo::BOPAlgoPaveFiller::new();
    let receipt = super::partition_capture::capture_mapped_partition(&copied, &filler, None);
    let PartitionObservation::Incomplete { partial, failure } = receipt else {
        panic!("uninitialized filler; expected incomplete actual missing DS observation");
    };
    assert_eq!(partial.source_shape_count, None);
    assert!(partial.sources.is_empty());
    assert!(matches!(
        failure.cause,
        PartitionReadCause::MissingContext("filler DS is None")
    ));
}

#[test]
fn cancellation_then_success_does_not_reuse_prior_partition_state() {
    let edges = crossing_partition_edges();
    let inputs = partition_inputs(&edges);
    let cancelled = observe_owned_partition(&inputs, &mut |_| true);
    assert!(matches!(
        cancelled.partition,
        PartitionObservation::NotReached
    ));
    let first = captured_partition(&inputs);
    let second = captured_partition(&inputs);
    assert_eq!(first.sources.len(), second.sources.len());
    for (old, new) in first.sources.iter().zip(&second.sources) {
        assert_eq!(old.range, new.range);
        assert_eq!(old.blocks.len(), new.blocks.len());
        assert_eq!(old.parents, new.parents);
    }
}
fn captured_partition(inputs: &[ArrangementSource<'_>]) -> OwnedPartitionSnapshot {
    let observed = observe_owned_partition(inputs, &mut |_| false);
    let PartitionObservation::Captured(snapshot) = observed.partition else {
        panic!("valid crossing inputs; expected captured owned partition snapshot");
    };
    snapshot
}

fn expected_partition_phases() -> [ArrangementPhase; 3] {
    [
        ArrangementPhase::Copy,
        ArrangementPhase::Interference,
        ArrangementPhase::History,
    ]
}

#[test]
fn actual_split_images_match_representative_edges_without_retaining_shapes() {
    let edges = crossing_partition_edges();
    let snapshot = captured_partition(&partition_inputs(&edges));
    for source in snapshot.sources {
        let images = source
            .images
            .expect("split source; expected actual bound image list");
        assert_eq!(images.len(), 2);
        for (image, block) in images.iter().zip(&source.blocks) {
            assert_eq!(image.shape_index, block.real_edge);
            assert!(image.occurrence.topology_address > 0);
            assert_eq!(block.original_edge, source.source_index);
            assert_eq!(block.real_original_edge, source.source_index);
        }
    }
}

#[test]
fn actual_unsplit_source_keeps_unbound_image_distinct_from_empty_binding() {
    let mut edges = crossing_partition_edges();
    edges.truncate(1);
    let snapshot = captured_partition(&partition_inputs(&edges));
    assert_eq!(snapshot.sources.len(), 1);
    assert_eq!(snapshot.sources[0].range, [0., 2.]);
    assert!(!snapshot.sources[0].has_reference);
    assert!(snapshot.sources[0].blocks.is_empty());
    assert!(snapshot.sources[0].images.is_none());
}

#[test]
fn same_geometry_distinct_topology_keeps_distinct_source_and_all_parents() {
    let mut edges = crossing_partition_edges();
    edges[1] = crossing_partition_edges().remove(0);
    assert!(!edges[0].is_same(&edges[1]));
    let snapshot = captured_partition(&partition_inputs(&edges));
    assert_eq!(snapshot.sources.len(), 2);
    assert_ne!(
        snapshot.sources[0].source_index,
        snapshot.sources[1].source_index
    );
    for source in snapshot.sources {
        assert_eq!(source.parents.len(), 1);
        assert_eq!(source.blocks.len(), 1);
        assert_eq!(source.blocks[0].own_range, [0., 2.]);
    }
}

#[test]
fn actual_invalid_pave_reference_preserves_typed_accessor_cause_and_parent() {
    let edges = crossing_partition_edges();
    let copied = copied_partition_inputs(&edges);
    let mut filler = crate::bop_algo::BOPAlgoPaveFiller::new();
    filler.set_arguments(&copied.arguments);
    filler.initialize().unwrap();
    let structure = filler.data_structure_mut().unwrap();
    let index = structure.index(&copied.roots[0].1);
    structure
        .change_shape_info(index)
        .unwrap()
        .set_reference(i32::MAX);
    let builder = crate::bop_algo::BOPAlgoBuilder::new();
    let observed =
        super::partition_capture::capture_mapped_partition(&copied, &filler, Some(&builder));
    assert_invalid_pave_reference(observed, index);
}
fn copied_partition_inputs(edges: &[Shape]) -> super::copy_inputs::CopiedSources {
    super::copy_inputs::copy_sources(&partition_inputs(edges))
        .unwrap_or_else(|_| panic!("valid crossing edges; expected successful owned batch copy"))
}
fn assert_invalid_pave_reference(observed: PartitionObservation, index: i32) {
    let PartitionObservation::Incomplete { failure, .. } = observed else {
        panic!("invalid pool reference; expected typed incomplete accessor failure");
    };
    assert_eq!(failure.source_index, Some(index));
    assert_eq!(failure.parents.len(), 1);
    assert_eq!(failure.parents[0].source, RegionSourceId(1));
    let PartitionReadCause::Native(cause) = failure.cause else {
        panic!("actual failed PaveBlocks accessor; expected original mapped Failure");
    };
    assert!(cause.message().contains("2147483647"));
}

#[test]
fn owned_partition_replays_attributed_native_source_pave_and_image_rows() {
    let expected = include_str!("../../tests/fixtures/partition-observation-native.txt");
    let mut actual = String::new();
    for mode in 0..5 {
        let edges = native_partition_mode(mode);
        let snapshot = captured_partition(&partition_inputs(&edges));
        actual.push_str(&format!(
            "mode {mode} {}\n",
            snapshot.source_shape_count.unwrap()
        ));
        for source in snapshot.sources {
            append_native_source(&mut actual, &source);
        }
    }
    assert_eq!(actual, expected);
}
fn native_partition_mode(mode: usize) -> Vec<Shape> {
    let mut edges = crossing_partition_edges();
    match mode {
        1 => edges.truncate(1),
        2 => edges[1] = crossing_partition_edges().remove(0),
        3 => edges.push(edges[0].reversed()),
        4 => {
            let mut shift = rustycad_tkmath::gp::Transform3::default();
            shift.set_translation(rustycad_tkmath::gp::Vector3::new(0., 3., 0.));
            edges[1] = edges[0]
                .located(&rustycad_tkmath::toploc::Location::new(shift), true)
                .unwrap();
        }
        _ => {}
    }
    edges
}
fn append_native_source(output: &mut String, source: &SourcePartitionObservation) {
    let count = source
        .images
        .as_ref()
        .map_or(-1, |images| images.len() as i64);
    output.push_str(&format!(
        "source {} {} {} {} {count}\n",
        source.source_index,
        source.range[0],
        source.range[1],
        i32::from(source.has_reference)
    ));
    for block in &source.blocks {
        append_native_block(output, block);
    }
    append_native_images(output, source);
}
fn append_native_images(output: &mut String, source: &SourcePartitionObservation) {
    if let Some(images) = &source.images {
        for image in images {
            output.push_str(&format!("image {}\n", image.shape_index));
        }
    }
}
fn append_native_block(output: &mut String, block: &PavePartitionObservation) {
    output.push_str(&format!(
        "block {} {} {} {} {} {} {} {} {} {} {} {}\n",
        block.original_edge,
        block.own_edge,
        block.real_original_edge,
        block.real_edge,
        block.own_range[0],
        block.own_range[1],
        block.real_range[0],
        block.real_range[1],
        block.own_vertices[0],
        block.own_vertices[1],
        block.real_vertices[0],
        block.real_vertices[1]
    ));
}

#[test]
fn differently_located_same_topology_has_distinct_native_source_binding() {
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Location,
    };
    let mut edges = crossing_partition_edges();
    let mut translation = Transform3::default();
    translation.set_translation(Vector3::new(0., 3., 0.));
    edges[1] = edges[0].located(&Location::new(translation), true).unwrap();
    let snapshot = captured_partition(&partition_inputs(&edges));
    assert_eq!(snapshot.sources.len(), 2);
    assert_ne!(
        snapshot.sources[0].parents[0].location,
        snapshot.sources[1].parents[0].location
    );
    assert_eq!(
        snapshot.sources[0].parents[0].topology_address,
        snapshot.sources[1].parents[0].topology_address
    );
}

#[test]
fn caller_replacement_does_not_mutate_owned_parameter_and_parent_receipt() {
    let mut edges = crossing_partition_edges();
    let snapshot = captured_partition(&partition_inputs(&edges));
    let address = edges[0].tshape().object_address().addr();
    edges[0] = Shape::default();
    assert!(edges[0].is_null());
    assert_eq!(snapshot.sources[0].parents[0].topology_address, address);
    assert_eq!(snapshot.sources[0].range, [0., 2.]);
    assert_eq!(snapshot.sources[0].blocks[0].own_range, [0., 1.]);
}

#[test]
fn actual_missing_copied_source_is_incomplete_with_exact_original_parent() {
    let edges = crossing_partition_edges();
    let copied = copied_partition_inputs(&edges);
    let mut filler = crate::bop_algo::BOPAlgoPaveFiller::new();
    filler.add_argument(&copied.roots[0].1);
    filler.initialize().unwrap();
    let builder = crate::bop_algo::BOPAlgoBuilder::new();
    let receipt =
        super::partition_capture::capture_mapped_partition(&copied, &filler, Some(&builder));
    let PartitionObservation::Incomplete { failure, .. } = receipt else {
        panic!("second copied edge absent from actual DS; expected incomplete binding observation");
    };
    assert!(matches!(
        failure.cause,
        PartitionReadCause::MissingCopiedSource(_)
    ));
    assert_eq!(failure.parents.len(), 1);
    assert_eq!(failure.parents[0].source, RegionSourceId(2));
}

#[test]
fn initialized_filler_without_builder_retains_actual_source_count_but_no_image_claim() {
    let edges = crossing_partition_edges();
    let copied = copied_partition_inputs(&edges);
    let mut filler = crate::bop_algo::BOPAlgoPaveFiller::new();
    filler.set_arguments(&copied.arguments);
    filler.initialize().unwrap();
    let receipt = super::partition_capture::capture_mapped_partition(&copied, &filler, None);
    let PartitionObservation::Incomplete { partial, failure } = receipt else {
        panic!("builder is absent; expected typed incomplete image-context observation");
    };
    assert_eq!(partial.source_shape_count, Some(6));
    assert!(partial.sources.is_empty());
    assert!(matches!(
        failure.cause,
        PartitionReadCause::MissingContext("mapped builder is None")
    ));
}

#[test]
fn actual_empty_common_block_retains_native_failure_and_exact_block_position() {
    let edges = crossing_partition_edges();
    let copied = copied_partition_inputs(&edges);
    let mut filler = crate::bop_algo::BOPAlgoPaveFiller::new();
    filler.set_arguments(&copied.arguments);
    filler.perform_internal().unwrap();
    let structure = filler.data_structure_mut().unwrap();
    let index = structure.index(&copied.roots[0].1);
    let block = structure.pave_blocks(index).unwrap()[0].clone();
    let common = crate::bop_ds::common_block_handle(crate::bop_ds::CommonBlock::new());
    structure.set_common_block(&block, &common);
    let builder = crate::bop_algo::BOPAlgoBuilder::new();
    let receipt =
        super::partition_capture::capture_mapped_partition(&copied, &filler, Some(&builder));
    assert_empty_common_block_failure(receipt, index);
}
fn assert_empty_common_block_failure(receipt: PartitionObservation, index: i32) {
    let PartitionObservation::Incomplete { failure, .. } = receipt else {
        panic!("empty actual common block; expected incomplete original First failure");
    };
    assert_eq!(failure.source_index, Some(index));
    assert_eq!(failure.block_position, Some(0));
    assert_eq!(failure.parents[0].source, RegionSourceId(1));
    let PartitionReadCause::Native(cause) = failure.cause else {
        panic!("empty common block First; expected original native-shaped typed Failure");
    };
    assert!(cause.message().contains("expected at least one pave block"));
}

#[test]
fn poisoned_actual_pave_block_is_observational_incomplete_not_mapped_success() {
    let edges = crossing_partition_edges();
    let copied = copied_partition_inputs(&edges);
    let mut filler = crate::bop_algo::BOPAlgoPaveFiller::new();
    filler.set_arguments(&copied.arguments);
    filler.perform_internal().unwrap();
    let structure = filler.data_structure().unwrap();
    let index = structure.index(&copied.roots[0].1);
    let block = structure.pave_blocks(index).unwrap()[0].clone();
    poison_partition_pave(block);
    let Err(failure) = super::partition_blocks::capture_blocks(index, structure) else {
        panic!("poisoned actual pave lock; expected observational incomplete cause");
    };
    assert_eq!(failure.block_position, Some(0));
    assert!(matches!(
        failure.cause,
        PartitionReadCause::PoisonedPaveBlock
    ));
}
fn poison_partition_pave(block: crate::bop_ds::PaveBlockHandle) {
    let poisoned = std::thread::spawn(move || {
        let _guard = block.lock().unwrap();
        panic!("intentional owned-test pave poisoning");
    });
    assert!(poisoned.join().is_err());
}
