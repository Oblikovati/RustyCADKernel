//! Rust port of the OCCT TKLCAF toolkit: the OCAF data framework.
//!
//! - [`tdf`]: labels, attributes, transactions, deltas and the copy tools (`TDF`).
//!
//! The design rules are in `docs/ocaf-design.md`.
mod guid_text;
pub mod json;
pub mod tdata_std;
pub mod tdf;
pub mod tdoc_std;
pub use guid_text::guid_from_text;
pub use rustycad_tkcdf::live;
