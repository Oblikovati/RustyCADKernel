// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismReservationInapplicable {
    PartitionHasNoGeneratedRegions,
    BoundedSelectionExcludesComplementRoles,
    ComplementExcludesBoundedRoles,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismReservationStatus {
    /// Counts named field applications, including repeated physical members in different uses.
    Applied {
        named_fields: usize,
    },
    Inapplicable(PrismReservationInapplicable),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrismReservationRow {
    pub slot: SemanticNameSlot,
    pub status: PrismReservationStatus,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrismReservationCause {
    ForeignMap,
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    UnknownToken {
        actual: u64,
    },
    EmptyUse,
    MissingApplication {
        slot: SemanticNameSlot,
    },
    InapplicableApplied {
        slot: SemanticNameSlot,
        actual: usize,
    },
}
/// Coverage refusal is incomplete, with the original caller operation retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrismReservationFailure {
    pub operation: CallerOperationId,
    pub cause: PrismReservationCause,
}
pub(super) type ReservationCheck<T> = Result<T, PrismReservationCause>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrismReservationCoverage {
    rows: Vec<PrismReservationRow>,
    replay: Vec<u8>,
    map_range: std::ops::Range<usize>,
}
impl PrismReservationCoverage {
    pub(super) fn new(names: &ResolvedPrismNames, rows: Vec<PrismReservationRow>) -> Self {
        let (replay, map_range) = replay::encode(names, &rows);
        Self {
            rows,
            replay,
            map_range,
        }
    }
    /// The entire dictionary is classified; `coverage.rows()` includes explicit unused reservations.
    pub(crate) fn rows(&self) -> &[PrismReservationRow] {
        &self.rows
    }
    /// Bind every caller name and accepted child byte; `coverage.binds(&names)` is exact.
    pub(crate) fn binds(&self, names: &ResolvedPrismNames) -> bool {
        self.replay[self.map_range.clone()] == *names.replay_bytes()
    }
    /// Own full map and deterministic applicability rows; `coverage.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
