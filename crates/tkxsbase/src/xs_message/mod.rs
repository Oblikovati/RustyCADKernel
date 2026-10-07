// Rust translation of DataExchange/TKXSBase/XSMessage at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package XSMessage: the message texts of the data exchange toolkits.
//!
//! The package holds no code: each `.pxx` file defines one C string literal compiled from
//! the resource files `resources/XSMessage/XSTEP.us` and `XSTEP.fr`. The literals are
//! data, carried over byte for byte by `tools/generate_xsmessage_data.py` (with their
//! provenance in `porting/xsmessage-data.json`) and embedded here. Only the `us` literal is
//! compiled natively (the package's FILES.cmake lists `XSMessage_XSTEP_us.pxx` alone), and
//! `Interface_StaticStandards` loads it into the message catalog.

/// `XSMessage_XSTEP_us`: the English message texts, the bytes of the literal without its
/// terminating NUL (`sizeof(XSMessage_XSTEP_us) - 1` natively).
///
/// ```
/// use rustycad_tkxsbase::xs_message::XSTEP_US;
/// let text = std::str::from_utf8(XSTEP_US).unwrap();
/// assert!(text.starts_with("!LOADING PHASE\n.XSTEP_1\n"));
/// ```
pub const XSTEP_US: &[u8] = include_bytes!("XSTEP.us");

/// `XSMessage_XSTEP_fr`: the French message texts. The file ships with the package but is
/// not compiled natively; no member reads it.
///
/// ```
/// use rustycad_tkxsbase::xs_message::XSTEP_FR;
/// assert!(XSTEP_FR.len() > 1000);
/// ```
pub const XSTEP_FR: &[u8] = include_bytes!("XSTEP.fr");
