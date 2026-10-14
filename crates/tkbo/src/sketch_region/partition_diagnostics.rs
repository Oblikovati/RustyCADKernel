// SPDX-License-Identifier: GPL-2.0-only
//! Access failures remain typed and attributable without changing the mapped result.
use super::partition_types::*;
pub(super) fn missing_context(context: &'static str) -> PartitionReadFailure {
    read_failure(None, None, PartitionReadCause::MissingContext(context))
}
pub(super) fn native_failure(
    index: i32,
    cause: rustycad_tkernel::standard::Failure,
) -> PartitionReadFailure {
    read_failure(
        Some(index),
        None,
        PartitionReadCause::Native(Box::new(cause)),
    )
}
pub(super) fn read_failure(
    source_index: Option<i32>,
    block_position: Option<usize>,
    cause: PartitionReadCause,
) -> PartitionReadFailure {
    PartitionReadFailure {
        source_index,
        block_position,
        parents: Vec::new(),
        cause,
    }
}
