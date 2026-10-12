//! Actual application virtual dispatch retains native state/RTTI and cannot override nonvirtual members.
use super::*;
use super::{
    xr_derived_fake::XrDerivedFake, xr_recording_group::*,
    xr_recording_structure::xr_recording_presentation,
};
use crate::{prs_mgr::*, select_mgr::*, select3d::Select3DSensitiveEntityMembers};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
struct XrDispatchReplay {
    derived: Handle<XrDerivedFake>,
    source: Handle<dyn AisXrTrackedDeviceSource>,
    prs: Handle<dyn Graphic3dStructureSource>,
    selection: Handle<SelectMgrSelection>,
    _context: Handle<AisInteractiveContext>,
}
impl XrDispatchReplay {
    fn new() -> Self {
        let events = Arc::new(XrRecordingEvents::new());
        let derived = XrDerivedFake::new(events.clone());
        let source = derived.clone().cast::<dyn AisXrTrackedDeviceSource>();
        let (_, context) = super::xr_state_fixture::xr_state_device(0);
        source.set_context(&context);
        let prs = xr_recording_presentation(&events);
        let selection = Handle::new(SelectMgrSelection::new(0));
        Self {
            derived,
            source,
            prs,
            selection,
            _context: context,
        }
    }
    fn operate(&self, step: i32) -> Result<(), Failure> {
        match step {
            0 => self.source.set_laser_length(1.),
            1 => self.source.compute(&Handle::default(), &self.prs, 0),
            2 => self.source.set_laser_length(2.),
            3 => self.source.compute(&Handle::default(), &self.prs, 1),
            4 | 9 => self.source.compute_selection(&self.selection, 0),
            5 => self.source.compute_selection(&self.selection, 1),
            _ => self.operate_private(step),
        }
    }
    fn operate_private(&self, step: i32) -> Result<(), Failure> {
        if step == 6 {
            self.derived.hidden_laser_ray();
            return Ok(());
        }
        if step == 7 {
            return self.source.compute_laser_ray();
        }
        *self.derived.fail_after_compute.write() = true;
        self.source.compute(&Handle::default(), &self.prs, 0)
    }
    fn snapshot(&self, step: i32, failure: bool) -> Vec<f64> {
        let mut row = xr_dispatch_header(self, step, failure);
        append_xr_dispatch_traits(&mut row, self);
        let callbacks = self.derived.events.callbacks.read();
        row.push(callbacks.len() as f64);
        row.extend(callbacks.iter().copied().map(f64::from));
        row.push(f64::from(self.prs.number_of_groups()));
        for group in self.prs.groups() {
            row.extend(group.cast::<XrRecordingGroup>().snapshot());
        }
        row
    }
    fn replay(&self, step: i32, expected: &mut std::str::Lines<'_>) {
        self.derived.events.callbacks.write().clear();
        let result = self.operate(step);
        if let Err(ref failure) = result {
            assert_eq!(failure.exception_type(), "Standard_Failure");
        }
        crate::select3d::compare_oracle_snapshot(
            &self.snapshot(step, result.is_err()),
            expected.next().unwrap(),
        );
        self.verify_selected_owners();
    }
    fn verify_selected_owners(&self) {
        for entity in self.selection.entities() {
            let owner = entity.base_sensitive().owner_id();
            assert_eq!(
                owner.selectable().object_address(),
                self.source.object_address()
            );
        }
    }
}
fn xr_dispatch_header(scene: &XrDispatchReplay, step: i32, failure: bool) -> Vec<f64> {
    vec![
        f64::from(step),
        f64::from(failure),
        f64::from(scene.source.role() as i32),
        f64::from(scene.derived.role() as i32),
        f64::from(*scene.derived.hiding_calls.read()),
        f64::from(scene.source.laser_length()),
    ]
}
fn append_xr_dispatch_traits(row: &mut Vec<f64>, scene: &XrDispatchReplay) {
    row.extend([
        f64::from(scene.source.is_instance(AisXrTrackedDevice::static_type())),
        f64::from(
            scene
                .source
                .is_kind(<dyn AisInteractiveObject>::static_type()),
        ),
        f64::from(scene.source.accept_display_mode(0)),
        f64::from(scene.source.accept_display_mode(7)),
        scene.selection.entities().len() as f64,
    ]);
}
fn verify_xr_derived_identity(scene: &XrDispatchReplay) {
    let source = &scene.source;
    let presentable = source.clone().cast::<dyn PrsMgrPresentableObject>();
    let selectable = source.clone().cast::<dyn SelectMgrSelectableObject>();
    let concrete = source.clone().cast::<AisXrTrackedDevice>();
    assert_eq!(scene.derived.object_address(), source.object_address());
    for address in [
        presentable.object_address(),
        selectable.object_address(),
        concrete.object_address(),
        source.presentable_base().this().object_address(),
    ] {
        assert_eq!(address, source.object_address());
    }
}
#[test]
fn native_xr_application_dispatch_default_core_and_nonvirtual_binding() {
    let scene = XrDispatchReplay::new();
    verify_xr_derived_identity(&scene);
    let mut expected = include_str!("../../tests/fixtures/xr-device-dispatch.txt").lines();
    for step in 0..10 {
        scene.replay(step, &mut expected);
    }
    assert!(expected.next().is_none());
}
