//! Load and protected registration traces through the actual Martensite AIS context.
use super::{
    ais_object::context_object::ContextObject,
    common::{Reader, attempt, end, flag, int},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::v3d::V3dViewSource;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::*};
use std::sync::atomic::Ordering;
pub(super) struct LoadScene {
    pub(super) viewer: Handle<V3dViewer>,
    pub(super) ctx: Handle<AisInteractiveContext>,
    pub(super) foreign: Handle<AisInteractiveContext>,
    pub(super) objects: [Handle<ContextObject>; 4],
    pub(super) views: [Handle<dyn V3dViewSource>; 2],
}
pub(super) fn display_state(value: i32) -> PrsMgrDisplayStatus {
    match value {
        0 => PrsMgrDisplayStatus::Displayed,
        1 => PrsMgrDisplayStatus::Erased,
        _ => PrsMgrDisplayStatus::None,
    }
}
impl LoadScene {
    pub(super) fn new() -> Self {
        let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
        Self {
            ctx: AisInteractiveContext::new(&viewer).unwrap(),
            foreign: AisInteractiveContext::new(&viewer).unwrap(),
            objects: [
                ContextObject::new(),
                ContextObject::new(),
                ContextObject::new(),
                Handle::default(),
            ],
            views: [viewer.create_view().unwrap(), viewer.create_view().unwrap()],
            viewer,
        }
    }
    pub(super) fn operate(
        &self,
        op: i32,
        index: usize,
        value: i32,
        other: i32,
    ) -> Result<(), Failure> {
        let object = &self.objects[index];
        match op {
            0 => self.ctx.load(&object.cast(), value)?,
            1 => self.ctx.load_legacy(&object.cast(), value, other != 0)?,
            2 => self.ctx.set_object_status(
                &object.cast(),
                display_state(value),
                other,
                other + 1,
            )?,
            3 => object.set_display_status(display_state(value)),
            4 => self.ctx.set_view_affinity(
                &object.cast(),
                &self.views[value as usize],
                other != 0,
            )?,
            5 => object.set_context(&if value != 0 {
                self.foreign.clone()
            } else {
                Handle::default()
            }),
            6 => self.default_modes(object),
            _ => self.configure(op, object, value, other)?,
        }
        Ok(())
    }
    fn configure(
        &self,
        op: i32,
        object: &Handle<ContextObject>,
        value: i32,
        other: i32,
    ) -> Result<(), Failure> {
        match op {
            7 => object.attributes().set_display_mode(value),
            8 => object.set_hilight_mode(value),
            9 => self.ctx.default_drawer().set_display_mode(value),
            10 => object.accepted_mode.store(value, Ordering::Release),
            11 => {
                object
                    .presentable_base()
                    .fields
                    .write()
                    .has_own_presentations = value != 0
            }
            12 => object.set_global_sel_mode(value),
            13 => {
                object.add_child(&self.objects[value as usize].cast())?;
                object.set_propagate_visual_state(other != 0);
            }
            14 => object.view_affinity().set_visible(value != 0),
            _ => unreachable!(),
        }
        Ok(())
    }
    fn default_modes(&self, object: &Handle<ContextObject>) {
        let (mut display, mut highlight, mut selection) = (17, 18, 19);
        self.ctx
            .get_def_modes(&object.cast(), &mut display, &mut highlight, &mut selection);
        int(display);
        int(highlight);
        int(selection);
    }
    pub(super) fn state(&self, object: &Handle<ContextObject>) {
        int(self.ctx.display_status(&object.cast()) as i32);
        flag(self.ctx.is_displayed(&object.cast()));
        for mode in [-2, 0, 2, 4, 7] {
            flag(self.ctx.is_displayed_mode(&object.cast(), mode));
        }
        if object.is_null() {
            return;
        }
        int(object.display_status() as i32);
        flag(object.get_context() == self.ctx);
        flag(object.get_context() == self.foreign);
        self.recorded_status(object);
        for view in &self.views {
            flag(
                object
                    .view_affinity()
                    .is_visible(view.get().unwrap().view().identification()),
            );
        }
        flag(
            self.viewer
                .structure_manager()
                .object_affinity(&object.cast())
                == object.view_affinity(),
        );
        flag(self.ctx.selection_manager().contains(&object.cast()));
        flag(self.ctx.main_selector().contains(&object.cast()));
        int(object.selections().len() as i32);
        int(object.presentations().len() as i32);
        int(object.events.read().len() as i32);
        for &event in object.events.read().iter() {
            int(event);
        }
        object.events.write().clear();
    }
    fn recorded_status(&self, object: &Handle<ContextObject>) {
        let status = self.ctx.fields.read().objects.seek(&object.cast()).cloned();
        flag(status.is_some());
        if let Some(status) = status {
            int(status.display_mode());
            int(status.selection_modes().length());
            for mode in status.selection_modes().iter() {
                int(*mode);
            }
        }
    }
}
pub fn ais_load_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let scene = LoadScene::new();
    for _ in 0..reader.integer() {
        let (op, index, value, other) = (
            reader.integer(),
            reader.integer(),
            reader.integer(),
            reader.integer(),
        );
        attempt(|| scene.operate(op, index as usize, value, other));
        for object in &scene.objects {
            scene.state(object);
        }
    }
    end();
}
