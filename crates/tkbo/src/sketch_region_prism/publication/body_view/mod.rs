// SPDX-License-Identifier: GPL-2.0-only
//! ADR0047 read adapters; semantic correspondence and complete-body projection remain prerequisites.
mod coverage;
mod error;
mod faces;
mod geometry;
mod incidence;
mod lookup;
mod members;
mod occurrences;
mod operation;
mod references;
mod replay;
mod result;
mod row_views;
mod rows;
pub(crate) use error::*;
pub(crate) use geometry::{PrismBodySpaceCurveView, PrismBodySurfaceView};
pub(crate) use operation::project_named_prism_body;
pub(crate) use result::OwnedNamedPrismBody;
pub(crate) use rows::{
    PrismBodyCoedge, PrismBodyEdgeEndpoints, PrismBodyFace, PrismBodyMember, PrismBodyOccurrence,
    PrismBodyTrim, PrismBodyWireTraversal,
};
#[cfg(test)]
mod actual_operand_tests;
#[cfg(test)]
mod actual_replay_tests;
#[cfg(test)]
mod actual_tests;
#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod face_tests;
#[cfg(test)]
mod geometry_tests;
#[cfg(test)]
mod getter_snapshot;
#[cfg(test)]
mod incidence_tests;
#[cfg(test)]
mod member_tests;
#[cfg(test)]
mod occurrence_tests;
#[cfg(test)]
mod operation_tests;
#[cfg(test)]
mod row_replay_tests;
#[cfg(test)]
mod storage_tests;
#[cfg(test)]
mod test_fixture;
