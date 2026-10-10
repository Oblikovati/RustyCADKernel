//! Actual AIS display/erase callbacks and retained state compared with pinned OCCT.
use super::{
    ais_load::{LoadScene, display_state},
    common::{Reader, attempt, end, flag, int},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::{aspect::AspectNeutralWindow, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*};
use std::sync::atomic::Ordering;
pub(super) fn operate(
    scene: &LoadScene,
    op: i32,
    index: usize,
    values: [i32; 4],
) -> Result<(), Failure> {
    let [a, b, c, d] = values;
    let object = scene.objects[index].cast();
    match op {
        0..=14 => scene.operate(op, index, a, b)?,
        20 => scene.ctx.display(&object, a != 0)?,
        21 => scene
            .ctx
            .display_with_modes(&object, a, b, c != 0, display_state(d))?,
        22 => scene.ctx.erase(&object, a != 0)?,
        23 => scene.ctx.erase_global(&object, a != 0)?,
        24 => scene.ctx.unselect_owners(&object)?,
        _ => configure(scene, op, index, a, b)?,
    }
    Ok(())
}
fn configure(scene: &LoadScene, op: i32, index: usize, a: i32, b: i32) -> Result<(), Failure> {
    let object = &scene.objects[index];
    match op {
        25 => scene.ctx.set_auto_activate_selection(a != 0),
        26 => scene.ctx.set_automatic_hilight(a != 0),
        27 => object.set_auto_hilight(a != 0),
        28 => object.fail_mode.store(a, Ordering::Release),
        29 => {
            scene.ctx.add_select_object(&object.cast())?;
        }
        30 => {
            let style = scene.ctx.selection_style().clone();
            scene
                .ctx
                .hilight_with_color(&object.cast(), &style, false)?;
        }
        31 => scene.ctx.unhilight(&object.cast(), false)?,
        32 => object.global_sel_owner().set_selected(a != 0),
        33 => {
            scene.ctx.fields.write().last_picked = if object.is_null() {
                Handle::default()
            } else {
                object.global_sel_owner()
            }
        }
        _ => configure_managers(scene, op, index, a, b)?,
    }
    Ok(())
}
fn configure_managers(
    scene: &LoadScene,
    op: i32,
    index: usize,
    a: i32,
    b: i32,
) -> Result<(), Failure> {
    let object = &scene.objects[index];
    match op {
        35 => {
            let manager = scene.ctx.selection_manager().clone();
            manager.load(&object.cast(), a)?;
            if b != 0 {
                manager.activate(&object.cast(), a)?;
            }
        }
        36 => {
            if let Some(status) = scene.ctx.fields.read().objects.seek(&object.cast()) {
                status.remove_selection_mode(a);
            }
        }
        37 => {
            let window = Handle::new(AspectNeutralWindow::default());
            window.set_size(80, 60);
            scene.views[a as usize].set_window(&window.cast(), 0)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
fn presentation_state(object: &Handle<dyn AisInteractiveObject>) {
    for presentation in object.presentations() {
        int(presentation.mode());
        flag(presentation.is_displayed());
        flag(presentation.is_visible());
        flag(presentation.is_highlighted());
        flag(presentation.must_be_updated());
        int(presentation.groups().len() as i32);
    }
}
fn selection_state(scene: &LoadScene, object: &Handle<dyn AisInteractiveObject>) {
    let manager = scene.ctx.selection_manager().clone();
    for selection in object.selections() {
        int(selection.mode());
        int(selection.get_selection_state() as i32);
        int(selection.update_status() as i32);
        int(selection.entities().len() as i32);
        flag(
            manager
                .is_activated(&object.cast(), selection.mode())
                .unwrap(),
        );
    }
    let owner = object.global_sel_owner();
    flag(!owner.is_null());
    if !owner.is_null() {
        flag(owner.is_selected());
    }
    flag(scene.ctx.is_hilighted_object(object));
}
pub(super) fn display_state_rows(scene: &LoadScene) {
    int(scene.ctx.nb_selected());
    for owner in scene.ctx.selection().objects().iter() {
        int(scene
            .objects
            .iter()
            .position(|object| owner.is_same_selectable(&object.cast()))
            .map_or(-1, |index| index as i32));
    }
    for object in &scene.objects {
        scene.state(object);
        if !object.is_null() {
            presentation_state(&object.cast());
            selection_state(scene, &object.cast());
        }
    }
}
pub fn ais_display_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let scene = LoadScene::new();
    for _ in 0..reader.integer() {
        let (op, index) = (reader.integer(), reader.integer());
        let arguments = std::array::from_fn(|_| reader.integer());
        attempt(|| operate(&scene, op, index as usize, arguments));
        display_state_rows(&scene);
    }
    end();
}
