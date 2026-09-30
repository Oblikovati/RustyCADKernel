// Rust translation of the private EdgeEntry, CoEdgeSameRangeEntry, WireEntry and
// ShellEntry of ModelingData/TKBRep/BRepGraph/BRepGraph_CacheDerivedState.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The memoised entries of the derived-state cache: a node entry bound to the OwnGen
//! plus the packed flags of the value.
//!
//! Natively each entry packs its flags in one atomic byte for lock-free reads; here the
//! entries live behind the service's lock, so the byte is plain. `Reset()` is the node
//! entry's reset only: the flags survive it, as natively.
use super::{EdgeGeomStatus, ShellClosureStatus};
use crate::brep_graph::cache::NodeEntry;

// `EdgeEntry::Flags`.
const EDGE_STATUS_MASK: u8 = 0x07;
const EDGE_FLAG_CLOSED: u8 = 1 << 3;
const EDGE_FLAG_COMPUTED: u8 = 1 << 4;

/// `EdgeEntry`: the status and closure of an edge, bound to its OwnGen.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EdgeEntry {
    pub(super) node: NodeEntry,
    packed: u8,
}

impl EdgeEntry {
    /// `GetStatus()`.
    pub(super) fn status(&self) -> EdgeGeomStatus {
        match self.packed & EDGE_STATUS_MASK {
            0 => EdgeGeomStatus::HasCurve3D,
            1 => EdgeGeomStatus::DegenerateOnSurface,
            _ => EdgeGeomStatus::MissingCurve3D,
        }
    }

    /// `IsClosed()`.
    pub(super) fn is_closed(&self) -> bool {
        self.packed & EDGE_FLAG_CLOSED != 0
    }

    /// `IsComputed()`.
    pub(super) fn is_computed(&self) -> bool {
        self.packed & EDGE_FLAG_COMPUTED != 0
    }

    /// `Set(theStatus, theClosed)`: the whole byte is replaced.
    pub(super) fn set(&mut self, status: EdgeGeomStatus, closed: bool) {
        let closed_bit = if closed { EDGE_FLAG_CLOSED } else { 0 };
        self.packed = EDGE_FLAG_COMPUTED | status as u8 | closed_bit;
    }
}

/// `CoEdgeSameRangeEntry::ComputedSameRange`.
pub(super) const COMPUTED_SAME_RANGE: u8 = 1 << 0;
/// `CoEdgeSameRangeEntry::ComputedSameParam`.
pub(super) const COMPUTED_SAME_PARAM: u8 = 1 << 1;
const FLAG_SAME_RANGE: u8 = 1 << 2;
const FLAG_SAME_PARAMETER: u8 = 1 << 3;

/// `CoEdgeSameRangeEntry`: SameRange and SameParameter of a coedge, each computed on
/// demand, bound to the coedge's OwnGen.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CoEdgeSameRangeEntry {
    pub(super) node: NodeEntry,
    pub(super) packed: u8,
}

impl CoEdgeSameRangeEntry {
    /// `SameRange()`.
    pub(super) fn same_range(&self) -> bool {
        self.packed & FLAG_SAME_RANGE != 0
    }

    /// `SameParameter()`.
    pub(super) fn same_parameter(&self) -> bool {
        self.packed & FLAG_SAME_PARAMETER != 0
    }

    /// `Computed()`: the whole byte.
    pub(super) fn computed(&self) -> u8 {
        self.packed
    }

    /// `SetSameRange(theVal)`: ORs the computed bit and a true value in.
    pub(super) fn set_same_range(&mut self, value: bool) {
        self.packed |= COMPUTED_SAME_RANGE | if value { FLAG_SAME_RANGE } else { 0 };
    }

    /// `SetSameParameter(theVal)`: ORs the computed bit and a true value in.
    pub(super) fn set_same_parameter(&mut self, value: bool) {
        self.packed |= COMPUTED_SAME_PARAM | if value { FLAG_SAME_PARAMETER } else { 0 };
    }

    /// The merge of `ensureCoEdgeSameRangeEntry` into a fresh stored entry: the computed
    /// bits this entry adds, with their value bits (two places up), replace the stored
    /// ones; the rest of the stored byte is kept.
    pub(super) fn merge_into(&self, stored: &mut CoEdgeSameRangeEntry) {
        let computed = self.packed & (COMPUTED_SAME_RANGE | COMPUTED_SAME_PARAM);
        let new_bits = computed & !stored.packed;
        if new_bits == 0 {
            return;
        }
        stored.packed = (stored.packed & !((new_bits << 2) | new_bits))
            | (self.packed & (new_bits << 2))
            | new_bits;
    }
}

// `WireEntry::Flags`.
const WIRE_FLAG_CLOSED: u8 = 1 << 0;
const WIRE_FLAG_COMPUTED: u8 = 1 << 1;

/// `WireEntry`: the closure of a wire, bound to its OwnGen.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WireEntry {
    pub(super) node: NodeEntry,
    packed: u8,
}

impl WireEntry {
    /// `IsClosed()`.
    pub(super) fn is_closed(&self) -> bool {
        self.packed & WIRE_FLAG_CLOSED != 0
    }

    /// `IsComputed()`.
    pub(super) fn is_computed(&self) -> bool {
        self.packed & WIRE_FLAG_COMPUTED != 0
    }

    /// `SetClosed(theVal)`: the whole byte is replaced.
    pub(super) fn set_closed(&mut self, value: bool) {
        self.packed = WIRE_FLAG_COMPUTED | if value { WIRE_FLAG_CLOSED } else { 0 };
    }
}

/// `ShellEntry`: the closure status of a shell (`Invalid` until computed), bound to
/// its OwnGen.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ShellEntry {
    pub(super) node: NodeEntry,
    pub(super) status: ShellClosureStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_flags_follow_the_native_layout() {
        let mut edge = EdgeEntry::default();
        assert!(!edge.is_computed() && edge.status() == EdgeGeomStatus::HasCurve3D);
        edge.set(EdgeGeomStatus::DegenerateOnSurface, true);
        assert_eq!(edge.packed, 0x10 | 0x08 | 1);
        assert!(edge.is_closed() && edge.status() == EdgeGeomStatus::DegenerateOnSurface);
        let mut wire = WireEntry::default();
        wire.set_closed(true);
        assert!(wire.is_computed() && wire.is_closed());
        wire.set_closed(false);
        assert!(!wire.is_closed());
    }

    #[test]
    fn co_edge_merge_keeps_the_stored_computed_bits() {
        let mut stored = CoEdgeSameRangeEntry::default();
        stored.set_same_range(true);
        let mut fresh = CoEdgeSameRangeEntry::default();
        fresh.set_same_range(false);
        fresh.set_same_parameter(true);
        fresh.merge_into(&mut stored);
        assert!(stored.same_range() && stored.same_parameter());
        assert_eq!(stored.computed(), 0x0F);
        let unchanged = stored;
        fresh.merge_into(&mut stored);
        assert_eq!(stored.packed, unchanged.packed);
    }
}
