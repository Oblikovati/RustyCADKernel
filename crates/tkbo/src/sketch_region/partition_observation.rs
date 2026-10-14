// SPDX-License-Identifier: GPL-2.0-only
//! Opt-in observations do not infer a partition certificate from mapped completion.
use super::outcome::{ArrangementPhase, ArrangementSource};
use super::partition_types::{ObservedPartitionArrangement, PartitionObservation};

/// Preserve the actual arrangement result alongside an owned native partition observation.
/// Example: `observe_owned_partition(&sources, &mut |_| false)`; no region certificate is implied.
pub(crate) fn observe_owned_partition(
    sources: &[ArrangementSource<'_>],
    cancelled: &mut dyn FnMut(ArrangementPhase) -> bool,
) -> ObservedPartitionArrangement {
    observe_with_capture(sources, cancelled, None)
}
/// Snapshot source descriptors through the project-owned capture boundary.
/// Example: `observe_partition_space(&sources, &mut |_|false, capture_space_representation)`.
pub(crate) fn observe_partition_space(
    sources: &[ArrangementSource<'_>],
    cancelled: &mut dyn FnMut(ArrangementPhase) -> bool,
    capture: super::partition_types::PartitionSpaceCapture,
) -> ObservedPartitionArrangement {
    observe_with_capture(sources, cancelled, Some(capture))
}
fn observe_with_capture(
    sources: &[ArrangementSource<'_>],
    cancelled: &mut dyn FnMut(ArrangementPhase) -> bool,
    space_capture: Option<super::partition_types::PartitionSpaceCapture>,
) -> ObservedPartitionArrangement {
    let mut partition = PartitionObservation::NotReached;
    let completion = super::arrangement::arrange_with_executor(
        sources,
        cancelled,
        &mut super::arrangement::MappedArrangementExecutor {
            observation: Some(&mut partition),
            space_capture,
        },
    );
    ObservedPartitionArrangement {
        completion,
        partition,
    }
}
