//! Pinned actual-native differential replay includes callback faults, repair and retained aliases.
use super::*;
use super::{
    xr_recording_group::*, xr_recording_structure::xr_recording_presentation, xr_state_fixture::*,
};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
const XR_STATE_NATIVE: &str = concat!(
    include_str!("../../tests/fixtures/xr-device-state-00.txt"),
    include_str!("../../tests/fixtures/xr-device-state-01.txt")
);
fn xr_state_columns(
    device: &Handle<AisXrTrackedDevice>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    alias: &AisXrLaserColorRef<'_>,
    events: &XrRecordingEvents,
    ordinal: i32,
    step: i32,
    failure: bool,
) -> Vec<f64> {
    let color = alias.value().rgb();
    let mut row = vec![
        f64::from(ordinal),
        f64::from(step),
        f64::from(failure),
        f64::from(device.role() as i32),
        f64::from(device.unit_factor()),
        f64::from(device.laser_length()),
    ];
    row.extend(color.map(f64::from));
    row.push(f64::from(
        alias.is_same_reference(&device.laser_color_ref()),
    ));
    row.extend([
        f64::from(device.to_be_updated(false)),
        f64::from(prs.is_infinite()),
        f64::from(prs.number_of_groups()),
        events.callbacks.read().len() as f64,
    ]);
    row.extend(events.callbacks.read().iter().copied().map(f64::from));
    for group in prs.groups() {
        row.extend(group.cast::<XrRecordingGroup>().snapshot());
    }
    row
}
fn replay_xr_state(ordinal: i32, expected: &mut std::str::Lines<'_>) {
    let (device, _context) = xr_state_device(ordinal);
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    let alias = device.laser_color_ref();
    let second = device.laser_color_ref();
    assert!(alias.is_same_reference(&second));
    for step in 0..20 {
        events.callbacks.write().clear();
        let result = xr_state_operation(&device, &prs, &events, step);
        if let Err(ref failure) = result {
            assert_eq!(failure.exception_type(), "Standard_Failure");
        }
        let row = xr_state_columns(
            &device,
            &prs,
            &alias,
            &events,
            ordinal,
            step,
            result.is_err(),
        );
        crate::select3d::compare_oracle_snapshot(
            &row,
            expected.next().unwrap(),
        );
    }
}
#[test]
fn native_xr_models_axes_ray_accumulation_aliases_fault_repair_and_nan_comparisons() {
    let mut expected = XR_STATE_NATIVE.lines();
    for ordinal in 0..48 {
        replay_xr_state(ordinal, &mut expected);
    }
    assert!(expected.next().is_none());
}
