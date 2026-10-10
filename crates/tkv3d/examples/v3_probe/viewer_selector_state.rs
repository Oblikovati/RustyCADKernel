//! Native selector state operations, with protected ranking seeds and real lifecycle calls.
#[path = "../../tests/support/selector_scene.rs"]
mod selector_scene;
use super::boxed_selectable::BoxedSelectable;
use super::common::{Reader, attempt, end, flag, int, real, text};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkmath::bvh::{BvhBuilderHandle, BvhObjectTransientSource, BvhVec3d};
use rustycad_tkv3d::select_mgr::*;
use selector_scene::{mode_owner, mode_selection, seeded_pick};
struct SelectorStateFixture {
    selector: SelectMgrViewerSelector,
    objects: Vec<Handle<dyn SelectMgrSelectableObject>>,
    selections: Vec<Handle<SelectMgrSelection>>,
    owners: Vec<Handle<dyn SelectMgrEntityOwnerSource>>,
    builder: BvhBuilderHandle<BvhVec3d>,
}
impl SelectorStateFixture {
    fn new() -> Self {
        let selector = SelectMgrViewerSelector::new();
        let builder = selector.entity_set_builder();
        let objects: Vec<Handle<dyn SelectMgrSelectableObject>> = (0..3)
            .map(|i| BoxedSelectable::new(f64::from(i * 3), false).cast())
            .collect();
        let selections: Vec<_> = objects
            .iter()
            .enumerate()
            .map(|(i, object)| mode_selection(object, i as i32))
            .collect();
        let owners = selections
            .iter()
            .map(|selection| mode_owner(selection))
            .collect();
        Self {
            selector,
            objects,
            selections,
            owners,
            builder,
        }
    }
    fn owner_id(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> i32 {
        if owner.is_null() {
            return -1;
        }
        self.owners
            .iter()
            .position(|candidate| candidate == owner)
            .expect("fixture owner identity") as i32
    }
    fn operation(&mut self, op: i32, a: i32, b: i32, c: i32) -> Result<(), Failure> {
        let selector = &self.selector;
        match op {
            0 => selector.set_pixel_tolerance(a),
            1 => selector.set_depth_tolerance(
                [
                    SelectMgrTypeOfDepthTolerance::Uniform,
                    SelectMgrTypeOfDepthTolerance::UniformPixels,
                    SelectMgrTypeOfDepthTolerance::SensitivityFactor,
                ][a as usize],
                f64::from(b),
            ),
            2 => selector.set_pick_closest(a != 0),
            3 => selector.add_selectable_object(&self.objects[a as usize])?,
            4 => selector.remove_selectable_object(&self.objects[a as usize])?,
            5 => self.owners.push(seeded_pick(
                selector,
                &self.objects[a as usize],
                f64::from(b),
                c,
            )),
            6 => selector.sort_result()?,
            7 => selector.clear(),
            8 => int(self.owner_id(&selector.picked(a)?)),
            9 => {
                let picked = selector.picked_data(a)?;
                real(picked.depth);
                real(picked.point.x());
            }
            10 => flag(selector.remove_picked(&self.objects[a as usize])?),
            11 => selector
                .add_selection_to_object(&self.objects[a as usize], &self.selections[a as usize])?,
            12 => selector.remove_selection_of_object(
                &self.objects[a as usize],
                &self.selections[a as usize],
            )?,
            13 => selector.rebuild_objects_tree(a != 0)?,
            14 => selector.rebuild_sensitives_tree(&self.objects[a as usize], b != 0)?,
            _ => self.other_operation(op, a, b, c)?,
        }
        Ok(())
    }
    fn other_operation(&self, op: i32, a: i32, b: i32, c: i32) -> Result<(), Failure> {
        let selector = &self.selector;
        match op {
            15 => text(
                selector
                    .object_status(&self.objects[a as usize])?
                    .as_bytes(),
            ),
            16 => self.print_modes(a, b)?,
            17 => selector.reset_selection_activation_status()?,
            18 => self.print_owners()?,
            19 => selector.set_entity_set_builder(if a == 0 { None } else { self.builder.clone() }),
            20 => selector.set_to_prebuild_bvh(a != 0, b),
            21 => selector.wait_for_bvh_build(),
            22 => selector.move_selectable_object(&self.objects[a as usize])?,
            23 => selector.allow_overlap_detection(a != 0),
            24 => {
                self.selections[a as usize].set_selection_state(selection_state(b));
                if c != 0 {
                    self.selections[a as usize].entities()[0].set_active_for_selection();
                }
            }
            25 => selector
                .queue_bvh_build(&self.selections[a as usize].entities()[0].base_sensitive())?,
            26 => selector.viewer_selector_base().tolerances.write().add(a),
            27 => selector
                .viewer_selector_base()
                .tolerances
                .write()
                .decrement(a)?,
            28 => {
                let mut stream = rustycad_tkernel::standard::dump::DumpStream::default();
                selector.dump_json(&mut stream, a).unwrap();
                text(
                    super::frustum_core::named_pointer_identities(
                        stream.text().to_owned(),
                        &["SelectableObjects", "EntitySetBuilder"],
                    )
                    .as_bytes(),
                );
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    fn print_modes(&self, index: i32, wanted: i32) -> Result<(), Failure> {
        let mut modes = List::default();
        modes.append(99);
        flag(self.selector.modes(
            &self.objects[index as usize],
            &mut modes,
            selection_state(wanted),
        )?);
        int(modes.len() as i32);
        for mode in modes.iter() {
            int(*mode);
        }
        Ok(())
    }
    fn print_owners(&self) -> Result<(), Failure> {
        let mut owners = List::default();
        self.selector.active_owners(&mut owners)?;
        let mut ids: Vec<_> = owners.iter().map(|owner| self.owner_id(owner)).collect();
        ids.sort();
        int(ids.len() as i32);
        for id in ids {
            int(id);
        }
        Ok(())
    }
    fn state(&self) {
        let selector = &self.selector;
        int(selector.custom_pixel_tolerance());
        int(selector.pixel_tolerance());
        real(selector.sensitivity());
        flag(selector.to_pick_closest());
        int(selector.depth_tolerance_type() as i32);
        real(selector.depth_tolerance());
        int(selector.nb_picked());
        let fields = selector.viewer_selector_base().fields.read();
        flag(fields.results.is_sorted);
        int(fields.results.indexes.len() as i32);
        for index in fields.results.indexes.as_slice() {
            int(*index);
        }
        drop(fields);
        flag(selector.to_prebuild_bvh());
        flag(selector.entity_set_builder().is_none());
        self.object_states();
    }
    fn object_states(&self) {
        let selector = &self.selector;
        for (i, object) in self.objects.iter().enumerate() {
            flag(selector.contains(object));
            flag(selector.is_active(object, i as i32).unwrap());
            flag(selector.is_inside(object, i as i32).unwrap());
            int(selector.selection_status(&self.selections[i]).unwrap() as i32);
            let sets = selector.viewer_selector_base().object_sensitives.read();
            let set = sets.seek(object);
            int(set.map_or(-1, |set| set.size()));
            flag(set.is_some_and(|set| set.is_dirty()));
        }
    }
}
fn selection_state(value: i32) -> SelectMgrStateOfSelection {
    match value {
        -2 => SelectMgrStateOfSelection::Any,
        -1 => SelectMgrStateOfSelection::Unknown,
        0 => SelectMgrStateOfSelection::Deactivated,
        1 => SelectMgrStateOfSelection::Activated,
        _ => unreachable!(),
    }
}
pub fn viewer_selector_state_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let steps = reader.integer();
    let mut fixture = SelectorStateFixture::new();
    fixture.state();
    for _ in 0..steps {
        let (op, a, b, c) = (
            reader.integer(),
            reader.integer(),
            reader.integer(),
            reader.integer(),
        );
        attempt(|| fixture.operation(op, a, b, c));
        fixture.state();
    }
    end();
}
