//! Rust port of the OCCT TKDraw toolkit: the DRAW test harness.
//!
//! - [`draw`]: the `Draw` package — the Tcl-based command interpreter
//!   ([`draw::DrawInterpretor`]), variables and drawables, and the base command groups.
//!
//! Tcl is reached through the project-owned `rustycad-tcl` binding. The design (window
//! layer, session state that replaces the C++ globals, deviations) is in
//! `docs/draw-design.md`.
pub mod draw;
