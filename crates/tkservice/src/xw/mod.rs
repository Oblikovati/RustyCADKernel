//! Xw: the Xlib window (OCCT package Xw), over the project-owned Xlib wrapper
//! (visualization-design.md section 2).
mod window;
mod window_events;

pub use crate::xlib::XwXEvent;
pub use window::{XwWindow, XwWindowFields};
