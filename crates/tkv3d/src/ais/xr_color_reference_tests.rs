//! Retained native const references observe the assignment preceding failed ray refresh.
use super::*;
use super::{
    xr_recording_group::*, xr_recording_structure::xr_recording_presentation,
    xr_state_fixture::xr_state_device,
};
use crate::prs_mgr::PrsMgrPresentableObject;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
struct XrColorReferenceReplay {
    device: Handle<AisXrTrackedDevice>,
    other: Handle<AisXrTrackedDevice>,
    _context: Handle<AisInteractiveContext>,
    prs: Handle<dyn Graphic3dStructureSource>,
    events: Arc<XrRecordingEvents>,
    infinite: i32,
}
impl XrColorReferenceReplay {
    fn new(infinite: i32) -> Self {
        let (device, context) = xr_state_device(infinite * 4);
        let other = AisXrTrackedDevice::new().unwrap();
        device.set_laser_length(1.).unwrap();
        let events = Arc::new(XrRecordingEvents::new());
        let prs = xr_recording_presentation(&events);
        device.compute(&Handle::default(), &prs, 0).unwrap();
        Self {
            device,
            other,
            _context: context,
            prs,
            events,
            infinite,
        }
    }
    fn operate(&self, step: i32) -> Result<(), Failure> {
        match step {
            0 | 1 => self.device.set_laser_color(Color::named(NamedColor::GREEN)),
            2 => self.device.set_laser_color(Color::named(NamedColor::RED)),
            _ => self.device.compute_laser_ray(),
        }
    }
    fn snapshot(
        &self,
        aliases: [&AisXrLaserColorRef<'_>; 3],
        copy: Color,
        step: i32,
        failure: bool,
    ) -> Vec<f64> {
        let mut row = xr_color_reference_header(self, aliases, step, failure);
        row.extend(aliases[0].value().rgb().map(f64::from));
        row.extend(copy.rgb().map(f64::from));
        let callbacks = self.events.callbacks.read();
        row.push(callbacks.len() as f64);
        row.extend(callbacks.iter().copied().map(f64::from));
        for group in self.prs.groups() {
            row.extend(group.cast::<XrRecordingGroup>().snapshot());
        }
        row
    }
    fn replay(
        &self,
        aliases: [&AisXrLaserColorRef<'_>; 3],
        copy: Color,
        step: i32,
        expected: &mut std::str::Lines<'_>,
    ) {
        self.events.callbacks.write().clear();
        *self.events.fail_aspect.write() = step == 0;
        let result = self.operate(step);
        if let Err(ref failure) = result {
            assert_eq!(failure.exception_type(), "Standard_Failure");
        }
        crate::select3d::compare_oracle_snapshot(
            &self.snapshot(aliases, copy, step, result.is_err()),
            expected.next().unwrap(),
        );
    }
}
fn xr_color_reference_header(
    scene: &XrColorReferenceReplay,
    aliases: [&AisXrLaserColorRef<'_>; 3],
    step: i32,
    failure: bool,
) -> Vec<f64> {
    let [first, second, other] = aliases;
    vec![
        f64::from(scene.infinite),
        f64::from(step),
        f64::from(failure),
        f64::from(first.is_same_reference(second)),
        f64::from(!first.is_same_reference(other)),
        f64::from(first.is_same_reference(&scene.device.laser_color_ref())),
    ]
}
fn replay_xr_color_reference(infinite: i32, expected: &mut std::str::Lines<'_>) {
    let scene = XrColorReferenceReplay::new(infinite);
    let first = scene.device.laser_color_ref();
    let second = scene.device.laser_color_ref();
    let other = scene.other.laser_color_ref();
    let copy = first.value();
    for step in 0..4 {
        scene.replay([&first, &second, &other], copy, step, expected);
    }
}
#[test]
fn native_xr_color_reference_is_live_across_failure_and_does_not_retain_old_copy() {
    let mut expected = include_str!("../../tests/fixtures/xr-device-color-reference.txt").lines();
    for infinite in 0..2 {
        replay_xr_color_reference(infinite, &mut expected);
    }
    assert!(expected.next().is_none());
}
