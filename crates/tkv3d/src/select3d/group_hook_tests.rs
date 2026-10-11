//! Native protected qualified calls and actual application virtual dispatch share one group subobject.
use super::group_hook_fake::GroupHookFake;
use super::group_oracle_geometry::group_children;
use super::group_oracle_snapshot::group_snapshot;
use super::quadric_oracle_snapshot::quadric_seed;
use super::quadric_oracle_volume::quadric_volume;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup, Select3DSensitiveSet};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::{Failure, Handle};
struct GroupHookScenario {
    source: Handle<GroupHookFake>,
    children: Vec<Handle<dyn Select3DSensitiveEntity>>,
    ordinal: i32,
}
impl GroupHookScenario {
    fn new(ordinal: i32) -> Self {
        let owner = SelectMgrEntityOwner::new(13).cast();
        let children = group_children(2, &owner);
        let core = Select3DSensitiveGroup::new(&owner, ordinal / 32 % 4 / 2 != 0);
        core.add_all(&children);
        core.set_check_overlap_all(ordinal / 32 % 4 % 2 != 0);
        core.swap(0, 2);
        let source = Handle::new(GroupHookFake::new(core, ordinal / 8 % 4));
        Self {
            source,
            children,
            ordinal,
        }
    }
    fn operate(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if self.ordinal / 2 % 4 == 0 {
            let entity = self.source.clone().cast();
            return super::select3d_group_downcast(&entity).matches(volume, pick);
        }
        self.qualified_call(volume, pick)
    }
    fn qualified_call(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let source = self.source.get().unwrap();
        let inside = self.ordinal % 2 != 0;
        match self.ordinal / 2 % 4 {
            1 => super::select3d_group_overlaps_element(source, pick, volume, 0, inside),
            2 => super::select3d_group_element_is_inside(source, volume, 0, inside),
            _ => {
                pick.set_dist_to_geom_center(super::select3d_group_distance_to_cog(
                    source, volume,
                )?);
                Ok(true)
            }
        }
    }
    fn replay(&self, expected: &mut std::str::Lines<'_>) {
        let volume = quadric_volume(self.ordinal / 128, false, false, false);
        let mut pick = quadric_seed();
        let outcome = self.operate(&volume, &mut pick);
        let events = self.source.trace.read();
        let mut trace = vec![f64::from(self.ordinal), 0., events.len() as f64];
        trace.extend(events.iter().map(|event| f64::from(*event)));
        drop(events);
        compare_oracle_snapshot(&trace, expected.next().unwrap());
        let columns = group_snapshot(
            &self.source.core,
            &self.children,
            self.ordinal,
            0,
            outcome.is_err(),
            outcome.map(i32::from).unwrap_or(-1),
            &pick,
        );
        compare_oracle_snapshot(&columns, expected.next().unwrap());
    }
}
#[test]
fn native_group_protected_hooks_and_application_dispatch_preserve_callbacks_failures_and_outputs() {
    let mut expected = super::group_hook_fixtures::GROUP_HOOK_NATIVE.lines();
    for ordinal in 0..1152 {
        GroupHookScenario::new(ordinal).replay(&mut expected);
    }
    assert!(expected.next().is_none());
}
