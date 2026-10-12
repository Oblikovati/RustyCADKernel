//! Real managed presentations keep mode-zero and mode-one cache status across non-invalidating setters.
use super::xr_recording_group::{XrRecordingEvents, XrRecordingGroup};
use super::xr_recording_structure::install_xr_recording;
use super::*;
use crate::prs_mgr::*;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkservice::{
    aspect::AspectXRTrackedDeviceRole, graphic3d::*, null_driver::NullGraphicDriver,
};
use std::sync::Arc;
struct XrCachedScene {
    device: Handle<AisXrTrackedDevice>,
    _context: Handle<AisInteractiveContext>,
    presentations: Handle<PrsMgrPresentationManager>,
    _manager: Handle<Graphic3dStructureManager>,
    zero: Handle<PrsMgrPresentation>,
    one: Handle<PrsMgrPresentation>,
    events: Arc<XrRecordingEvents>,
}
impl XrCachedScene {
    fn new() -> Self {
        let (device, context) = super::xr_state_fixture::xr_state_device(0);
        let manager =
            Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
        let presentations = PrsMgrPresentationManager::new(&manager);
        let object = device.clone().cast();
        let (zero, one, events) = xr_cached_modes(&presentations, &object);
        Self {
            device,
            _context: context,
            presentations,
            _manager: manager,
            zero,
            one,
            events,
        }
    }
    fn mutate(&self, step: i32) {
        if step < 4 {
            self.mutate_role_unit(step);
            return;
        }
        match step {
            4 => self.device.set_laser_length(2.).unwrap(),
            5 => self
                .device
                .set_laser_color(Color::named(NamedColor::GREEN))
                .unwrap(),
            6 => self.device.compute_laser_ray().unwrap(),
            _ => self
                .presentations
                .update(&self.device.clone().cast(), 0)
                .unwrap(),
        }
    }
    fn mutate_role_unit(&self, step: i32) {
        self.device.set_role(
            [
                AspectXRTrackedDeviceRole::Head,
                AspectXRTrackedDeviceRole::LeftHand,
                AspectXRTrackedDeviceRole::RightHand,
                AspectXRTrackedDeviceRole::Other,
            ][step as usize],
        );
        self.device
            .set_unit_factor(if step == 0 { -3. } else { step as f32 });
    }
    fn snapshot(&self, step: i32) -> Vec<f64> {
        let mut row = xr_cached_header(self, step);
        row.extend(self.events.callbacks.read().iter().copied().map(f64::from));
        row.extend([
            f64::from(self.zero.number_of_groups()),
            f64::from(self.one.number_of_groups()),
        ]);
        for group in self.zero.groups() {
            row.extend(group.cast::<XrRecordingGroup>().snapshot());
        }
        row
    }
}
fn xr_cached_header(scene: &XrCachedScene, step: i32) -> Vec<f64> {
    vec![
        f64::from(step),
        f64::from(scene.device.role() as i32),
        f64::from(scene.device.unit_factor()),
        f64::from(scene.device.laser_length()),
        f64::from(scene.device.to_be_updated(false)),
        f64::from(scene.zero.must_be_updated()),
        f64::from(scene.one.must_be_updated()),
        scene.events.callbacks.read().len() as f64,
    ]
}
fn xr_cached_modes(
    presentations: &Handle<PrsMgrPresentationManager>,
    object: &Handle<dyn PrsMgrPresentableObject>,
) -> (
    Handle<PrsMgrPresentation>,
    Handle<PrsMgrPresentation>,
    Arc<XrRecordingEvents>,
) {
    let zero = presentations
        .presentation(object, 0, true, &Handle::default())
        .unwrap();
    let one = presentations
        .presentation(object, 1, true, &Handle::default())
        .unwrap();
    let events = Arc::new(XrRecordingEvents::new());
    install_xr_recording(&zero.clone().cast(), &events);
    presentations.update(object, 0).unwrap();
    (zero, one, events)
}
#[test]
fn native_xr_existing_mode_cache_unit_role_and_nonvirtual_ray_setters() {
    let scene = XrCachedScene::new();
    let mut expected =
        include_str!("../../tests/fixtures/xr-device-cached-presentation.txt").lines();
    for step in 0..8 {
        scene.events.callbacks.write().clear();
        scene.mutate(step);
        crate::select3d::compare_oracle_snapshot(&scene.snapshot(step), expected.next().unwrap());
    }
    assert!(expected.next().is_none());
}
