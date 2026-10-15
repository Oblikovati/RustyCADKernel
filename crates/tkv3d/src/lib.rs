//! Rust port of the OCCT TKV3d toolkit: selection, presentation, interactive objects and the
//! 3D viewer, over the Graphic3d driver interfaces of TKService.
//!
//! - [`ais`]: interactive objects and the interactive context.
//! - [`prs3d`]: the aspects and the drawer of presentations and the presentation tools.
//! - [`prs_mgr`]: presentable objects, their presentations and the presentation manager.
//! - [`select3d`]: the sensitive entities.
//! - [`select_basics`]: the pick result and the abstract selecting volume.
//! - [`select_mgr`]: selection modes, owners, selecting volumes and the selectors.
//! - [`v3d`]: the viewer, its views, lights, grids and trihedron.
//!
//! The design rules, including how the viewer reaches the Vulkan 1.3 driver through the
//! Graphic3d interfaces only, are in `docs/v3d-design.md`.
#[cfg(test)]
#[macro_use]
mod test_window_forwarding;
pub mod ais;
pub mod prs3d;
pub mod prs_mgr;
pub mod select3d;
pub mod select_basics;
pub mod select_mgr;
pub mod std_select;
pub mod v3d;

/// Glyph outline regularization and glyph/text topology builders.
pub mod brep_font;

/// Standard curve presentation algorithms.
pub mod std_prs;

/// Dimension and datum presentation algorithms.
pub mod dsg_prs;

/// Dimension and relation interactive objects.
pub mod prs_dim;
