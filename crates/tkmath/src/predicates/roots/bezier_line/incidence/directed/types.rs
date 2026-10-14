// SPDX-License-Identifier: GPL-2.0-only
use super::super::{BezierIncidenceQuery, BezierIncidenceReport};

/// Explicit opt-in to source-bound directed positions; version 1 requires endpoint signs.
#[derive(Clone, Copy, Debug)]
pub struct DirectedBezierIncidenceQuery<'a> {
    /// Original input and construction policy; Support still constructs every support root.
    pub incidence: BezierIncidenceQuery<'a>,
    /// Directed behavior version; only 1 is admitted.
    pub version: u32,
}
impl<'a> DirectedBezierIncidenceQuery<'a> {
    /// Opt into endpoint admission for both domains; example: `DirectedBezierIncidenceQuery::v1(query)`.
    pub fn v1(incidence: BezierIncidenceQuery<'a>) -> Self {
        Self {
            incidence,
            version: 1,
        }
    }
}
/// Exact position relative to the directed line endpoints, independent of constructed rounding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectedBezierPosition {
    /// Strictly behind the supplied origin along B-A.
    BeforeOrigin,
    /// Exactly equal to the supplied origin.
    AtOrigin,
    /// Strictly between the supplied endpoints.
    Interior,
    /// Exactly equal to the supplied terminal endpoint.
    AtTerminal,
    /// Strictly beyond the supplied terminal endpoint along B-A.
    BeyondTerminal,
}
/// Complete owned endpoint evidence sharing one support isolation and world construction.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectedBezierIncidenceReport {
    pub(super) version: u32,
    pub(super) incidence: BezierIncidenceReport,
    pub(super) positions: Vec<DirectedBezierPosition>,
}
impl DirectedBezierIncidenceReport {
    /// Read the selected behavior version; example: `report.version()`.
    pub fn version(&self) -> u32 {
        self.version
    }
    /// Read owned source, complete isolation and selected contacts; example: `report.incidence()`.
    pub fn incidence(&self) -> &BezierIncidenceReport {
        &self.incidence
    }
    /// Read one position per support root, in parameter order; example: `report.positions()[contact.support_root_index()]`.
    pub fn positions(&self) -> &[DirectedBezierPosition] {
        &self.positions
    }
}
