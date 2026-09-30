// Rust translation of IntRes2d_IntersectionPoint.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntRes2d_IntersectionPoint`: one intersection point of two 2D curves.
use super::Transition;
use rustycad_tkmath::gp::Point2;

/// `IntRes2d_IntersectionPoint`: the point, its parameter on each curve and the
/// transition of each curve relative to the other.
///
/// ```
/// use rustycad_tkgeomalgo::int_res2d::{IntersectionPoint, Transition};
/// use rustycad_tkmath::gp::Point2;
/// let t = Transition::new();
/// let p = IntersectionPoint::new(Point2::new(1.0, 2.0), 0.5, 0.25, t, t, true);
/// assert_eq!(p.param_on_first(), 0.25);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntersectionPoint {
    pt: Point2,
    p1: f64,
    p2: f64,
    trans1: Transition,
    trans2: Transition,
}

impl Default for IntersectionPoint {
    /// `IntRes2d_IntersectionPoint()`: the origin, both parameters at RealLast().
    /// `IntersectionPoint::default().param_on_first() == f64::MAX`.
    fn default() -> Self {
        Self {
            pt: Point2::new(0.0, 0.0),
            p1: f64::MAX,
            p2: f64::MAX,
            trans1: Transition::default(),
            trans2: Transition::default(),
        }
    }
}

impl IntersectionPoint {
    /// `IntRes2d_IntersectionPoint(P, Uc1, Uc2, Trans1, Trans2, ReversedFlag)`: when
    /// `reversed_flag` is set, `uc1`/`trans1` belong to the second curve and
    /// `uc2`/`trans2` to the first one;
    /// `IntersectionPoint::new(point, 0.5, 0.25, t1, t2, false)`.
    pub fn new(
        p: Point2,
        uc1: f64,
        uc2: f64,
        trans1: Transition,
        trans2: Transition,
        reversed_flag: bool,
    ) -> Self {
        let mut point: Self = Self::default();
        point.set_values(p, uc1, uc2, trans1, trans2, reversed_flag);
        point
    }
    /// `SetValues(P, Uc1, Uc2, Trans1, Trans2, ReversedFlag)`: the values of
    /// [`IntersectionPoint::new`]; `point.set_values(p, 0.5, 0.25, t1, t2, false)`.
    pub fn set_values(
        &mut self,
        p: Point2,
        uc1: f64,
        uc2: f64,
        trans1: Transition,
        trans2: Transition,
        reversed_flag: bool,
    ) {
        self.pt = p;
        if reversed_flag {
            self.trans1 = trans2;
            self.trans2 = trans1;
            self.p1 = uc2;
            self.p2 = uc1;
        } else {
            self.trans1 = trans1;
            self.trans2 = trans2;
            self.p1 = uc1;
            self.p2 = uc2;
        }
    }
    /// `Value()`: the point in the plane. `point.value()`.
    pub fn value(&self) -> Point2 {
        self.pt
    }
    /// `ParamOnFirst()`: the parameter on the first curve. `point.param_on_first()`.
    pub fn param_on_first(&self) -> f64 {
        self.p1
    }
    /// `ParamOnSecond()`: the parameter on the second curve. `point.param_on_second()`.
    pub fn param_on_second(&self) -> f64 {
        self.p2
    }
    /// `TransitionOfFirst()`: the transition of the first curve relative to the
    /// second one. `point.transition_of_first()`.
    pub fn transition_of_first(&self) -> Transition {
        self.trans1
    }
    /// `TransitionOfSecond()`: the transition of the second curve relative to the
    /// first one. `point.transition_of_second()`.
    pub fn transition_of_second(&self) -> Transition {
        self.trans2
    }
}
