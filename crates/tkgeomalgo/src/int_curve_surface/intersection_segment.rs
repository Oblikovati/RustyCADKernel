// Rust translation of IntCurveSurface_IntersectionSegment.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_IntersectionSegment`: a curve segment lying on the surface.
use super::IntersectionPoint;
use std::fmt;

/// `IntCurveSurface_IntersectionSegment`: the curve segment `(w1, w2)` where the
/// distance to the surface stays below the tolerance, given by its two end points.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::{IntersectionPoint, IntersectionSegment};
/// let s = IntersectionSegment::new(IntersectionPoint::default(), IntersectionPoint::default());
/// assert_eq!(s.first_point().w(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct IntersectionSegment {
    my_p1: IntersectionPoint,
    my_p2: IntersectionPoint,
}

impl IntersectionSegment {
    /// `IntCurveSurface_IntersectionSegment(P1, P2)`; `IntersectionSegment::new(p1, p2)`.
    pub fn new(p1: IntersectionPoint, p2: IntersectionPoint) -> Self {
        Self {
            my_p1: p1,
            my_p2: p2,
        }
    }
    /// `SetValues(P1, P2)`; `segment.set_values(p1, p2)`.
    pub fn set_values(&mut self, p1: IntersectionPoint, p2: IntersectionPoint) {
        (self.my_p1, self.my_p2) = (p1, p2);
    }
    /// `Values(P1, P2)`: both end points; `let (p1, p2) = segment.values();`.
    pub fn values(&self) -> (IntersectionPoint, IntersectionPoint) {
        (self.my_p1, self.my_p2)
    }
    /// `FirstPoint()` and `FirstPoint(P1)`: the first end point; `segment.first_point()`.
    pub fn first_point(&self) -> IntersectionPoint {
        self.my_p1
    }
    /// `SecondPoint()` and `SecondPoint(P2)`: the second end point;
    /// `segment.second_point()`.
    pub fn second_point(&self) -> IntersectionPoint {
        self.my_p2
    }
    /// `Dump()`: a title, both points and a blank line; `segment.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        write!(out, "\nIntersectionSegment : \n")?;
        self.my_p1.dump(out)?;
        self.my_p2.dump(out)?;
        writeln!(out)
    }
}
