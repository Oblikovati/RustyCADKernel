// SPDX-License-Identifier: GPL-2.0-only
use super::super::BezierRayError;
use std::fmt;
/// No complete closed-wire location is available.
#[derive(Clone, Debug, PartialEq)]
pub enum BezierWireError {
    /// A wire needs at least one directed occurrence.
    Empty,
    /// Repeated caller identity would make incidence diagnostics ambiguous.
    DuplicateId {
        /// Repeated caller-owned occurrence ID.
        id: u64,
    },
    /// Consecutive directed endpoints differ in exact coordinates.
    OpenJoin {
        /// Occurrence whose directed endpoint precedes the join.
        from: u64,
        /// Occurrence whose directed start follows the join.
        to: u64,
    },
    /// A segment retained its original typed numerical failure.
    Segment {
        /// Affected occurrence ID.
        id: u64,
        /// Original stage and numerical cause.
        cause: BezierRayError,
    },
    /// Cancellation before an occurrence starts.
    Cancelled {
        /// Next affected occurrence ID.
        id: u64,
    },
    /// The complete winding number cannot fit the result representation.
    WindingOverflow {
        /// Accumulated count before the failed addition.
        partial: i64,
        /// Directed count that does not fit.
        contribution: i64,
    },
}
impl fmt::Display for BezierWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.problem(), self.expectation())
    }
}
impl BezierWireError {
    fn problem(&self) -> String {
        match self {
            Self::Empty => "0 wire segments".into(),
            Self::DuplicateId { id } => format!("duplicate segment ID {id}"),
            Self::OpenJoin { from, to } => format!("open join {from} -> {to}"),
            Self::Segment { id, cause } => format!("wire segment {id}: {cause}"),
            Self::Cancelled { id } => format!("wire cancelled at segment {id}"),
            Self::WindingOverflow {
                partial,
                contribution,
            } => format!("winding {partial} + {contribution}"),
        }
    }
    fn expectation(&self) -> &'static str {
        match self {
            Self::Empty => "; expected at least one closed occurrence",
            Self::DuplicateId { .. } => "; expected unique occurrence IDs",
            Self::OpenJoin { .. } => "; expected exactly equal directed endpoints",
            Self::Segment { .. } => "",
            Self::Cancelled { .. } => "; expected a complete classification",
            Self::WindingOverflow { .. } => "; expected an i64 sum",
        }
    }
}
impl std::error::Error for BezierWireError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Segment { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
