// Rust translation of ModelingAlgorithms/TKGeomAlgo/GccEnt (GccEnt.hxx/.cxx, GccEnt_Position.hxx,
// GccEnt_BadQualifier.hxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package GccEnt: qualified lines and circles, the arguments of the 2d
//! constraint solvers of GccAna and Geom2dGcc.
//!
//! The qualifier tells which subfamily of solutions a solver keeps: the solution
//! encloses the argument, is enclosed by it, is outside it, or is unqualified (all
//! solutions). The interior of a line is its left-hand side.
mod qualified_circ;
mod qualified_lin;
pub use qualified_circ::QualifiedCirc;
pub use qualified_lin::QualifiedLin;

use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::{Circle2, Line2};
use std::io::{self, Write};

/// GccEnt_Position: the position of a solution relative to one argument of a
/// construction algorithm; `Position::Enclosing as i32 == 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Position {
    /// GccEnt_unqualified: the position is undefined, every solution applies.
    #[default]
    Unqualified,
    /// GccEnt_enclosing: the solution encloses the argument.
    Enclosing,
    /// GccEnt_enclosed: the solution is enclosed by the argument.
    Enclosed,
    /// GccEnt_outside: the solution and the argument are outside each other.
    Outside,
    /// GccEnt_noqualifier.
    NoQualifier,
}

impl Position {
    /// Every position in native enumeration order.
    pub const ALL: [Position; 5] = [
        Position::Unqualified,
        Position::Enclosing,
        Position::Enclosed,
        Position::Outside,
        Position::NoQualifier,
    ];
}

// GccEnt_Table_PrintPosition of GccEnt.cxx.
const POSITION_NAMES: [&str; 5] = [
    "UNQUALIFIED",
    "ENCLOSING",
    "ENCLOSED",
    "OUTSIDE",
    "NOQUALIFIER",
];

/// `GccEnt_BadQualifier` (a Standard_DomainError natively), raised by the solvers
/// when an argument has a qualifier they do not accept;
/// `bad_qualifier("GccAna_Lin2d2Tan: Qualified1 is ENCLOSING; expected ENCLOSED, OUTSIDE or UNQUALIFIED")`.
pub fn bad_qualifier(message: &str) -> Failure {
    Failure::with_type("GccEnt_BadQualifier", message, "")
}

/// GccEnt::PositionToString: the upper-case name of a position;
/// `position_to_string(Position::Outside) == "OUTSIDE"`.
pub fn position_to_string(position: Position) -> &'static str {
    POSITION_NAMES[position as usize]
}

/// GccEnt::PositionFromString(const char*, GccEnt_Position&): the position named
/// `text`, compared case-insensitively, or `None` when the name is unknown (native
/// then leaves its output unchanged and returns false);
/// `try_position_from_string("enclosed") == Some(Position::Enclosed)`.
pub fn try_position_from_string(text: &str) -> Option<Position> {
    // `TCollection_AsciiString aName(text); aName.UpperCase();` as native compares.
    let mut name: AsciiString = AsciiString::new(text);
    name.upper_case();
    Position::ALL
        .into_iter()
        .find(|&position| name == position_to_string(position))
}

/// GccEnt::PositionFromString(const char*): the position named `text`, or
/// Unqualified when the name is unknown;
/// `position_from_string("Outside") == Position::Outside`.
pub fn position_from_string(text: &str) -> Position {
    try_position_from_string(text).unwrap_or(Position::Unqualified)
}

/// GccEnt::Print: write the name of `position` to `stream`;
/// `print(Position::Enclosing, &mut out)?`.
pub fn print(position: Position, stream: &mut impl Write) -> io::Result<()> {
    stream.write_all(position_to_string(position).as_bytes())
}

/// GccEnt::Unqualified(gp_Lin2d): the line with no qualifier (all solutions apply);
/// `unqualified_lin(line).is_unqualified()`.
pub fn unqualified_lin(line: Line2) -> QualifiedLin {
    QualifiedLin::new(line, Position::Unqualified)
}

/// GccEnt::Unqualified(gp_Circ2d): the circle with no qualifier;
/// `unqualified_circ(circle).is_unqualified()`.
pub fn unqualified_circ(circle: Circle2) -> QualifiedCirc {
    QualifiedCirc::new(circle, Position::Unqualified)
}

/// GccEnt::Enclosing(gp_Circ2d): the solution must enclose the circle;
/// `enclosing(circle).is_enclosing()`.
pub fn enclosing(circle: Circle2) -> QualifiedCirc {
    QualifiedCirc::new(circle, Position::Enclosing)
}

/// GccEnt::Enclosed(gp_Lin2d): the solution must lie on the left of the line;
/// `enclosed_lin(line).is_enclosed()`.
pub fn enclosed_lin(line: Line2) -> QualifiedLin {
    QualifiedLin::new(line, Position::Enclosed)
}

/// GccEnt::Enclosed(gp_Circ2d): the solution must be enclosed by the circle;
/// `enclosed_circ(circle).is_enclosed()`.
pub fn enclosed_circ(circle: Circle2) -> QualifiedCirc {
    QualifiedCirc::new(circle, Position::Enclosed)
}

/// GccEnt::Outside(gp_Lin2d): the solution must lie on the right of the line;
/// `outside_lin(line).is_outside()`.
pub fn outside_lin(line: Line2) -> QualifiedLin {
    QualifiedLin::new(line, Position::Outside)
}

/// GccEnt::Outside(gp_Circ2d): the solution and the circle must be outside each
/// other; `outside_circ(circle).is_outside()`.
pub fn outside_circ(circle: Circle2) -> QualifiedCirc {
    QualifiedCirc::new(circle, Position::Outside)
}
