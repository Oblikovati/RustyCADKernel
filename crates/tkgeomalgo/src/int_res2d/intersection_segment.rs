// Rust translation of IntRes2d_IntersectionSegment.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntRes2d_IntersectionSegment`: a common part of two 2D curves.
use super::IntersectionPoint;
use super::failures::domain_error;
use rustycad_tkernel::standard::Failure;

/// `IntRes2d_IntersectionSegment`: an intersection curve, bounded by 0, 1 or 2
/// intersection points, with the relative orientation of the two curves on it.
///
/// ```
/// use rustycad_tkgeomalgo::int_res2d::IntersectionSegment;
/// let infinite = IntersectionSegment::infinite(true);
/// assert!(infinite.is_opposite() && !infinite.has_first_point());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct IntersectionSegment {
    oppos: bool,
    first: bool,
    last: bool,
    ptfirst: IntersectionPoint,
    ptlast: IntersectionPoint,
}

impl IntersectionSegment {
    /// `IntRes2d_IntersectionSegment()`: an unbounded segment with the same
    /// orientation on both curves. `IntersectionSegment::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IntRes2d_IntersectionSegment(P1, P2, Oppos, ReverseFlag)`: a bounded segment;
    /// the points swap when both flags are set;
    /// `IntersectionSegment::bounded(p1, p2, false, false)`.
    pub fn bounded(
        p1: IntersectionPoint,
        p2: IntersectionPoint,
        oppos: bool,
        reverse_flag: bool,
    ) -> Self {
        let (ptfirst, ptlast) = if reverse_flag && oppos {
            (p2, p1)
        } else {
            (p1, p2)
        };
        Self {
            oppos,
            first: true,
            last: true,
            ptfirst,
            ptlast,
        }
    }
    /// `IntRes2d_IntersectionSegment(P, First, Oppos, ReverseFlag)`: a half-bounded
    /// segment; `p` is its first point when `first` (inverted when both flags are
    /// set), its last point otherwise;
    /// `IntersectionSegment::half_bounded(p, true, false, false)`.
    pub fn half_bounded(
        p: IntersectionPoint,
        first: bool,
        oppos: bool,
        reverse_flag: bool,
    ) -> Self {
        let is_first: bool = if reverse_flag && oppos { !first } else { first };
        let mut segment: Self = Self::infinite(oppos);
        if is_first {
            segment.first = true;
            segment.ptfirst = p;
        } else {
            segment.last = true;
            segment.ptlast = p;
        }
        segment
    }
    /// `IntRes2d_IntersectionSegment(Oppos)`: an infinite segment;
    /// `IntersectionSegment::infinite(true)`.
    pub fn infinite(oppos: bool) -> Self {
        Self {
            oppos,
            ..Self::default()
        }
    }
    /// `IsOpposite()`: false when the segment has the same orientation on both
    /// curves. `segment.is_opposite()`.
    pub fn is_opposite(&self) -> bool {
        self.oppos
    }
    /// `HasFirstPoint()`. `segment.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        self.first
    }
    /// `FirstPoint()`; Standard_DomainError without a first point.
    /// `segment.first_point()?`.
    pub fn first_point(&self) -> Result<IntersectionPoint, Failure> {
        if !self.first {
            return Err(domain_error(
                "IntRes2d_IntersectionSegment::FirstPoint",
                "the segment has no first point; expected HasFirstPoint()",
            ));
        }
        Ok(self.ptfirst)
    }
    /// `HasLastPoint()`. `segment.has_last_point()`.
    pub fn has_last_point(&self) -> bool {
        self.last
    }
    /// `LastPoint()`; Standard_DomainError without a last point.
    /// `segment.last_point()?`.
    pub fn last_point(&self) -> Result<IntersectionPoint, Failure> {
        if !self.last {
            return Err(domain_error(
                "IntRes2d_IntersectionSegment::LastPoint",
                "the segment has no last point; expected HasLastPoint()",
            ));
        }
        Ok(self.ptlast)
    }
}
