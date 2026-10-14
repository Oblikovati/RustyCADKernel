// SPDX-License-Identifier: GPL-2.0-only
//! Representative edge indices bind actual images, never a guessed parent-only correspondence.
use crate::sketch_region::{OwnedPartitionSnapshot, SourcePartitionObservation};

pub(crate) struct PartitionImageSpanFault {
    pub source_index: i32,
    pub block_position: Option<usize>,
    pub edge_index: i32,
    pub cause: PartitionImageSpanCause,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PartitionImageSpanCause {
    RepresentativeMultiplicity { count: usize },
    ExtraImage,
}
pub(super) fn inspect_image_partitions(
    snapshot: &OwnedPartitionSnapshot,
) -> Vec<PartitionImageSpanFault> {
    snapshot
        .sources
        .iter()
        .flat_map(source_image_faults)
        .collect()
}
fn source_image_faults(source: &SourcePartitionObservation) -> Vec<PartitionImageSpanFault> {
    let Some(images) = &source.images else {
        return Vec::new();
    };
    let mut faults = representative_image_faults(source, images);
    faults.extend(extra_image_faults(source, images));
    faults
}
fn representative_image_faults(
    source: &SourcePartitionObservation,
    images: &[crate::sketch_region::PartitionImageObservation],
) -> Vec<PartitionImageSpanFault> {
    source
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(position, block)| {
            let count = images
                .iter()
                .filter(|image| image.shape_index == block.real_edge)
                .count();
            (count != 1).then(|| {
                representative_fault(source.source_index, position, block.real_edge, count)
            })
        })
        .collect()
}
fn extra_image_faults(
    source: &SourcePartitionObservation,
    images: &[crate::sketch_region::PartitionImageObservation],
) -> Vec<PartitionImageSpanFault> {
    images
        .iter()
        .filter(|image| {
            !source
                .blocks
                .iter()
                .any(|block| block.real_edge == image.shape_index)
        })
        .map(|image| extra_image_fault(source.source_index, image.shape_index))
        .collect()
}

fn representative_fault(
    source_index: i32,
    position: usize,
    edge_index: i32,
    count: usize,
) -> PartitionImageSpanFault {
    PartitionImageSpanFault {
        source_index,
        block_position: Some(position),
        edge_index,
        cause: PartitionImageSpanCause::RepresentativeMultiplicity { count },
    }
}
fn extra_image_fault(source_index: i32, edge_index: i32) -> PartitionImageSpanFault {
    PartitionImageSpanFault {
        source_index,
        block_position: None,
        edge_index,
        cause: PartitionImageSpanCause::ExtraImage,
    }
}
