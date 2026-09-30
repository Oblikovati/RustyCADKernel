// Rust translation of ModelingAlgorithms/TKGeomAlgo/Hatch (Hatch_LineForm.hxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Hatch: a simple cross-hatching algorithm on straight 2d lines,
//! trimmed by straight boundaries (used for display).
//!
//! `Hatch_Parameter` and `Hatch_Line` keep their data private natively and grant
//! access to their friends `Hatch_Line` and `Hatch_Hatcher`; the port keeps the
//! fields private to this module for the same reason.
mod hatcher;
mod line;
mod parameter;
pub use hatcher::Hatcher;
pub use line::Line;
pub use parameter::Parameter;

/// Hatch_LineForm: the form of a trimmed line, kept for information only;
/// `LineForm::AnyLine as i32 == 2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LineForm {
    /// Hatch_XLINE: a line of constant X.
    XLine,
    /// Hatch_YLINE: a line of constant Y.
    YLine,
    /// Hatch_ANYLINE: any other line.
    #[default]
    AnyLine,
}
