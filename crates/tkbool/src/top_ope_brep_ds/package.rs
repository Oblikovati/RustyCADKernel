// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The package functions of TopOpeBRepDS: the short names of states, kinds, shape
//! types, orientations and configurations, and the kind/shape type conversions.
use super::{TopOpeBRepDSConfig, TopOpeBRepDSKind};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use std::io::{self, Write};

/// `SPrint(TopAbs_State)`: "IN", "OU", "ON" or "UN"; `sprint_state(State::Out) == "OU"`.
pub fn sprint_state(s: State) -> &'static str {
    match s {
        State::In => "IN",
        State::Out => "OU",
        State::On => "ON",
        State::Unknown => "UN",
    }
}

/// `Print(TopAbs_State, OS)`; `print_state(State::In, &mut out)?`.
pub fn print_state(s: State, os: &mut impl Write) -> io::Result<()> {
    os.write_all(sprint_state(s).as_bytes())
}

/// `SPrint(TopOpeBRepDS_Kind)`: the two-letter name, empty for UNKNOWN;
/// `sprint_kind(TopOpeBRepDSKind::Edge) == "ED"`.
pub fn sprint_kind(k: TopOpeBRepDSKind) -> &'static str {
    const NAMES: [&str; 12] = [
        "PO", "CU", "SU", "VE", "ED", "WI", "FA", "SH", "SO", "CS", "CO", "",
    ];
    NAMES[k as usize]
}

/// `SPrint(K, I, B, A)`: `B(<K>,<I>)A`, a one-digit index padded by a space;
/// `sprint_kind_index(TopOpeBRepDSKind::Face, 3, "", "") == "(FA, 3)"`.
pub fn sprint_kind_index(k: TopOpeBRepDSKind, i: i32, b: &str, a: &str) -> String {
    let si = if (0..=9).contains(&i) { " " } else { "" };
    format!("{b}({},{si}{i}){a}", sprint_kind(k))
}

/// `Print(TopOpeBRepDS_Kind, S)`; `print_kind(TopOpeBRepDSKind::Point, &mut out)?`.
pub fn print_kind(k: TopOpeBRepDSKind, s: &mut impl Write) -> io::Result<()> {
    s.write_all(sprint_kind(k).as_bytes())
}

/// `Print(K, I, S, B, A)` (flushed); `print_kind_index(TopOpeBRepDSKind::Edge, 12, &mut out, "", "")?`.
pub fn print_kind_index(
    k: TopOpeBRepDSKind,
    i: i32,
    s: &mut impl Write,
    b: &str,
    a: &str,
) -> io::Result<()> {
    s.write_all(sprint_kind_index(k, i, b, a).as_bytes())?;
    s.flush()
}

/// `SPrint(TopAbs_ShapeEnum)`: the name of its kind; Standard_ProgramError for SHAPE;
/// `sprint_shape_type(ShapeEnum::Face)? == "FA"`.
pub fn sprint_shape_type(t: ShapeEnum) -> Result<&'static str, Failure> {
    Ok(sprint_kind(shape_to_kind(t)?))
}

/// `SPrint(TopAbs_ShapeEnum, I)`: `(<kind>,<I>)`; `sprint_shape_type_index(ShapeEnum::Edge, 2)?`.
pub fn sprint_shape_type_index(t: ShapeEnum, i: i32) -> Result<String, Failure> {
    Ok(sprint_kind_index(shape_to_kind(t)?, i, "", ""))
}

/// `Print(TopAbs_ShapeEnum, I, S)`; `print_shape_type_index(ShapeEnum::Edge, 2, &mut out)?`.
pub fn print_shape_type_index(t: ShapeEnum, i: i32, s: &mut impl Write) -> Result<(), Failure> {
    let text = sprint_shape_type_index(t, i)?;
    s.write_all(text.as_bytes())
        .map_err(|e| Failure::new(&format!("TopOpeBRepDS::Print: {e}")))
}

/// `SPrint(TopAbs_Orientation)`: `TopAbs::ShapeOrientationToString`;
/// `sprint_orientation(Orientation::Forward) == "FORWARD"`.
pub fn sprint_orientation(o: Orientation) -> &'static str {
    top_abs::shape_orientation_to_string(o)
}

/// `SPrint(TopOpeBRepDS_Config)`: "UNSH", "SAME" or "DIFF";
/// `sprint_config(TopOpeBRepDSConfig::SameOriented) == "SAME"`.
pub fn sprint_config(c: TopOpeBRepDSConfig) -> &'static str {
    match c {
        TopOpeBRepDSConfig::UnshGeometry => "UNSH",
        TopOpeBRepDSConfig::SameOriented => "SAME",
        TopOpeBRepDSConfig::DiffOriented => "DIFF",
    }
}

/// `Print(TopOpeBRepDS_Config, S)`; `print_config(TopOpeBRepDSConfig::DiffOriented, &mut out)?`.
pub fn print_config(c: TopOpeBRepDSConfig, s: &mut impl Write) -> io::Result<()> {
    s.write_all(sprint_config(c).as_bytes())
}

/// `IsTopology(K)`: a shape kind (VERTEX to COMPOUND); `is_topology(TopOpeBRepDSKind::Edge)`.
pub fn is_topology(k: TopOpeBRepDSKind) -> bool {
    matches!(
        k,
        TopOpeBRepDSKind::Compound
            | TopOpeBRepDSKind::CompSolid
            | TopOpeBRepDSKind::Solid
            | TopOpeBRepDSKind::Shell
            | TopOpeBRepDSKind::Face
            | TopOpeBRepDSKind::Wire
            | TopOpeBRepDSKind::Edge
            | TopOpeBRepDSKind::Vertex
    )
}

/// `IsGeometry(K)`: POINT, CURVE or SURFACE; `is_geometry(TopOpeBRepDSKind::Curve)`.
pub fn is_geometry(k: TopOpeBRepDSKind) -> bool {
    matches!(
        k,
        TopOpeBRepDSKind::Surface | TopOpeBRepDSKind::Curve | TopOpeBRepDSKind::Point
    )
}

/// `ShapeToKind(S)`: the kind of a shape type; Standard_ProgramError for SHAPE;
/// `shape_to_kind(ShapeEnum::Face)? == TopOpeBRepDSKind::Face`.
pub fn shape_to_kind(s: ShapeEnum) -> Result<TopOpeBRepDSKind, Failure> {
    Ok(match s {
        ShapeEnum::Vertex => TopOpeBRepDSKind::Vertex,
        ShapeEnum::Edge => TopOpeBRepDSKind::Edge,
        ShapeEnum::Wire => TopOpeBRepDSKind::Wire,
        ShapeEnum::Face => TopOpeBRepDSKind::Face,
        ShapeEnum::Shell => TopOpeBRepDSKind::Shell,
        ShapeEnum::Solid => TopOpeBRepDSKind::Solid,
        ShapeEnum::CompSolid => TopOpeBRepDSKind::CompSolid,
        ShapeEnum::Compound => TopOpeBRepDSKind::Compound,
        ShapeEnum::Shape => {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "TopOpeBRepDS::ShapeToKind: SHAPE; expected a typed shape (VERTEX to COMPOUND)",
                "TopOpeBRepDS",
            ));
        }
    })
}

/// `KindToShape(K)`: the shape type of a topology kind, SHAPE for a geometry or
/// UNKNOWN; `kind_to_shape(TopOpeBRepDSKind::Edge) == ShapeEnum::Edge`.
pub fn kind_to_shape(k: TopOpeBRepDSKind) -> ShapeEnum {
    match k {
        TopOpeBRepDSKind::Vertex => ShapeEnum::Vertex,
        TopOpeBRepDSKind::Edge => ShapeEnum::Edge,
        TopOpeBRepDSKind::Wire => ShapeEnum::Wire,
        TopOpeBRepDSKind::Face => ShapeEnum::Face,
        TopOpeBRepDSKind::Shell => ShapeEnum::Shell,
        TopOpeBRepDSKind::Solid => ShapeEnum::Solid,
        TopOpeBRepDSKind::CompSolid => ShapeEnum::CompSolid,
        TopOpeBRepDSKind::Compound => ShapeEnum::Compound,
        _ => ShapeEnum::Shape,
    }
}
