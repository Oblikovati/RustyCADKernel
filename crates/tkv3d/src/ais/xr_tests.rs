//! Focused Martensite XR CPU proofs keep family test wiring separate from the AIS public surface.
use super::*;
use super::{color_scale_recording, xr_tracked_device_object};
#[path = "xr_cached_presentation_tests.rs"]
mod xr_cached_presentation_tests;
#[path = "xr_callback_failure_tests.rs"]
mod xr_callback_failure_tests;
#[path = "xr_color_reference_tests.rs"]
mod xr_color_reference_tests;
#[path = "xr_derived_fake.rs"]
mod xr_derived_fake;
#[path = "xr_dispatch_tests.rs"]
mod xr_dispatch_tests;
#[path = "xr_recording_group.rs"]
pub(crate) mod xr_recording_group;
#[path = "xr_recording_structure.rs"]
pub(crate) mod xr_recording_structure;
#[path = "xr_selection_tests.rs"]
mod xr_selection_tests;
#[path = "xr_state_fixture.rs"]
mod xr_state_fixture;
#[path = "xr_state_tests.rs"]
mod xr_state_tests;
#[path = "xr_texture_source_fake.rs"]
mod xr_texture_source_fake;
#[path = "xr_texture_tests.rs"]
mod xr_texture_tests;

#[path = "xr_recording_delegate_tests.rs"]
mod xr_recording_delegate_tests;
