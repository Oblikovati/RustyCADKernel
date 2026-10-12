//! Qualified native failure transport must never erase an unrelated application panic.
use super::*;
use super::{
    xr_recording_group::XrRecordingEvents, xr_recording_structure::xr_recording_presentation,
};
use crate::prs_mgr::PrsMgrPresentableObject;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};
#[test]
fn xr_ray_refresh_resumes_unrelated_application_panic_after_assigning_color() {
    let (device, _context) = super::xr_state_fixture::xr_state_device(0);
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    device.set_laser_length(1.).unwrap();
    device.compute(&Handle::default(), &prs, 0).unwrap();
    let alias = device.laser_color_ref();
    *events.unexpected_panic.write() = true;
    let panic = catch_unwind(AssertUnwindSafe(|| {
        device.set_laser_color(Color::named(NamedColor::RED))
    }))
    .unwrap_err();
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"Martensite_XrUnexpectedCallback")
    );
    assert_eq!(alias.value(), Color::named(NamedColor::RED));
    *events.unexpected_panic.write() = false;
    device.set_laser_length(2.).unwrap();
}
