// SPDX-License-Identifier: GPL-2.0-only
//! Physical image membership and directed copy identity are separate observations.
use super::occurrence_keys::same_physical_occurrence;
use crate::sketch_region::{OwnedPartitionSnapshot, PartitionOccurrenceIdentity};
use crate::sketch_region_candidates::CandidateImageCopyLink;

pub(crate) struct PartitionImageBinding {
    pub source_index: i32,
    pub image_index: i32,
    pub copy_rows: Vec<usize>,
    pub fault: Option<PartitionImageCause>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PartitionImageCause {
    MissingCopy,
    MissingImageContext,
    EmptyImageMap,
    MissingParent {
        copy_row: usize,
        parent_position: usize,
    },
}
pub(super) fn inspect_image_links(
    snapshot: &OwnedPartitionSnapshot,
    links: &[CandidateImageCopyLink],
) -> Vec<PartitionImageBinding> {
    snapshot
        .sources
        .iter()
        .flat_map(|source| {
            source_images(source)
                .into_iter()
                .map(|(index, key)| image_binding(source, index, key, links))
                .collect::<Vec<_>>()
        })
        .collect()
}
fn source_images(
    source: &crate::sketch_region::SourcePartitionObservation,
) -> Vec<(i32, &PartitionOccurrenceIdentity)> {
    let Some(images) = &source.images else {
        return vec![(source.source_index, &source.occurrence)];
    };
    if images.is_empty() {
        return vec![(source.source_index, &source.occurrence)];
    }
    images
        .iter()
        .map(|image| (image.shape_index, &image.occurrence))
        .collect()
}

fn image_binding(
    source: &crate::sketch_region::SourcePartitionObservation,
    image: i32,
    key: &PartitionOccurrenceIdentity,
    links: &[CandidateImageCopyLink],
) -> PartitionImageBinding {
    let copy_rows: Vec<_> = links
        .iter()
        .enumerate()
        .filter(|(_, link)| same_physical_occurrence(key, &link.original))
        .map(|(row, _)| row)
        .collect();
    let fault = image_link_fault(source, &copy_rows, links);
    PartitionImageBinding {
        source_index: source.source_index,
        image_index: image,
        copy_rows,
        fault,
    }
}
fn image_parent_fault(
    parents: &[crate::sketch_region::OriginalMemberIdentity],
    rows: &[usize],
    links: &[CandidateImageCopyLink],
) -> Option<PartitionImageCause> {
    if rows.is_empty() {
        return Some(PartitionImageCause::MissingCopy);
    }
    rows.iter()
        .find_map(|&row| missing_link_parent(parents, row, &links[row]))
}
fn missing_link_parent(
    parents: &[crate::sketch_region::OriginalMemberIdentity],
    row: usize,
    link: &CandidateImageCopyLink,
) -> Option<PartitionImageCause> {
    parents
        .iter()
        .position(|parent| !link_has_parent(link, parent))
        .map(|parent_position| PartitionImageCause::MissingParent {
            copy_row: row,
            parent_position,
        })
}

fn link_has_parent(
    link: &CandidateImageCopyLink,
    parent: &crate::sketch_region::OriginalMemberIdentity,
) -> bool {
    link.preserved
        .iter()
        .chain(&link.modified)
        .chain(&link.generated)
        .any(|member| member == parent)
}

fn image_context_fault(
    source: &crate::sketch_region::SourcePartitionObservation,
) -> Option<PartitionImageCause> {
    match &source.images {
        None if source.has_reference => Some(PartitionImageCause::MissingImageContext),
        Some(images) if images.is_empty() => Some(PartitionImageCause::EmptyImageMap),
        _ => None,
    }
}

fn image_link_fault(
    source: &crate::sketch_region::SourcePartitionObservation,
    rows: &[usize],
    links: &[CandidateImageCopyLink],
) -> Option<PartitionImageCause> {
    image_context_fault(source).or_else(|| image_parent_fault(&source.parents, rows, links))
}
