//! Direct all/check-all selection keeps one reusable child pick and preserves earlier output writes.
use super::select3d_set_matches;
use super::{Select3DSensitiveGroup, Select3DSensitiveSet};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::standard::Failure;
impl Select3DSensitiveGroup {
    pub(super) fn match_group_volume(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // OCCT 3d097a0 invokes selection type twice; each callback may change the subsequent policy.
        let to_match_all = manager.get_active_selection_type() != SelectMgrSelectionType::Point
            && self.must_match_all();
        let to_check_all = manager.get_active_selection_type() != SelectMgrSelectionType::Point
            && self.to_check_overlap_all();
        if !to_match_all && !to_check_all {
            return select3d_set_matches(self, manager, pick_result, false);
        }
        self.matches_all(manager, pick_result, to_match_all, to_check_all)
    }

    /// `SetMatchType`; `group.set_match_type(false)`.
    pub fn set_match_type(&self, must_match_all: bool) {
        // This policy affects only nonpoint selection and never dirties geometry.
        self.state.write().must_match_all = must_match_all;
    }
    /// `MustMatchAll`; `group.must_match_all()`.
    pub fn must_match_all(&self) -> bool {
        // Read after its native selection-type callback, not before both callbacks.
        self.state.read().must_match_all
    }
    /// `ToCheckOverlapAll`; `group.to_check_overlap_all()`.
    pub fn to_check_overlap_all(&self) -> bool {
        // Independent policy observes callback-induced changes before dispatch.
        self.state.read().to_check_overlap_all
    }
    /// `SetCheckOverlapAll`; `group.set_check_overlap_all(true)`.
    pub fn set_check_overlap_all(&self, check_overlap_all: bool) {
        // Checking all children keeps their own detection state coherent on misses.
        self.state.write().to_check_overlap_all = check_overlap_all;
    }
    /// The match of every entity (`Matches` for a box or polyline with `MustMatchAll` or
    /// `ToCheckOverlapAll`), keeping the nearest pick.
    pub(super) fn matches_all(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        result: &mut SelectBasicsPickResult,
        must_all: bool,
        check_all: bool,
    ) -> Result<bool, Failure> {
        let failed = self.check_group_children(volume, result, (must_all, check_all))?;
        if failed {
            return Ok(false);
        }
        result.set_dist_to_geom_center(self.distance_to_cog(volume)?);
        Ok(true)
    }
    fn check_group_children(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        result: &mut SelectBasicsPickResult,
        policy: (bool, bool),
    ) -> Result<bool, Failure> {
        let mut pick = SelectBasicsPickResult::default();
        let mut failed = false;
        for child in self.entities().iter().filter_map(|handle| handle.get()) {
            let matched = merge_group_child_pick(child, volume, &mut pick, result)?;
            failed |= !matched && policy.0;
            if failed && !policy.1 {
                break;
            }
        }
        Ok(failed)
    }
    /// `LastDetectedEntity`; `group.last_detected_entity()`.
    pub fn last_detected_entity(
        &self,
    ) -> rustycad_tkernel::standard::Handle<dyn super::Select3DSensitiveEntity> {
        let index = self.last_detected_entity_index();
        if index == -1 {
            return rustycad_tkernel::standard::Handle::default();
        }
        super::triangulation_failure::native_checked(self.sub_entity_ref(index)).entity()
    }
    /// `LastDetectedEntityIndex`: the one-based index of the detected entity, -1 without.
    /// Example: `group.last_detected_entity_index()`.
    pub fn last_detected_entity_index(&self) -> i32 {
        let detected = self.set.detected_idx();
        if detected == -1 {
            return -1;
        }
        // Valid native detection indices address the independently permuted primitive vector.
        self.state.read().bvh_prim_indexes[detected as usize]
    }
}

fn merge_group_child_pick(
    child: &dyn super::Select3DSensitiveEntity,
    volume: &dyn SelectBasicsSelectingVolumeManager,
    pick: &mut SelectBasicsPickResult,
    result: &mut SelectBasicsPickResult,
) -> Result<bool, Failure> {
    let matched = child.matches(volume, pick)?;
    if matched {
        *result = *SelectBasicsPickResult::min(result, pick);
    }
    Ok(matched)
}
