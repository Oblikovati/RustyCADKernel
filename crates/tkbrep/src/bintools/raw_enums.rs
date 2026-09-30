// Rust translation of the enum casts of the BinTools readers
// (`(TopAbs_ShapeEnum)IS.get()`, `(TopAbs_Orientation)aChar`,
// `(GeomAbs_Shape)IS.get()`) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The enumerators the readers take from raw bytes. Natively the cast stores any
//! value in the enum variable; a value outside the enumeration is undefined behaviour
//! once used (an array index, a switch). The port refuses such a value with
//! Standard_OutOfRange where the native code would store it.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// `(TopAbs_ShapeEnum)value`: `None` outside `COMPOUND..=SHAPE`, the values the
/// readers report as "Unexpected topology type".
pub(crate) fn shape_enum_from_raw(value: i32) -> Option<ShapeEnum> {
    usize::try_from(value)
        .ok()
        .and_then(|index| ShapeEnum::ALL.get(index).copied())
}

/// `(TopAbs_Orientation)value`.
pub(crate) fn orientation_from_raw(value: i32) -> Result<Orientation, Failure> {
    usize::try_from(value)
        .ok()
        .and_then(|index| Orientation::ALL.get(index).copied())
        .ok_or_else(|| out_of_enum("TopAbs_Orientation", value, 3))
}

/// `(GeomAbs_Shape)value`.
pub(crate) fn continuity_from_raw(value: i32) -> Result<Continuity, Failure> {
    const ALL: [Continuity; 7] = [
        Continuity::C0,
        Continuity::G1,
        Continuity::C1,
        Continuity::G2,
        Continuity::C2,
        Continuity::C3,
        Continuity::CN,
    ];
    usize::try_from(value)
        .ok()
        .and_then(|index| ALL.get(index).copied())
        .ok_or_else(|| out_of_enum("GeomAbs_Shape", value, 6))
}

fn out_of_enum(enumeration: &str, value: i32, last: i32) -> Failure {
    Failure::out_of_range(&format!(
        "BinTools: {enumeration} value {value} read from the stream; expected 0..={last}"
    ))
}
