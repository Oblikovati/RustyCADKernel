// SPDX-License-Identifier: GPL-2.0-only
//! Reuse exact native-role conversion without retaining raw lookup identities in owned proof views.
use super::{
    failure::TwoLoopRegionFailure, member_parents, member_types::TwoLoopOriginalUse, request::*,
    work::TwoLoopWork,
};
use crate::sketch_region::OriginalMemberIdentity;
pub(in crate::sketch_region_prism) struct TwoLoopMappedParents {
    pub rows: Vec<Vec<TwoLoopOriginalUse>>,
    pub retained_units: usize,
}
/// Map a batch with one cumulative budget; `map_two_loop_original_parents(request, &slices)`.
/// The caller must rebind its region before and after use; this scalar mapping grants no region authority.
pub(in crate::sketch_region_prism) fn map_two_loop_original_parents<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    parent_slices: &[&[OriginalMemberIdentity]],
) -> Result<TwoLoopMappedParents, Box<TwoLoopRegionFailure<'a>>> {
    let mut progress = CallbackFreeParentMap;
    let mut work = TwoLoopWork::new(
        request.identities.operation,
        request.requested,
        &mut progress,
    );
    work.stage = TwoLoopRegionStage::Membership;
    work.count("two admitted original parent sources", request.sources.len(), 2)?;
    work.units("original parent mapper directed roster", 6)?;
    work.units("original parent mapper outer rows", parent_slices.len())?;
    let originals = member_parents::originals(request, &mut work)?;
    let mut rows = Vec::new();
    for parents in parent_slices {
        rows.push(member_parents::parents(parents, &originals, &mut work)?);
    }
    Ok(TwoLoopMappedParents {
        rows,
        retained_units: work.retained_units,
    })
}
struct CallbackFreeParentMap;
impl TwoLoopRegionProgress for CallbackFreeParentMap {
    fn checkpoint(
        &mut self,
        _: TwoLoopRegionStage,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok(false)
    }
}
