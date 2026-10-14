// SPDX-License-Identifier: GPL-2.0-only
//! Historical scalar contexts retain actual copier queries; indices are lookup operands only.
use super::{PartitionOccurrenceIdentity, RegionContextFailure, RegionDirectedContext};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RegionObservedContext {
    pub occurrence: PartitionOccurrenceIdentity,
    pub ancestors: Vec<PartitionOccurrenceIdentity>,
    pub child_slots: Vec<usize>,
    pub kind: ShapeEnum,
}
#[derive(Clone, Debug)]
pub(crate) enum RegionContextReadFailure {
    Native(Arc<Failure>),
    RepeatedAncestor(PartitionOccurrenceIdentity),
}
#[derive(Clone, Debug)]
pub(crate) enum RegionContextObservation {
    NotReached,
    Captured(Vec<RegionObservedContext>),
    Incomplete(RegionContextReadFailure),
}
#[derive(Clone, Debug)]
pub(crate) struct RegionContextCopyQuery {
    pub source: RegionObservedContext,
    pub images: Result<Vec<PartitionOccurrenceIdentity>, Arc<Failure>>,
}
#[derive(Clone, Debug)]
pub(crate) struct RegionCopyContexts {
    pub source: Vec<RegionObservedContext>,
    pub copied: Vec<RegionObservedContext>,
    pub queries: Vec<RegionContextCopyQuery>,
}
#[derive(Clone, Debug)]
pub(crate) enum RegionCopyContextObservation {
    NotReached,
    Captured(RegionCopyContexts),
    Incomplete(RegionContextReadFailure),
}
/// Snapshot strict cumulative contexts without handles; `observe_region_contexts(&root)`.
pub(crate) fn observe_region_contexts(root: &Shape) -> RegionContextObservation {
    match captured_contexts(root) {
        Ok(rows) => RegionContextObservation::Captured(rows),
        Err(cause) => RegionContextObservation::Incomplete(cause),
    }
}
/// Observe actual Modified results while the copier lives; `observe_copy_contexts(&original,&copy,&mut copier)`.
/// Query errors are retained independently and cannot turn missing observations into empty results.
pub(crate) fn observe_copy_contexts(
    original: &Shape,
    copied: &Shape,
    copier: &mut BRepBuilderAPICopy,
) -> RegionCopyContextObservation {
    match capture_copy_contexts(original, copied, copier) {
        Ok(rows) => RegionCopyContextObservation::Captured(rows),
        Err(cause) => RegionCopyContextObservation::Incomplete(cause),
    }
}
fn capture_copy_contexts(
    original: &Shape,
    copied: &Shape,
    copier: &mut BRepBuilderAPICopy,
) -> Result<RegionCopyContexts, RegionContextReadFailure> {
    let live = super::strict_region_contexts(original).map_err(context_failure)?;
    let source = live
        .iter()
        .map(snapshot_context)
        .collect::<Result<Vec<_>, _>>()?;
    let copied = captured_contexts(copied)?;
    let queries = live
        .iter()
        .zip(&source)
        .map(|(live, source)| observed_modified(copier, live, source))
        .collect();
    Ok(RegionCopyContexts {
        source,
        copied,
        queries,
    })
}
fn captured_contexts(root: &Shape) -> Result<Vec<RegionObservedContext>, RegionContextReadFailure> {
    super::strict_region_contexts(root)
        .map_err(context_failure)?
        .iter()
        .map(snapshot_context)
        .collect()
}
fn snapshot_context(
    context: &RegionDirectedContext,
) -> Result<RegionObservedContext, RegionContextReadFailure> {
    Ok(RegionObservedContext {
        occurrence: PartitionOccurrenceIdentity::capture(&context.occurrence),
        ancestors: context
            .ancestors
            .iter()
            .map(PartitionOccurrenceIdentity::capture)
            .collect(),
        child_slots: context.child_slots.clone(),
        kind: context
            .occurrence
            .shape_type()
            .map_err(|cause| RegionContextReadFailure::Native(Arc::new(cause)))?,
    })
}
fn observed_modified(
    copier: &mut BRepBuilderAPICopy,
    live: &RegionDirectedContext,
    source: &RegionObservedContext,
) -> RegionContextCopyQuery {
    let images = copier
        .modified(&live.occurrence)
        .map(|images| {
            images
                .iter()
                .map(PartitionOccurrenceIdentity::capture)
                .collect()
        })
        .map_err(Arc::new);
    RegionContextCopyQuery {
        source: source.clone(),
        images,
    }
}
fn context_failure(cause: RegionContextFailure) -> RegionContextReadFailure {
    match cause {
        RegionContextFailure::Native(failure) => {
            RegionContextReadFailure::Native(Arc::new(failure))
        }
        RegionContextFailure::RepeatedAncestor { member } => {
            RegionContextReadFailure::RepeatedAncestor(PartitionOccurrenceIdentity::capture(
                &member,
            ))
        }
    }
}
