// SPDX-License-Identifier: GPL-2.0-only
use super::super::{BezierIncidenceCompletion, BezierIncidenceFailure, BezierIncidenceSource};
use std::{cmp::Ordering, fmt};

/// Typed noncompletion; no endpoint evidence is published as a partial success.
#[derive(Clone, Debug, PartialEq)]
pub enum DirectedBezierIncidenceCause {
    /// Unsupported directed behavior version; expected 1.
    Version(u32),
    /// Original projection, admission, isolation, cancellation or construction failure.
    Incidence(BezierIncidenceFailure),
    /// Sign vectors do not cover the same complete isolation.
    Alignment {
        /// Complete support root count.
        roots: usize,
        /// Origin sign count.
        origins: usize,
        /// Terminal sign count.
        terminals: usize,
    },
    /// Endpoint signs contradict the strictly positive squared line length.
    EndpointOrder {
        /// Index into the shared complete support isolation.
        support_root_index: usize,
        /// Exact origin comparison.
        origin: Ordering,
        /// Exact terminal comparison.
        terminal: Ordering,
    },
}
/// Owned request provenance and typed cause for a directed query that did not complete.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectedBezierIncidenceFailure {
    pub(super) version: u32,
    pub(super) source: Box<BezierIncidenceSource>,
    pub(super) cause: DirectedBezierIncidenceCause,
}
impl DirectedBezierIncidenceFailure {
    /// Read the requested version, including rejected versions; example: `failure.version()`.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Read retained source coefficients and occurrence IDs; example: `failure.source().curve_id()`.
    pub fn source(&self) -> &BezierIncidenceSource {
        &self.source
    }
    /// Read the original typed cause; example: `failure.cause()`.
    pub fn cause(&self) -> &DirectedBezierIncidenceCause {
        &self.cause
    }
    /// Distinguish invalid, declined, cancelled and incomplete; example: `failure.completion()`.
    pub fn completion(&self) -> BezierIncidenceCompletion {
        match &self.cause {
            DirectedBezierIncidenceCause::Version(_) => BezierIncidenceCompletion::Invalid,
            DirectedBezierIncidenceCause::Incidence(failure) => failure.completion(),
            _ => BezierIncidenceCompletion::Incomplete,
        }
    }
}
impl fmt::Display for DirectedBezierIncidenceCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version(version) => {
                write!(formatter, "directed behavior version {version}; expected 1")
            }
            Self::Incidence(failure) => failure.fmt(formatter),
            Self::Alignment {
                roots,
                origins,
                terminals,
            } => format_alignment(formatter, *roots, *origins, *terminals),
            Self::EndpointOrder {
                support_root_index,
                origin,
                terminal,
            } => format_endpoint_order(formatter, *support_root_index, *origin, *terminal),
        }
    }
}
fn format_alignment(
    formatter: &mut fmt::Formatter<'_>,
    roots: usize,
    origins: usize,
    terminals: usize,
) -> fmt::Result {
    write!(
        formatter,
        "endpoint counts ({origins}, {terminals}); expected ({roots}, {roots}) for complete support isolation"
    )
}
fn format_endpoint_order(
    formatter: &mut fmt::Formatter<'_>,
    index: usize,
    origin: Ordering,
    terminal: Ordering,
) -> fmt::Result {
    write!(
        formatter,
        "root {index}, signs ({origin:?}, {terminal:?}); expected (--), (0-), (+-), (+0), or (++)"
    )
}
impl fmt::Display for DirectedBezierIncidenceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "directed incidence v{}, curve {}, line {}: {}",
            self.version,
            self.source.curve_id(),
            self.source.line_id(),
            self.cause
        )
    }
}
impl std::error::Error for DirectedBezierIncidenceCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Incidence(failure) => Some(failure),
            _ => None,
        }
    }
}
impl std::error::Error for DirectedBezierIncidenceFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
