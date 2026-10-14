//! Aspect: window, grid, background and input definitions (OCCT package Aspect).
//!
//! Each class keeps its package prefix (`Aspect_Grid` is [`AspectGrid`]); abstract classes
//! are traits over embedded bases (visualization-design.md section 1).
mod background;
mod circular_grid;
mod display_connection;
pub mod exceptions;
mod gen_id;
mod grid;
mod grid_params;
mod input_values;
mod native_types;
mod neutral_window;
mod openvr_session;
mod rectangular_grid;
mod skydome_background;
mod style_enums;
mod view_enums;
mod vkey;
mod vkey_set;
mod window;
mod window_input_listener;
mod xr_action;
mod xr_session;

pub use background::{AspectBackground, AspectGradientBackground};
pub use circular_grid::{AspectCircularGrid, AspectCircularGridBase, AspectCircularGridMembers};
pub use display_connection::{
    AspectDisplayConnection, AspectDisplayConnectionLease, AspectDisplayFailureClass,
    AspectDisplayLeaseCounts, AspectDisplayLeasePurpose, AspectDisplayLifecycleCause,
    AspectDisplayLifecycleFailure, AspectDisplayOperation,
};
pub use gen_id::AspectGenId;
pub use grid::{AspectGrid, AspectGridBase, AspectGridFields, AspectGridMembers};
pub use grid_params::AspectGridParams;
pub use input_values::{
    AspectFrustumLrbt, AspectScrollDelta, AspectTouch, AspectTrackedDevicePose,
};
pub use native_types::{
    AspectDisplay, AspectDrawable, AspectFbConfig, AspectHandle, AspectRenderingContext,
};
pub use neutral_window::AspectNeutralWindow;
pub use openvr_session::AspectOpenVRSession;
pub use rectangular_grid::{
    AspectRectangularGrid, AspectRectangularGridBase, AspectRectangularGridMembers,
};
pub use skydome_background::AspectSkydomeBackground;
pub use style_enums::*;
pub use view_enums::*;
pub use vkey::*;
pub use vkey_set::{AspectVKeySet, AspectVKeySetState};
pub use window::{AspectWindow, AspectWindowBase, AspectWindowFields, AspectWindowMembers};
mod native_window_lifecycle;
pub use native_window_lifecycle::{
    AspectNativeWindowClosure, AspectNativeWindowClosureCause, AspectNativeWindowFailure,
    AspectNativeWindowOperation,
};
pub use window_input_listener::{
    ASPECT_TOUCH_ID_ALL, AspectWindowInputListener, AspectWindowInputListenerBase,
    AspectWindowInputListenerMembers,
};
pub use xr_action::{
    ASPECT_XR_GENERIC_ACTION_NB, AspectXRAction, AspectXRActionSet, AspectXRAnalogActionData,
    AspectXRDigitalActionData, AspectXRGenericAction, AspectXRHapticActionData,
    AspectXRPoseActionData,
};
pub use xr_session::{
    ASPECT_XR_TRACKED_DEVICE_ROLE_NB, AspectXRSession, AspectXRSessionBase, AspectXRSessionFields,
    AspectXRSessionInfoString, AspectXRSessionMembers, AspectXRSessionTrackingUniverseOrigin,
};

mod space_mouse_dispatch;
mod space_mouse_input;
pub use space_mouse_dispatch::{
    dispatch_space_mouse_keys, dispatch_space_mouse_rotation, dispatch_space_mouse_translation,
};
pub use space_mouse_input::MartensiteSpaceMouseInput;

#[cfg(test)]
mod space_mouse_dispatch_tests;

mod xr_pose_reference;
pub use xr_pose_reference::AspectXrTransformRef;

mod xr_action_reference;
pub use xr_action_reference::AspectXrActionRef;

#[cfg(test)]
mod xr_reference_tests;

mod xr_tracked_array_reference;
pub use xr_tracked_array_reference::AspectXrTrackedPosesRef;

#[cfg(test)]
mod xr_array_replay_tests;

#[cfg(test)]
mod xr_action_rebase_tests;

mod native_style_values;

#[cfg(test)]
mod native_style_values_tests;

#[cfg(all(test, target_os = "linux"))]
pub(crate) use display_connection::recorded_display_fixture;
