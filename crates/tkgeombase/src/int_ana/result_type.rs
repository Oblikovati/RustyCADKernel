// Rust translation of IntAna_ResultType.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_ResultType: the kind of a geometric quadric-quadric intersection.

/// IntAna_ResultType: the kind of result of [`super::QuadQuadGeo`]. The
/// discriminants are the native enumerator values.
/// `assert_eq!(ResultType::Empty as i32, 7)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum ResultType {
    /// IntAna_Point: one or two points.
    Point = 0,
    /// IntAna_Line: one or two lines.
    Line = 1,
    /// IntAna_Circle: one to four circles.
    Circle = 2,
    /// IntAna_PointAndCircle: a point and a circle.
    PointAndCircle = 3,
    /// IntAna_Ellipse: one or two ellipses.
    Ellipse = 4,
    /// IntAna_Parabola: one parabola.
    Parabola = 5,
    /// IntAna_Hyperbola: the two branches of a hyperbola.
    Hyperbola = 6,
    /// IntAna_Empty: no intersection (the value of a new QuadQuadGeo).
    #[default]
    Empty = 7,
    /// IntAna_Same: the quadrics are identical.
    Same = 8,
    /// IntAna_NoGeometricSolution: an analytic algorithm must compute the result.
    NoGeometricSolution = 9,
}

impl ResultType {
    /// Every enumerator in native order; `ResultType::ALL[9] == ResultType::NoGeometricSolution`.
    pub const ALL: [Self; 10] = [
        Self::Point,
        Self::Line,
        Self::Circle,
        Self::PointAndCircle,
        Self::Ellipse,
        Self::Parabola,
        Self::Hyperbola,
        Self::Empty,
        Self::Same,
        Self::NoGeometricSolution,
    ];

    /// The native enumerator name; `ResultType::Same.name() == "IntAna_Same"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Point => "IntAna_Point",
            Self::Line => "IntAna_Line",
            Self::Circle => "IntAna_Circle",
            Self::PointAndCircle => "IntAna_PointAndCircle",
            Self::Ellipse => "IntAna_Ellipse",
            Self::Parabola => "IntAna_Parabola",
            Self::Hyperbola => "IntAna_Hyperbola",
            Self::Empty => "IntAna_Empty",
            Self::Same => "IntAna_Same",
            Self::NoGeometricSolution => "IntAna_NoGeometricSolution",
        }
    }
}
