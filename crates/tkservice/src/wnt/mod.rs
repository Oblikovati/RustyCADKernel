//! WNT: the platform-neutral part of the Windows package (OCCT package WNT).
//!
//! `WNT_Window` and `WNT_WClass` exist only on Windows (visualization-design.md section 4);
//! the Space Mouse decoder, the orientation enumeration and the class-definition exception
//! are built on every platform and ported here.
mod hid_space_mouse;

use rustycad_tkernel::standard::Failure;

pub use hid_space_mouse::{
    WNT_HID_VENDOR_ID_3DCONNEXION, WNT_HID_VENDOR_ID_LOGITECH, WntHidSpaceMouse,
};

/// Portrait or landscape orientation (`WNT_OrientationType`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WntOrientationType {
    /// `WNT_OT_PORTRAIT`.
    Portrait = 0,
    /// `WNT_OT_LANDSCAPE`.
    Landscape = 1,
}

/// `WNT_ClassDefinitionError` (a `Standard_ConstructionError`); `class_definition_error("why")`.
pub fn class_definition_error(message: &str) -> Failure {
    Failure::with_type("WNT_ClassDefinitionError", message, "")
}
