//! Rust port of the OCCT TKService toolkit: the visualization services under TKV3d.
//!
//! - [`aspect`]: window, grid, background and input definitions shared by the viewers.
//! - [`image`]: pixel maps, pixel formats and image comparison.
//! - [`font`]: font manager, FreeType fonts and text layout.
//! - [`graphic3d`]: the 3D presentation data model and the graphic driver interfaces.
//! - [`media`]: the media player context (FFmpeg is not in the oracle build).
//! - [`xw`]: the Xlib window.
//!
//! The design rules, including how the Vulkan 1.3 driver implements the Graphic3d
//! interfaces, are in `docs/visualization-design.md`.
pub mod aspect;
pub mod font;
pub mod graphic3d;
pub mod image;
pub mod media;
pub mod null_driver;
pub mod wasm;
pub mod wnt;
pub mod xw;

mod cast;
mod dynlib;
mod json_dump;
#[cfg(target_os = "linux")]
mod xlib;
