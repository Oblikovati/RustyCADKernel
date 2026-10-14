//! Test-owned Xlib operations establish delivered destruction without sleeps or global handlers.
#![allow(unsafe_code)]
use rustycad_tkservice::{
    aspect::AspectWindow,
    xw::{XwWindow, XwXEvent},
};
use x11_dl::xlib;

/// Destroy an owned fixture and retrieve its event; `destroy_native_window(&window)`.
pub fn destroy_native_window(window: &XwWindow) -> XwXEvent {
    let api = xlib::Xlib::open().unwrap();
    let connection = window.window_base().fields().display.clone();
    let display = connection.get_display_aspect() as *mut xlib::Display;
    // SAFETY: the test owns this live window and retains its display through event delivery.
    unsafe {
        (api.XDestroyWindow)(display, window.native_handle());
        (api.XSync)(display, 0);
    }
    take_destroy_event(&api, display, window.native_handle())
}
fn take_destroy_event(api: &xlib::Xlib, display: *mut xlib::Display, window: u64) -> XwXEvent {
    // SAFETY: XEvent is a plain C union, and the live display just completed destruction.
    let mut event = unsafe { std::mem::zeroed() };
    let found =
        unsafe { (api.XCheckTypedWindowEvent)(display, window, xlib::DestroyNotify, &mut event) };
    assert_ne!(
        found, 0,
        "window={window}; expected delivered DestroyNotify"
    );
    encode_event(event)
}
/// Distinguish the event parent from its target; `synthetic_destroy_event(8, 9)`.
pub fn synthetic_destroy_event(parent: u64, destroyed: u64) -> XwXEvent {
    encode_event(xlib::XEvent {
        destroy_window: xlib::XDestroyWindowEvent {
            type_: xlib::DestroyNotify,
            serial: 0,
            send_event: 0,
            display: std::ptr::null_mut(),
            event: parent,
            window: destroyed,
        },
    })
}
fn encode_event(event: xlib::XEvent) -> XwXEvent {
    // SAFETY: both representations preserve exactly the same plain XEvent union bytes.
    let bytes = unsafe { std::mem::transmute::<xlib::XEvent, [u8; XwXEvent::SIZE]>(event) };
    XwXEvent::from_bytes(&bytes)
}
/// Exercise the public event path; `deliver(&window, &mut event)`.
pub fn deliver(window: &XwWindow, event: &mut XwXEvent) -> bool {
    let mut listener = rustycad_tkv3d::ais::AisViewController::new().unwrap();
    window.process_message(&mut listener, event).unwrap()
}
