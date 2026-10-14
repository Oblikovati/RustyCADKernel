// SPDX-License-Identifier: GPL-2.0-only
//! Retain actual per-member queries before the mapped builder retires; no legacy decision changes.
use super::{OriginalMemberIdentity, PartitionOccurrenceIdentity, copy_inputs::CopiedSources};
use crate::bop_algo::BOPAlgoBuilder;
use rustycad_tkbrep::{brep_tools::History, topods::Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct PartitionMemberQuery {
    pub original: OriginalMemberIdentity,
    pub copied: PartitionOccurrenceIdentity,
    pub kind: Result<ShapeEnum, Arc<Failure>>,
    pub supported: Result<bool, Arc<Failure>>,
    pub modified: Result<Vec<PartitionOccurrenceIdentity>, Arc<Failure>>,
    pub generated: Result<Vec<PartitionOccurrenceIdentity>, Arc<Failure>>,
    pub deleted: Result<bool, Arc<Failure>>,
}
#[derive(Clone, Debug)]
pub(crate) enum PartitionMemberQueries {
    NotReached,
    Captured {
        history_requested: bool,
        history_available: bool,
        rows: Vec<PartitionMemberQuery>,
    },
}
pub(super) fn capture_member_queries(
    copied: &CopiedSources,
    builder: &BOPAlgoBuilder,
) -> PartitionMemberQueries {
    let rows = copied
        .members
        .iter()
        .map(|(original, member)| member_query(original, member, builder))
        .collect();
    PartitionMemberQueries::Captured {
        history_requested: builder.has_history(),
        history_available: builder.has_filled_history(),
        rows,
    }
}
fn member_query(
    original: &OriginalMemberIdentity,
    member: &Shape,
    builder: &BOPAlgoBuilder,
) -> PartitionMemberQuery {
    PartitionMemberQuery {
        original: original.clone(),
        copied: PartitionOccurrenceIdentity::capture(member),
        kind: member.shape_type().map_err(Arc::new),
        supported: History::is_supported_type(member).map_err(Arc::new),
        modified: query_images(builder.modified(member)),
        generated: query_images(builder.generated(member)),
        deleted: builder.is_deleted(member).map_err(Arc::new),
    }
}
fn query_images(
    result: Result<Vec<Shape>, Failure>,
) -> Result<Vec<PartitionOccurrenceIdentity>, Arc<Failure>> {
    result
        .map(|images| {
            images
                .iter()
                .map(PartitionOccurrenceIdentity::capture)
                .collect()
        })
        .map_err(Arc::new)
}
