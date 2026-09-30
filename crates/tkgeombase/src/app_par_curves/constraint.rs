// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_Constraint.hxx, AppParCurves_ConstraintCouple.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_Constraint` and `AppParCurves_ConstraintCouple`: the constraint of
//! one point of a MultiLine.

/// `AppParCurves_Constraint`; the order matters (the algorithms compare
/// `Cons >= AppParCurves_TangencyPoint`); `Constraint::PassPoint < Constraint::TangencyPoint`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Constraint {
    /// `AppParCurves_NoConstraint`.
    #[default]
    NoConstraint = 0,
    /// `AppParCurves_PassPoint`: the curve passes through the point.
    PassPoint = 1,
    /// `AppParCurves_TangencyPoint`: passes through it with the given tangent.
    TangencyPoint = 2,
    /// `AppParCurves_CurvaturePoint`: passes with the given tangent and curvature.
    CurvaturePoint = 3,
}

impl Constraint {
    /// The native enumeration value; `Constraint::TangencyPoint.code() == 2`.
    pub fn code(self) -> i32 {
        self as i32
    }

    /// The constraint of a native value (0 to 3), None otherwise;
    /// `Constraint::from_code(1) == Some(Constraint::PassPoint)`.
    pub fn from_code(code: i32) -> Option<Self> {
        [
            Self::NoConstraint,
            Self::PassPoint,
            Self::TangencyPoint,
            Self::CurvaturePoint,
        ]
        .get(usize::try_from(code).ok()?)
        .copied()
    }
}

/// `AppParCurves_ConstraintCouple`: a point index and its constraint.
///
/// ```
/// use rustycad_tkgeombase::app_par_curves::{Constraint, ConstraintCouple};
/// let mut couple = ConstraintCouple::default();
/// assert_eq!((couple.index(), couple.constraint()), (-1, Constraint::NoConstraint));
/// couple.set_constraint(Constraint::PassPoint);
/// assert_eq!(ConstraintCouple::new(3, Constraint::PassPoint).index(), 3);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstraintCouple {
    index: i32,
    constraint: Constraint,
}

impl Default for ConstraintCouple {
    /// `AppParCurves_ConstraintCouple()`: index -1, no constraint.
    fn default() -> Self {
        Self {
            index: -1,
            constraint: Constraint::NoConstraint,
        }
    }
}

impl ConstraintCouple {
    /// `AppParCurves_ConstraintCouple(TheIndex, Cons)`; `ConstraintCouple::new(1, Constraint::PassPoint)`.
    pub fn new(index: i32, constraint: Constraint) -> Self {
        Self { index, constraint }
    }

    /// `Index()`; `couple.index()`.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// `Constraint()`; `couple.constraint()`.
    pub fn constraint(&self) -> Constraint {
        self.constraint
    }

    /// `SetIndex(TheIndex)`; `couple.set_index(2)`.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }

    /// `SetConstraint(Cons)`; `couple.set_constraint(Constraint::PassPoint)`.
    pub fn set_constraint(&mut self, constraint: Constraint) {
        self.constraint = constraint;
    }
}
