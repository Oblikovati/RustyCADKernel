//! Compare direct, group and nested-group threshold prebuilding with native OCCT.
#[path = "../../tests/support/prebuild_entity.rs"]
#[allow(dead_code)]
mod prebuild_entity;
use super::common::{Reader, attempt, end, int};
use prebuild_entity::PrebuildEntity;
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::{
    select_mgr::SelectMgrSelection, select3d::Select3DSensitiveGroup,
    std_select::StdSelectBrepSelectionTool,
};
use std::sync::atomic::Ordering;
pub fn prebuild_threshold_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (grouped, count, fail_at) = (r.integer(), r.integer(), r.integer());
    let selection = SelectMgrSelection::new(0);
    let group = Handle::new(Select3DSensitiveGroup::new(&Handle::default(), true));
    let mut entities = Vec::new();
    for index in 0..count {
        let entity = PrebuildEntity::new(r.integer(), index == fail_at);
        if grouped != 0 {
            group.add(&entity.cast());
        } else {
            selection.add(&entity.cast()).unwrap();
        }
        entities.push(entity);
    }
    attach_prebuild_group(&selection, &group, grouped);
    attempt(|| StdSelectBrepSelectionTool::pre_build_bvh(&selection));
    for entity in entities {
        int(entity.calls.load(Ordering::Acquire) as i32);
    }
    end();
}
fn attach_prebuild_group(
    selection: &SelectMgrSelection,
    group: &Handle<Select3DSensitiveGroup>,
    grouped: i32,
) {
    if grouped == 2 {
        let outer = Handle::new(Select3DSensitiveGroup::new(&Handle::default(), true));
        outer.add(&group.cast());
        selection.add(&outer.cast()).unwrap();
    } else if grouped != 0 {
        selection.add(&group.cast()).unwrap();
    }
}
