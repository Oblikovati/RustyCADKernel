// Rust translation of ModelingAlgorithms/TKShHealing/SHMessage at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package SHMessage: the message texts of Shape Healing.
//!
//! The package holds no code: each `.pxx` file defines one C string literal compiled from
//! the resource files `resources/SHMessage/SHAPE.us` and `SHAPE.fr`. The literals are
//! data, carried over byte for byte by `tools/generate_shmessage_data.py` (with their
//! provenance in `porting/shmessage-data.json`) and embedded here. Only the `us` literal is
//! compiled natively (the package's FILES.cmake lists `SHMessage_SHAPE_us.pxx` alone), and
//! `ShapeExtend::init` loads it into the message catalog, as `ShapeExtend::Init` does.

/// `SHMessage_SHAPE_us`: the English message texts, the bytes of the literal without its
/// terminating NUL (`sizeof(SHMessage_SHAPE_us) - 1` natively).
///
/// ```
/// use rustycad_tkshhealing::sh_message::SHAPE_US;
/// let text = std::str::from_utf8(SHAPE_US).unwrap();
/// assert!(text.contains(".ShapeFix.FixSmallSolid.MSG0\n"));
/// ```
pub const SHAPE_US: &[u8] = include_bytes!("SHAPE.us");

/// `SHMessage_SHAPE_fr`: the French message texts. The file ships with the package but is
/// not compiled natively; no Shape Healing member reads it.
///
/// ```
/// use rustycad_tkshhealing::sh_message::SHAPE_FR;
/// assert!(SHAPE_FR.starts_with(b"! Message file for Shape Healing"));
/// ```
pub const SHAPE_FR: &[u8] = include_bytes!("SHAPE.fr");
