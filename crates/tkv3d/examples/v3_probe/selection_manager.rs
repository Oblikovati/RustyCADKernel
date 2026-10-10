//! Stateful native manager traces, including callback order and recoverable failures.
#[path = "../../tests/support/managed_selectable.rs"]
mod managed_selectable;
use super::common::{Reader, attempt, end, flag, int};
use managed_selectable::ManagedSelectionObject;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::bvh::BvhObjectTransientSource;
use rustycad_tkv3d::{prs_mgr::*, select_mgr::*, select3d::Select3DSensitiveEntityMembers};
use std::{sync::Arc, sync::atomic::Ordering};
struct ManagerProbeScene {
    selector: Handle<SelectMgrViewerSelector>,
    manager: SelectMgrSelectionManager,
    objects: Vec<Handle<ManagedSelectionObject>>,
    events: Arc<SharedState<Vec<i32>>>,
}
impl ManagerProbeScene {
    fn new(hierarchy: i32) -> Self {
        let selector = Handle::new(SelectMgrViewerSelector::new());
        let manager = SelectMgrSelectionManager::new(&selector.cast());
        let events = Arc::new(SharedState::default());
        let objects: Vec<_> = (0..3)
            .map(|tag| ManagedSelectionObject::new(tag, events.clone()))
            .collect();
        if hierarchy != 0 {
            objects[0].add_child(&objects[1].cast()).unwrap();
            objects[usize::from(hierarchy != 1)]
                .add_child(&objects[2].cast())
                .unwrap();
        }
        events.write().clear();
        Self {
            selector,
            manager,
            objects,
            events,
        }
    }
    fn operation(&self, op: i32, a: i32, b: i32, c: i32) -> Result<(), Failure> {
        let object = self.objects[a as usize].cast();
        let manager = &self.manager;
        match op {
            0 => manager.load(&object, b)?,
            1 => manager.remove(&object)?,
            2 => manager.activate(&object, b)?,
            3 => manager.deactivate(&object, b)?,
            4 => flag(manager.is_activated(&object, b)?),
            5 => manager.clear_selection_structures(&object, b)?,
            6 => manager.restore_selection_structures(&object, b)?,
            7 => manager.recompute_selection(&object, c != 0, b)?,
            8 => manager.update(&object, c != 0)?,
            9 => manager.set_update_mode(&object, update_status(c))?,
            10 => manager.set_mode_update(&object, b, update_status(c))?,
            11 => manager.set_selection_sensitivity(&object, b, c)?,
            12 => manager.update_selection(&object)?,
            _ => self.fixture_operation(op, a, b, c)?,
        }
        Ok(())
    }
    fn fixture_operation(&self, op: i32, a: i32, b: i32, c: i32) -> Result<(), Failure> {
        let object = &self.objects[a as usize];
        match op {
            13 => object.own_presentations(c != 0),
            14 => object.erased(c != 0),
            15 => object.fail_mode.store(b, Ordering::Release),
            16 => object.empty.store(c, Ordering::Release),
            17 => {
                if let Some(selection) = object.selection(b).get() {
                    selection.update_bvh_status(bvh_status(c));
                }
            }
            18 => {
                if let Some(selection) = object.selection(b).get() {
                    selection.set_selection_state(selection_status(c));
                }
            }
            19 => self.selector.set_to_prebuild_bvh(c != 0, 1),
            20 => self.selector.remove_selectable_object(&object.cast())?,
            _ => unreachable!(),
        }
        Ok(())
    }
    fn state(&self) {
        int(self.selector.pixel_tolerance());
        flag(self.selector.to_prebuild_bvh());
        for object in &self.objects {
            self.object_state(object);
        }
        let mut events = self.events.write();
        int(events.len() as i32);
        for event in events.iter() {
            int(*event);
        }
        events.clear();
    }
    fn object_state(&self, object: &Handle<ManagedSelectionObject>) {
        let selectable = object.cast();
        flag(self.manager.contains(&selectable));
        flag(self.selector.contains(&selectable));
        flag(object.has_own_presentations());
        int(object.display_status() as i32);
        let sets = self
            .selector
            .viewer_selector_base()
            .object_sensitives
            .read();
        let set = sets.seek(&selectable);
        int(set.map_or(-1, |s| s.size()));
        flag(set.is_some_and(|s| s.is_dirty()));
        let selections = object.selections();
        int(selections.len() as i32);
        for selection in selections {
            print_selection(&selection);
        }
    }
}
fn print_selection(selection: &SelectMgrSelection) {
    int(selection.mode());
    int(selection.update_status() as i32);
    int(selection.bvh_update_status() as i32);
    int(selection.get_selection_state() as i32);
    int(selection.sensitivity());
    let entities = selection.entities();
    int(entities.len() as i32);
    for entity in entities {
        flag(entity.is_active_for_selection());
        int(entity.base_sensitive().sensitivity_factor());
    }
}
fn update_status(value: i32) -> SelectMgrTypeOfUpdate {
    match value {
        0 => SelectMgrTypeOfUpdate::Full,
        1 => SelectMgrTypeOfUpdate::Partial,
        2 => SelectMgrTypeOfUpdate::None,
        _ => unreachable!(),
    }
}
fn bvh_status(value: i32) -> SelectMgrTypeOfBvhUpdate {
    match value {
        0 => SelectMgrTypeOfBvhUpdate::Add,
        1 => SelectMgrTypeOfBvhUpdate::Remove,
        2 => SelectMgrTypeOfBvhUpdate::Renew,
        3 => SelectMgrTypeOfBvhUpdate::Invalidate,
        4 => SelectMgrTypeOfBvhUpdate::None,
        _ => unreachable!(),
    }
}
fn selection_status(value: i32) -> SelectMgrStateOfSelection {
    match value {
        -1 => SelectMgrStateOfSelection::Unknown,
        0 => SelectMgrStateOfSelection::Deactivated,
        1 => SelectMgrStateOfSelection::Activated,
        _ => unreachable!(),
    }
}
pub fn selection_manager_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let scene = ManagerProbeScene::new(r.integer());
    let steps = r.integer();
    scene.state();
    for _ in 0..steps {
        let (op, a, b, c) = (r.integer(), r.integer(), r.integer(), r.integer());
        attempt(|| scene.operation(op, a, b, c));
        scene.state();
    }
    end();
}
