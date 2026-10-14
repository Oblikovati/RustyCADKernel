// SPDX-License-Identifier: GPL-2.0-only
//! Capture before filler retirement; no mutable DS/pave aliases leave this module.
use super::copy_inputs::CopiedSources;
use super::partition_blocks::capture_blocks;
use super::partition_diagnostics::{missing_context, native_failure, read_failure};
use super::partition_types::*;
use crate::{
    bop_algo::{BOPAlgoBuilder, BOPAlgoPaveFiller},
    bop_ds::DataStructure,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_mapped_partition(
    copied: &CopiedSources,
    filler: &BOPAlgoPaveFiller,
    builder: Option<&BOPAlgoBuilder>,
) -> PartitionObservation {
    capture_partition_with_space(copied, filler, builder, None)
}
pub(super) fn capture_partition_with_space(
    copied: &CopiedSources,
    filler: &BOPAlgoPaveFiller,
    builder: Option<&BOPAlgoBuilder>,
    space: Option<PartitionSpaceCapture>,
) -> PartitionObservation {
    let mut partial = OwnedPartitionSnapshot {
        source_shape_count: None,
        sources: Vec::new(),
        vertices: Vec::new(),
    };
    match capture_context(copied, filler, builder, &mut partial, space) {
        Ok(()) => PartitionObservation::Captured(partial),
        Err(failure) => PartitionObservation::Incomplete { partial, failure },
    }
}
fn capture_context(
    copied: &CopiedSources,
    filler: &BOPAlgoPaveFiller,
    builder: Option<&BOPAlgoBuilder>,
    partial: &mut OwnedPartitionSnapshot,
    space: Option<PartitionSpaceCapture>,
) -> Result<(), PartitionReadFailure> {
    let structure = filler
        .data_structure()
        .ok_or_else(|| missing_context("filler DS is None"))?;
    partial.source_shape_count = Some(structure.nb_source_shapes());
    let builder = builder.ok_or_else(|| missing_context("mapped builder is None"))?;
    capture_members(copied, structure)?;
    capture_sources(copied, structure, builder, partial, space)?;
    super::partition_vertices::capture_vertices(
        copied,
        structure,
        partial,
        PartitionOccurrenceIdentity::capture,
    )
}
fn capture_sources(
    copied: &CopiedSources,
    structure: &DataStructure,
    builder: &BOPAlgoBuilder,
    partial: &mut OwnedPartitionSnapshot,
    space: Option<PartitionSpaceCapture>,
) -> Result<(), PartitionReadFailure> {
    for index in 0..structure.nb_source_shapes() {
        let kind = source_kind(index, structure)?;
        if kind != ShapeEnum::Edge {
            continue;
        }
        partial
            .sources
            .push(capture_source(index, copied, structure, builder, space)?);
    }
    Ok(())
}

fn capture_source(
    index: i32,
    copied: &CopiedSources,
    structure: &DataStructure,
    builder: &BOPAlgoBuilder,
    space: Option<PartitionSpaceCapture>,
) -> Result<SourcePartitionObservation, PartitionReadFailure> {
    let parents = source_parents(index, copied, structure)?;
    source_fields(index, structure, builder, space)
        .map(|mut source| {
            source.parents = parents.clone();
            source
        })
        .map_err(|mut failure| {
            failure.parents = parents;
            failure
        })
}
fn source_fields(
    index: i32,
    structure: &DataStructure,
    builder: &BOPAlgoBuilder,
    space: Option<PartitionSpaceCapture>,
) -> Result<SourcePartitionObservation, PartitionReadFailure> {
    let shape = structure
        .shape(index)
        .map_err(|cause| native_failure(index, cause))?;
    let mut source = source_shell(index, shape, structure)?;
    source.space = capture_source_space(index, shape, space)?;
    source.source_vertices = source_children(index, shape, structure)?;
    source.blocks = capture_blocks(index, structure)?;
    source.images = capture_images(index, shape, structure, builder)?;
    Ok(source)
}
fn source_shell(
    index: i32,
    shape: &Shape,
    structure: &DataStructure,
) -> Result<SourcePartitionObservation, PartitionReadFailure> {
    Ok(SourcePartitionObservation {
        source_index: index,
        parents: Vec::new(),
        occurrence: PartitionOccurrenceIdentity::capture(shape),
        range: source_range(index, shape)?,
        space: None,
        has_reference: source_has_reference(index, structure)?,
        source_vertices: Vec::new(),
        blocks: Vec::new(),
        images: None,
    })
}

fn source_range(index: i32, shape: &Shape) -> Result<[f64; 2], PartitionReadFailure> {
    let typed = edge(shape).map_err(|cause| native_failure(index, cause))?;
    let range = BRepTool::range(&typed).map_err(|cause| native_failure(index, cause))?;
    Ok([range.0, range.1])
}

fn source_parents(
    index: i32,
    copied: &CopiedSources,
    structure: &DataStructure,
) -> Result<Vec<super::outcome::OriginalMemberIdentity>, PartitionReadFailure> {
    let parents: Vec<_> = copied
        .members
        .iter()
        .filter(|(_, shape)| structure.index(shape) == index)
        .map(|(key, _)| key.clone())
        .collect();
    if parents.is_empty() {
        return Err(read_failure(
            Some(index),
            None,
            PartitionReadCause::MissingSourceParents(index),
        ));
    }
    Ok(parents)
}
fn capture_images(
    index: i32,
    shape: &Shape,
    structure: &DataStructure,
    builder: &BOPAlgoBuilder,
) -> Result<Option<Vec<PartitionImageObservation>>, PartitionReadFailure> {
    let Some(images) = builder.images().seek(shape) else {
        return Ok(None);
    };
    images
        .iter()
        .map(|image| image_observation(index, image, structure))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
fn image_index(
    source: i32,
    image: &Shape,
    structure: &DataStructure,
) -> Result<i32, PartitionReadFailure> {
    let index = structure.index(image);
    if index < 0 {
        return Err(read_failure(
            Some(source),
            None,
            PartitionReadCause::MissingImageIndex(index),
        ));
    }
    Ok(index)
}

fn source_has_reference(
    index: i32,
    structure: &DataStructure,
) -> Result<bool, PartitionReadFailure> {
    Ok(structure
        .shape_info(index)
        .map_err(|cause| native_failure(index, cause))?
        .has_reference())
}

fn image_observation(
    source: i32,
    image: &Shape,
    structure: &DataStructure,
) -> Result<PartitionImageObservation, PartitionReadFailure> {
    Ok(PartitionImageObservation {
        shape_index: image_index(source, image, structure)?,
        occurrence: PartitionOccurrenceIdentity::capture(image),
    })
}

fn capture_members(
    copied: &CopiedSources,
    structure: &DataStructure,
) -> Result<(), PartitionReadFailure> {
    for (position, (parent, member)) in copied.members.iter().enumerate() {
        let kind = member
            .shape_type()
            .map_err(|cause| member_failure(parent, cause))?;
        if kind != ShapeEnum::Edge || structure.index(member) >= 0 {
            continue;
        }
        return Err(missing_member_failure(parent, position));
    }
    Ok(())
}
fn member_failure(
    parent: &super::outcome::OriginalMemberIdentity,
    cause: rustycad_tkernel::standard::Failure,
) -> PartitionReadFailure {
    let mut failure = read_failure(None, None, PartitionReadCause::Native(Box::new(cause)));
    failure.parents.push(parent.clone());
    failure
}

fn missing_member_failure(
    parent: &super::outcome::OriginalMemberIdentity,
    position: usize,
) -> PartitionReadFailure {
    let mut failure = read_failure(
        None,
        None,
        PartitionReadCause::MissingCopiedSource(position),
    );
    failure.parents.push(parent.clone());
    failure
}

fn capture_source_space(
    index: i32,
    shape: &Shape,
    capture: Option<PartitionSpaceCapture>,
) -> Result<Option<PartitionSpaceRead>, PartitionReadFailure> {
    let Some(capture) = capture else {
        return Ok(None);
    };
    let typed = edge(shape).map_err(|cause| native_failure(index, cause))?;
    Ok(Some(capture(&typed).map_err(std::sync::Arc::new)))
}

fn source_kind(index: i32, structure: &DataStructure) -> Result<ShapeEnum, PartitionReadFailure> {
    Ok(structure
        .shape_info(index)
        .map_err(|cause| native_failure(index, cause))?
        .shape_type())
}

fn source_children(
    index: i32,
    shape: &Shape,
    structure: &DataStructure,
) -> Result<Vec<SourceVertexObservation>, PartitionReadFailure> {
    super::partition_source_vertices::capture_source_vertices(
        index,
        shape,
        structure,
        PartitionOccurrenceIdentity::capture,
    )
}
