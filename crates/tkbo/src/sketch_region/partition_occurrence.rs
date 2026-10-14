// SPDX-License-Identifier: GPL-2.0-only
use super::PartitionOccurrenceIdentity;
use rustycad_tkbrep::topods::Shape;
impl PartitionOccurrenceIdentity {
    /// Capture the actual cumulative copied occurrence, not its parent or a recovered pointer.
    /// Example: `PartitionOccurrenceIdentity::capture(&actual_copied_edge)`.
    pub(crate) fn capture(shape: &Shape) -> Self {
        Self {
            topology_address: shape.tshape().object_address().addr(),
            location: shape.location().clone(),
            orientation: shape.orientation(),
        }
    }
}
