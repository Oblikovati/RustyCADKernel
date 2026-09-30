// Rust translation of ModelingAlgorithms/TKHLR/TopCnx/TopCnx_EdgeFaceTransition.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TopCnx_EdgeFaceTransition: the cumulated transition of an edge through the
//! boundary curves of a face at one point.
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::toptrans::CurveTransition;
use rustycad_tkmath::gp::Direction3;

/// TopCnx_EdgeFaceTransition: accumulates the interferences of an edge with the
/// boundary of a face at one point. The transition comes from a
/// TopTrans_CurveTransition over the boundary curves; the boundary transition counts
/// the FORWARD and REVERSED boundary transitions of the interferences.
///
/// ```
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkhlr::top_cnx::EdgeFaceTransition;
/// use rustycad_tkmath::gp::Direction3;
/// let mut transition = EdgeFaceTransition::new();
/// transition.reset_line(Direction3::X);
/// transition.add_interference(1e-7, Direction3::Y, Direction3::Z, 0.0,
///     Orientation::Forward, Orientation::Forward, Orientation::Forward);
/// assert_eq!(transition.transition(), Orientation::Forward);
/// assert_eq!(transition.boundary_transition(), Orientation::Forward);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeFaceTransition {
    curve_transition: CurveTransition,
    nb_bound_forward: i32,
    nb_bound_reversed: i32,
}

impl EdgeFaceTransition {
    /// TopCnx_EdgeFaceTransition(): an empty algorithm; `EdgeFaceTransition::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset(Tgt, Norm, Curv): starts with the local description of the edge (tangent,
    /// normal, curvature) and clears the boundary counts;
    /// `transition.reset(tgt, norm, 0.5)`.
    pub fn reset(&mut self, tgt: Direction3, norm: Direction3, curv: f64) {
        self.curve_transition.reset(tgt, norm, curv);
        self.clear_counts();
    }

    /// Reset(Tgt): starts with a linear edge and clears the boundary counts;
    /// `transition.reset_line(Direction3::X)`.
    pub fn reset_line(&mut self, tgt: Direction3) {
        self.curve_transition.reset_line(tgt);
        self.clear_counts();
    }

    /// AddInterference: adds a boundary curve element with tangent `tang`, normal
    /// `norm` and curvature `curv` at tolerance `tole`; `or` is the orientation of the
    /// interference on the boundary curve, `tr` its transition and `b_tr` its
    /// boundary transition (FORWARD and REVERSED are counted, INTERNAL and EXTERNAL
    /// are not); `transition.add_interference(1e-7, t, n, 0.0, or, tr, b_tr)`.
    #[allow(clippy::too_many_arguments)] // The native member takes these 7 values.
    pub fn add_interference(
        &mut self,
        tole: f64,
        tang: Direction3,
        norm: Direction3,
        curv: f64,
        or: Orientation,
        tr: Orientation,
        b_tr: Orientation,
    ) {
        self.curve_transition
            .compare(tole, tang, norm, curv, tr, or);
        match b_tr {
            Orientation::Forward => self.nb_bound_forward += 1,
            Orientation::Reversed => self.nb_bound_reversed += 1,
            Orientation::Internal | Orientation::External => {}
        }
    }

    /// Transition: the cumulated transition, from the states before and after the
    /// point (IN-IN INTERNAL, IN-OUT REVERSED, OUT-IN FORWARD, OUT-OUT EXTERNAL, and
    /// INTERNAL for any other state, which a debug build of OCCT reports);
    /// `transition.transition()`.
    pub fn transition(&self) -> Orientation {
        let before = self.curve_transition.state_before();
        let after = self.curve_transition.state_after();
        match (before, after) {
            (State::In, State::Out) => Orientation::Reversed,
            (State::Out, State::In) => Orientation::Forward,
            (State::Out, State::Out) => Orientation::External,
            _ => Orientation::Internal,
        }
    }

    /// BoundaryTransition: FORWARD or REVERSED when that count is larger, EXTERNAL
    /// when the counts are equal (the native parity test gives EXTERNAL on both
    /// branches); `transition.boundary_transition()`.
    pub fn boundary_transition(&self) -> Orientation {
        if self.nb_bound_forward > self.nb_bound_reversed {
            Orientation::Forward
        } else if self.nb_bound_forward < self.nb_bound_reversed {
            Orientation::Reversed
        } else {
            Orientation::External
        }
    }

    fn clear_counts(&mut self) {
        self.nb_bound_forward = 0;
        self.nb_bound_reversed = 0;
    }
}
