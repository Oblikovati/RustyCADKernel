// Rust translation of ModelingAlgorithms/TKGeomAlgo/TopTrans/TopTrans_CurveTransition.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TopTrans_CurveTransition: the complex transition of a curve crossing several
//! boundary curves at one point.
use super::surface_transition::{get_after, get_before};
use rustycad_tkg3d::top_abs::{self, Orientation, State};
use rustycad_tkmath::gp::Direction3;

/// The result of comparing two angle cosines at a tolerance (the GREATER, SAME and
/// LOWER macros of the native file).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AngleOrder {
    Greater,
    Same,
    Lower,
}

/// TopTrans_CurveTransition: accumulates the boundary curves met by an intersecting
/// curve at one point and keeps the first and the last of them around the point, from
/// which the states of the curve before and after the point follow.
///
/// ```
/// use rustycad_tkgeomalgo::toptrans::CurveTransition;
/// use rustycad_tkg3d::top_abs::{Orientation, State};
/// use rustycad_tkmath::gp::Direction3;
/// let mut transition = CurveTransition::new();
/// transition.reset_line(Direction3::X);
/// transition.compare(1e-7, Direction3::Y, Direction3::Z, 0.0, Orientation::Forward, Orientation::Forward);
/// assert_eq!((transition.state_before(), transition.state_after()), (State::Out, State::In));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CurveTransition {
    tgt: Direction3,
    norm: Direction3,
    curv: f64,
    init: bool,
    tgt_first: Direction3,
    norm_first: Direction3,
    curv_first: f64,
    tran_first: Orientation,
    tgt_last: Direction3,
    norm_last: Direction3,
    curv_last: f64,
    tran_last: Orientation,
}

impl Default for CurveTransition {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveTransition {
    /// TopTrans_CurveTransition(): an empty transition; its states are UNKNOWN until
    /// the first comparison after a reset. The directions are the default `gp_Dir`
    /// (+X); `CurveTransition::new()`.
    pub fn new() -> Self {
        Self {
            tgt: Direction3::default(),
            norm: Direction3::default(),
            curv: 0.0,
            init: false,
            tgt_first: Direction3::default(),
            norm_first: Direction3::default(),
            curv_first: 0.0,
            tran_first: Orientation::Forward,
            tgt_last: Direction3::default(),
            norm_last: Direction3::default(),
            curv_last: 0.0,
            tran_last: Orientation::Forward,
        }
    }

    /// Reset(Tgt, Norm, Curv): starts a transition for a curve with tangent `tgt`,
    /// normal `norm` and curvature `curv`; `transition.reset(t, n, 0.5)`.
    pub fn reset(&mut self, tgt: Direction3, norm: Direction3, curv: f64) {
        self.tgt = tgt;
        self.norm = norm;
        self.curv = curv;
        self.init = true;
    }

    /// Reset(Tgt): starts a transition for a straight line; the normal keeps its
    /// previous value, as natively; `transition.reset_line(Direction3::X)`.
    pub fn reset_line(&mut self, tgt: Direction3) {
        self.tgt = tgt;
        self.curv = 0.0;
        self.init = true;
    }

    /// Compare: adds the boundary curve with tangent `tang`, normal `norm` and
    /// curvature `curv` at tolerance `tole`. `transition` says how the curve crosses
    /// the boundary; `orientation` places the point on the boundary curve (REVERSED:
    /// the curve ends there, FORWARD: it starts there, INTERNAL: it passes through);
    /// `transition.compare(1e-7, t, n, 0.0, Orientation::Forward, Orientation::Forward)`.
    pub fn compare(
        &mut self,
        tole: f64,
        tang: Direction3,
        norm: Direction3,
        curv: f64,
        transition: Orientation,
        orientation: Orientation,
    ) {
        let side = self.adjusted_transition(tang, transition, orientation);
        let element = BoundaryElement {
            tang,
            norm,
            curv,
            side,
            orientation,
        };
        if self.init {
            self.start_with(element);
        } else {
            self.insert(tole, element);
        }
    }

    /// StateBefore: the state of the curve just before the point, UNKNOWN before the
    /// first comparison; `transition.state_before()`.
    pub fn state_before(&self) -> State {
        if self.init {
            return State::Unknown;
        }
        get_before(self.tran_first)
    }

    /// StateAfter: the state of the curve just after the point, UNKNOWN before the
    /// first comparison; `transition.state_after()`.
    pub fn state_after(&self) -> State {
        if self.init {
            return State::Unknown;
        }
        get_after(self.tran_last)
    }

    /// An INTERNAL transition takes the side of the orientation, reversed when the
    /// boundary tangent opposes the reference tangent.
    fn adjusted_transition(
        &self,
        tang: Direction3,
        transition: Orientation,
        orientation: Orientation,
    ) -> Orientation {
        if transition != Orientation::Internal {
            return transition;
        }
        if tang.dot(self.tgt) < 0.0 {
            top_abs::reverse(orientation)
        } else {
            orientation
        }
    }

    /// The first comparison after a reset: the element is both the first and the last.
    fn start_with(&mut self, element: BoundaryElement) {
        self.init = false;
        self.set_first(element, element.tang);
        self.set_last(element, element.tang);
        match element.orientation {
            // The point ends the boundary curve: its tangent is reversed.
            Orientation::Reversed => {
                self.tgt_first.reverse();
                self.tgt_last.reverse();
            }
            // The point is inside the boundary curve: one side is reversed, chosen by
            // the reference tangent.
            Orientation::Internal if self.tgt.dot(element.tang) > 0.0 => self.tgt_first.reverse(),
            Orientation::Internal => self.tgt_last.reverse(),
            Orientation::Forward | Orientation::External => {}
        }
    }

    /// A later comparison: the element may replace the first, then the last.
    fn insert(&mut self, tole: f64, element: BoundaryElement) {
        let mut cos_with_t: f64 = self.tgt.dot(element.tang);
        match element.orientation {
            Orientation::Reversed => cos_with_t = -cos_with_t,
            Orientation::Internal if cos_with_t > 0.0 => cos_with_t = -cos_with_t,
            _ => {}
        }
        let first_set = self.update_first(tole, cos_with_t, element);
        if first_set && element.orientation != Orientation::Internal {
            return;
        }
        // In tangent cases the first can also be the last.
        if element.orientation == Orientation::Internal {
            cos_with_t = -cos_with_t;
        }
        self.update_last(tole, cos_with_t, element);
    }

    /// Replaces the first element when the new one comes before it; returns whether
    /// it did (FirstSet).
    fn update_first(&mut self, tole: f64, cos_with_t: f64, element: BoundaryElement) -> bool {
        let cos_with_first: f64 = self.tgt.dot(self.tgt_first);
        let replace = match compare_angles(cos_with_t, cos_with_first, tole) {
            AngleOrder::Lower => true,
            AngleOrder::Same => self.is_before(
                tole,
                cos_with_t,
                element.norm,
                element.curv,
                self.norm_first,
                self.curv_first,
            ),
            AngleOrder::Greater => false,
        };
        if replace {
            let reversed = match element.orientation {
                Orientation::Reversed => true,
                Orientation::Internal => self.tgt.dot(element.tang) > 0.0,
                _ => false,
            };
            self.set_first(element, oriented(element.tang, reversed));
        }
        replace
    }

    /// Replaces the last element when the new one comes after it.
    fn update_last(&mut self, tole: f64, cos_with_t: f64, element: BoundaryElement) {
        let cos_with_last: f64 = self.tgt.dot(self.tgt_last);
        let replace = match compare_angles(cos_with_t, cos_with_last, tole) {
            AngleOrder::Greater => true,
            AngleOrder::Same => self.is_before(
                tole,
                cos_with_t,
                self.norm_last,
                self.curv_last,
                element.norm,
                element.curv,
            ),
            AngleOrder::Lower => false,
        };
        if !replace {
            return;
        }
        let reversed = match element.orientation {
            Orientation::Reversed => true,
            Orientation::Internal => self.tgt.dot(element.tang) < 0.0,
            _ => false,
        };
        self.set_last(element, oriented(element.tang, reversed));
    }

    fn set_first(&mut self, element: BoundaryElement, tangent: Direction3) {
        self.tgt_first = tangent;
        self.norm_first = element.norm;
        self.curv_first = element.curv;
        self.tran_first = element.side;
    }

    fn set_last(&mut self, element: BoundaryElement, tangent: Direction3) {
        self.tgt_last = tangent;
        self.norm_last = element.norm;
        self.curv_last = element.curv;
        self.tran_last = element.side;
    }

    /// IsBefore: whether the element `(n1, c1)` comes before `(n2, c2)` along the
    /// reference curve when their tangent angles are the same.
    fn is_before(
        &self,
        tole: f64,
        cos_angle: f64,
        n1: Direction3,
        c1: f64,
        n2: Direction3,
        c2: f64,
    ) -> bool {
        let tn1: f64 = self.tgt.dot(n1);
        let tn2: f64 = self.tgt.dot(n2);
        if tn1.abs() <= tole || tn2.abs() <= tole {
            return self.is_before_tangent(cos_angle, n1, c1, n2, c2);
        }
        if tn1 < 0.0 {
            // Before the first we are in the curvature: it comes first when the second
            // is out of it /* ->)( */, or when it has the greater curvature /* ->)) */.
            return tn2 > 0.0 || c1 > c2;
        }
        // Before the first we are out of the curvature: with both out /* ->(( */ the
        // lower curvature comes first. NaN normals reach neither branch natively.
        tn1 > 0.0 && tn2 > 0.0 && c1 < c2
    }

    /// The tangent case of IsBefore: the element nearest the reference curvature
    /// comes first, inverted when the angle cosine is positive (the 2000 fix by
    /// Sergey Khromov applies it to the straight reference too).
    fn is_before_tangent(
        &self,
        cos_angle: f64,
        n1: Direction3,
        c1: f64,
        n2: Direction3,
        c2: f64,
    ) -> bool {
        let one_before = if self.curv == 0.0 {
            // The reference is straight: the lowest curvature comes first.
            c1 < c2
        } else {
            self.curvature_gap(n1, c1) < self.curvature_gap(n2, c2)
        };
        if cos_angle > 0.0 {
            !one_before
        } else {
            one_before
        }
    }

    /// The curvature of an element relative to the reference, signed by the side of
    /// its normal unless one of the curves is straight.
    fn curvature_gap(&self, normal: Direction3, curv: f64) -> f64 {
        if curv == 0.0 || self.curv == 0.0 {
            curv - self.curv
        } else {
            (curv - self.curv) * normal.dot(self.norm)
        }
    }
}

/// One boundary curve passed to Compare, with its transition adjusted for INTERNAL.
#[derive(Clone, Copy, Debug)]
struct BoundaryElement {
    tang: Direction3,
    norm: Direction3,
    curv: f64,
    side: Orientation,
    orientation: Orientation,
}

/// `tangent`, reversed when `reversed` holds.
fn oriented(tangent: Direction3, reversed: bool) -> Direction3 {
    if reversed {
        tangent.reversed()
    } else {
        tangent
    }
}

/// Compare(Ang1, Ang2, Tole): orders two angle cosines at tolerance `tole`.
fn compare_angles(ang1: f64, ang2: f64, tole: f64) -> AngleOrder {
    if ang1 - ang2 > tole {
        AngleOrder::Greater
    } else if ang2 - ang1 > tole {
        AngleOrder::Lower
    } else {
        AngleOrder::Same
    }
}
