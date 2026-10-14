// SPDX-License-Identifier: GPL-2.0-only
//! ADR0049: internal policy for certified parameter domains, not caller-supplied Euler weights.
use super::super::{FrozenPrismProjection, FrozenPrismUse};
use super::{execution::TopologyWork, model::*, pairing, types::*};

pub(super) trait FaceDomainPolicy {
    fn check_face_children(
        &self,
        face: &FrozenPrismUse,
        index: &TopologyIndex<'_>,
    ) -> TopologyResult<()>;
    fn check_euler(
        &self,
        source: &FrozenPrismProjection,
        index: &TopologyIndex<'_>,
        work: &mut TopologyWork<'_>,
    ) -> TopologyResult<()>;
}

pub(super) struct LegacySingleJordanDomain;
impl FaceDomainPolicy for LegacySingleJordanDomain {
    fn check_face_children(
        &self,
        face: &FrozenPrismUse,
        _index: &TopologyIndex<'_>,
    ) -> TopologyResult<()> {
        count(
            Some(&face.key),
            face.children.len(),
            1,
            "one shell per solid / one wire per face",
        )
    }
    fn check_euler(
        &self,
        source: &FrozenPrismProjection,
        _index: &TopologyIndex<'_>,
        work: &mut TopologyWork<'_>,
    ) -> TopologyResult<()> {
        // Keep V1's exact callback, witness and arithmetic available for historical replay.
        pairing::euler(source.graph.counts, work)
    }
}
