// Rust translation of the text and the sequence checks shared by the HatchGen and
// Geom2dHatch Dump members at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pieces of text the native `Dump` members write with `std::cout`, and the
//! bounds check of the `NCollection_Sequence` members they read.
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::format_general;
use rustycad_tkg3d::top_abs::{Orientation, State};
use std::fmt;

/// `std::cout << double` with the default stream flags: C's `%g` with six
/// significant digits; `stream_real(0.5) == "0.5"`.
pub(crate) fn stream_real(value: f64) -> String {
    format_general(value, 6)
}

/// The index part of a dump title: `# ` and the index on three columns when it is
/// positive, `fill` otherwise; `index_title(7, "---") == "#   7 "`.
pub(crate) fn index_title(index: i32, fill: &str) -> String {
    if index > 0 {
        return format!("# {index:>3} ");
    }
    fill.to_string()
}

/// Writes each `label = value` line of a dump with `indent`, the labels padded to
/// `width` columns as the native literals are.
pub(super) fn write_fields(
    out: &mut dyn fmt::Write,
    indent: &str,
    width: usize,
    fields: &[(String, String)],
) -> fmt::Result {
    for (label, value) in fields {
        writeln!(out, "{indent}{label:<width$} = {value}")?;
    }
    Ok(())
}

/// The native spelling of a position on a curve in the point dumps.
pub(super) fn position_text(position: Orientation) -> &'static str {
    match position {
        Orientation::Forward => "FORWARD  (i.e. BEGIN  )",
        Orientation::Internal => "INTERNAL (i.e. MIDDLE )",
        Orientation::Reversed => "REVERSED (i.e. END    )",
        Orientation::External => "EXTERNAL (i.e. UNKNOWN)",
    }
}

/// The native spelling of a state in the point dumps.
pub(super) fn state_text(state: State) -> &'static str {
    match state {
        State::In => "IN",
        State::Out => "OUT",
        State::On => "ON",
        State::Unknown => "UNKNOWN",
    }
}

/// The native spelling of a flag in the point dumps.
pub(super) fn flag_text(flag: bool) -> &'static str {
    if flag { "TRUE" } else { "FALSE" }
}

/// The 0-based slot of the 1-based `index` of an `NCollection_Sequence` of `len`
/// items, or the Standard_OutOfRange that `Value`, `ChangeValue` and `Remove`
/// raise; `sequence_slot(2, 3, "HatchGen_PointOnHatching::Point")` is an error.
pub(crate) fn sequence_slot(len: usize, index: i32, member: &str) -> Result<usize, Failure> {
    usize::try_from(index)
        .ok()
        .filter(|&one_based| one_based >= 1 && one_based <= len)
        .map(|one_based| one_based - 1)
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "{member}: index {index} out of range; expected in [1, {len}]"
            ))
        })
}
