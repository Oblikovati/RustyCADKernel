// Rust translation of GccEnt_QualifiedLin.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccEnt_QualifiedLin: a 2d line with a qualifier.
use super::Position;
use rustycad_tkmath::gp::Line2;

/// GccEnt_QualifiedLin: a line argument of a construction algorithm and the side
/// its solutions must lie on (the interior is the left-hand side).
/// ```
/// use rustycad_tkgeomalgo::gcc_ent::{Position, QualifiedLin};
/// use rustycad_tkmath::gp::Line2;
/// let qualified = QualifiedLin::new(Line2::default(), Position::Enclosed);
/// assert!(qualified.is_enclosed());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QualifiedLin {
    the_qualifier: Position,
    the_qualified: Line2,
}

impl QualifiedLin {
    /// GccEnt_QualifiedLin(Qualified, Qualifier). Any position is stored, even
    /// Enclosing, which the solvers then reject with GccEnt_BadQualifier;
    /// `QualifiedLin::new(line, Position::Outside)`.
    pub fn new(qualified: Line2, qualifier: Position) -> Self {
        Self {
            the_qualifier: qualifier,
            the_qualified: qualified,
        }
    }

    /// Qualified: the line; `qualified.qualified().direction()`.
    pub fn qualified(&self) -> Line2 {
        self.the_qualified
    }

    /// Qualifier: the stored position (GccEnt_unqualified, not the documented
    /// GccEnt_noqualifier, for an unqualified line); `qualified.qualifier()`.
    pub fn qualifier(&self) -> Position {
        self.the_qualifier
    }

    /// IsUnqualified: true when all solutions apply; `qualified.is_unqualified()`.
    pub fn is_unqualified(&self) -> bool {
        self.the_qualifier == Position::Unqualified
    }

    /// IsEnclosed: the solution lies on the left of the line; `qualified.is_enclosed()`.
    pub fn is_enclosed(&self) -> bool {
        self.the_qualifier == Position::Enclosed
    }

    /// IsOutside: the solution lies on the right of the line; `qualified.is_outside()`.
    pub fn is_outside(&self) -> bool {
        self.the_qualifier == Position::Outside
    }
}
