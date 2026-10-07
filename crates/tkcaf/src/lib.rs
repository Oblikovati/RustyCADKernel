//! Rust port of the OCCT TKCAF toolkit: the topological naming and the extended attributes
//! of OCAF.
//!
//! - [`tnaming`]: named shapes, their evolution graph, names and selections (`TNaming`).
//! - [`tdata_xtd`]: construction geometry, constraints, patterns, presentation and
//!   triangulation attributes (`TDataXtd`).
//! - [`app_std`]: the standard OCAF application (`AppStd`).
//!
//! The design rules are in `docs/ocaf-design.md` (sections 10 and 11).
pub mod app_std;
pub mod tdata_xtd;
pub mod tnaming;
