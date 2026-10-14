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
    map_two_loop_original_parents_after(request, parent_slices, 0)
}
/// Continue one structural budget without changing requested options; `map_two_loop_original_parents_after(request, rows, reserved)`.
pub(in crate::sketch_region_prism) fn map_two_loop_original_parents_after<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    parent_slices: &[&[OriginalMemberIdentity]],
    reserved_units: usize,
) -> Result<TwoLoopMappedParents, Box<TwoLoopRegionFailure<'a>>> {
    let mut progress = CallbackFreeParentMap;
    let mut work = TwoLoopWork::new(
        request.identities.operation,
        request.requested,
        &mut progress,
    );
    work.stage = TwoLoopRegionStage::Membership;
    admit_mapping(
        &mut work,
        request.sources.len(),
        parent_slices.len(),
        reserved_units,
    )?;
    let originals = member_parents::originals(request, &mut work)?;
    map_rows(parent_slices, &originals, &mut work)
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

fn admit_mapping<'a>(
    work: &mut TwoLoopWork<'_>,
    source_count: usize,
    row_count: usize,
    reserved_units: usize,
) -> Result<(), Box<TwoLoopRegionFailure<'a>>> {
    work.count("two admitted original parent sources", source_count, 2)?;
    work.units(
        "previously retained operation structural units",
        reserved_units,
    )?;
    work.units("original parent mapper directed roster", 6)?;
    work.units("original parent mapper outer rows", row_count)?;
    Ok(())
}

fn map_rows<'a>(
    parent_slices: &[&[OriginalMemberIdentity]],
    originals: &[member_parents::OriginalUseBinding],
    work: &mut TwoLoopWork<'_>,
) -> Result<TwoLoopMappedParents, Box<TwoLoopRegionFailure<'a>>> {
    let mut rows = Vec::new();
    for parents in parent_slices {
        rows.push(member_parents::parents(parents, originals, work)?);
    }
    Ok(TwoLoopMappedParents {
        rows,
        retained_units: work.retained_units,
    })
}
