// SPDX-License-Identifier: GPL-2.0-only
use crate::sketch_region::PartitionOccurrenceIdentity;

pub(super) fn same_physical_occurrence(
    left: &PartitionOccurrenceIdentity,
    right: &PartitionOccurrenceIdentity,
) -> bool {
    // Builder image maps and native copied-parent lookup use IsSame, retaining direction separately.
    left.same_physical(right)
}
