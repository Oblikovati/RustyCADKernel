//! Committed native CPU evidence replays through real Rust selecting volumes without OCCT.
use super::group_oracle_fixtures::GROUP_SELECTION_NATIVE;
use super::group_oracle_geometry::construct_group;
use super::group_oracle_snapshot::snapshot_group_match;
use super::quadric_oracle_volume::quadric_volume;
use super::triangulation_oracle_snapshot::compare_oracle_snapshot;
use super::{Select3DSensitiveEntity, Select3DSensitiveGroup};
use crate::select_mgr::SelectMgrEntityOwner;
use rustycad_tkernel::standard::Handle;
struct GroupSelectionReplay {
    original: Handle<Select3DSensitiveGroup>,
    connected: Handle<Select3DSensitiveGroup>,
    children: Vec<Handle<dyn Select3DSensitiveEntity>>,
    ordinal: i32,
}
impl GroupSelectionReplay {
    fn new(ordinal: i32) -> Self {
        let owner = SelectMgrEntityOwner::new(13).cast();
        let (original, children) = construct_group(ordinal, &owner);
        let connected = original
            .get_connected()
            .unwrap()
            .cast::<Select3DSensitiveGroup>();
        Self {
            original,
            connected,
            children,
            ordinal,
        }
    }
    fn volume_for_step(&self, step: i32) -> crate::select_mgr::SelectMgrSelectingVolumeManager {
        // Projection and clipping are replayed without any native library dependency.
        quadric_volume(
            self.ordinal / 16 % 9,
            self.ordinal / 2 % 2 != 0,
            self.ordinal % 2 != 0,
            step == 1,
        )
    }
    fn replay_step(&self, step: i32, expected: &str) {
        let probe = if step == 3 {
            &self.connected
        } else {
            &self.original
        };
        let volume = self.volume_for_step(step);
        let columns = snapshot_group_match(
            probe.get().unwrap(),
            &self.children,
            self.ordinal,
            step,
            &volume,
        );
        compare_oracle_snapshot(&columns, expected);
    }
}
#[test]
fn native_group_real_volumes_preserve_matching_policies_detection_and_connected_state() {
    let mut expected = GROUP_SELECTION_NATIVE.lines();
    for ordinal in 0..1728 {
        let scenario = GroupSelectionReplay::new(ordinal);
        for step in 0..4 {
            scenario.replay_step(step, expected.next().unwrap());
        }
    }
    assert!(expected.next().is_none());
}
