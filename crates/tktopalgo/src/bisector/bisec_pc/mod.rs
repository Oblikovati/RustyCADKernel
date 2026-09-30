// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecPC.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_BisecPC`: the bisector of a curve and a point.
mod curve;
mod perform;
mod state;

pub(crate) use perform::copy_curve;
pub(crate) use state::{PcState, PcValues};

use super::BisectorCurve;
use crate::bisector::intervals::{cout_real, dump_intervals};
use crate::bisector::is_convex;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point2;

/// `Bisector_BisecPC`: the locus of the points equidistant from a curve and a
/// point, parameterized by the curve (shifted on a periodic curve), on the part of
/// the curve whose bisector points are closer than `DistMax`, extended by half
/// lines at an end of a concave curve that is the point (docs/topalgo-design.md
/// section 3).
///
/// `FirstParameter` and `LastParameter` read the first and last interval; they
/// raise Standard_NoSuchObject natively when there is none (an empty or a default
/// bisector), where the port, whose `Geom2d_Curve` members cannot fail, answers
/// NaN.
/// ```
/// use rustycad_tktopalgo::bisector::BisecPC;
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 2.0, true)?).cast();
/// let bisector = BisecPC::with_curve(&circle, Point2::new(0.5, 0.0), 1.0, 500.0)?;
/// assert!(!bisector.is_empty());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BisecPC {
    parent: StandardTransient,
    state: SharedState<PcState>,
}
impl_transient!(
    BisecPC,
    "Bisector_BisecPC",
    dyn BisectorCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn BisectorCurve]
);

impl Default for BisecPC {
    fn default() -> Self {
        Self::new()
    }
}

impl BisecPC {
    /// `Bisector_BisecPC()`: an empty bisector; `BisecPC::new()`.
    pub fn new() -> Self {
        Self::from_state(PcState {
            is_empty: true,
            ..PcState::default()
        })
    }
    pub(crate) fn from_state(state: PcState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// `Bisector_BisecPC(Cu, P, Side, DistMax)`: [`BisecPC::perform`] on a new object;
    /// `BisecPC::with_curve(&curve, p, 1.0, 500.0)?`.
    pub fn with_curve(
        cu: &Handle<dyn Geom2dCurve>,
        p: Point2,
        side: f64,
        dist_max: f64,
    ) -> Result<Self, Failure> {
        let mut state = PcState::default();
        state.perform(cu, p, side, dist_max)?;
        Ok(Self::from_state(state))
    }
    /// `Bisector_BisecPC(Cu, P, Side, UMin, UMax)`: the bisector on `[UMin, UMax]`
    /// without extension. The native constructor leaves the shift, the maximal
    /// distance and the empty flag as the cleared allocation has them (0, 0 and
    /// false); `BisecPC::with_bounds(&curve, p, 1.0, 0.0, 1.0)?`.
    pub fn with_bounds(
        cu: &Handle<dyn Geom2dCurve>,
        p: Point2,
        side: f64,
        umin: f64,
        umax: f64,
    ) -> Result<Self, Failure> {
        let mut state = PcState {
            curve: copy_curve(cu)?,
            point: p,
            sign: side,
            bis_interval: 1,
            ..PcState::default()
        };
        state.start_intervals.append(umin);
        state.end_intervals.append(umax);
        state.point_start_bis = state.value(umin)?;
        state.point_end_bis = state.value(umax)?;
        state.is_convex = is_convex(&state.curve, state.sign)?;
        Ok(Self::from_state(state))
    }
    /// `Perform(Cu, P, Side, DistMax)`: the bisector of a copy of `Cu` and `P`, on the
    /// side `Side` (1 on the left of the curve, -1 on the right), trimmed where the
    /// distance exceeds `DistMax`. The intervals are appended to the existing ones,
    /// as natively; `bisector.perform(&curve, p, 1.0, 500.0)?`.
    pub fn perform(
        &self,
        cu: &Handle<dyn Geom2dCurve>,
        p: Point2,
        side: f64,
        dist_max: f64,
    ) -> Result<(), Failure> {
        // The fields keep what a failing native Perform left in them.
        let mut state: PcState = self.state.snapshot();
        let result = state.perform(cu, p, side, dist_max);
        *self.state.write() = state;
        result
    }
    /// `IsEmpty()`; `bisector.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.state.read().is_empty
    }
    /// `Distance(U)`: the square distance from the bisector point of `u` to the
    /// curve (and the point); `bisector.distance(0.5)?`.
    pub fn distance(&self, u: f64) -> Result<f64, Failure> {
        self.state.read().distance(u)
    }
    /// `LinkBisCurve(U)`: the parameter on the curve of the bisector parameter `u`;
    /// `bisector.link_bis_curve(0.5)`.
    pub fn link_bis_curve(&self, u: f64) -> f64 {
        self.state.read().link_bis_curve(u)
    }
    /// `LinkCurveBis(U)`: the reverse of [`BisecPC::link_bis_curve`];
    /// `bisector.link_curve_bis(0.5)`.
    pub fn link_curve_bis(&self, u: f64) -> f64 {
        self.state.read().link_curve_bis(u)
    }
    /// `IntervalContinuity()`: the continuity of the curve lowered by one;
    /// `bisector.interval_continuity()`.
    pub fn interval_continuity(&self) -> Shape {
        lowered_continuity(self.state.read().curve.continuity())
    }
    /// `Dump(Deep, Offset)`: the point, the side and the intervals as `std::cout`
    /// gets them; `bisector.dump(0, 0, &mut text)`.
    pub fn dump(&self, _deep: i32, offset: i32, out: &mut String) -> Result<(), Failure> {
        let state = self.state.read();
        let indent: String = " ".repeat(usize::try_from(offset).unwrap_or(0));
        out.push_str(&format!("{indent}Bisector_BisecPC :\n{indent}Point :\n"));
        out.push_str(&format!(
            " X = {}\n Y = {}\n",
            cout_real(state.point.x()),
            cout_real(state.point.y())
        ));
        out.push_str(&format!("Sign  :{}\n", cout_real(state.sign)));
        dump_intervals(
            &state.start_intervals,
            &state.end_intervals,
            state.current_interval,
            out,
        )
    }
}

/// The continuity of a bisector from the one of its curve: one order lower, CN
/// stays CN (`Continuity` and `IntervalContinuity` of the algorithmic bisectors).
pub(crate) fn lowered_continuity(cont: Shape) -> Shape {
    match cont {
        Shape::C1 => Shape::C0,
        Shape::C2 => Shape::C1,
        Shape::C3 => Shape::C2,
        Shape::CN => Shape::CN,
        _ => Shape::C0,
    }
}

impl BisectorCurve for BisecPC {
    fn parameter(&self, p: Point2) -> Result<f64, Failure> {
        self.state.read().parameter(p)
    }
    fn is_extend_at_start(&self) -> bool {
        self.state.read().extension_start
    }
    fn is_extend_at_end(&self) -> bool {
        self.state.read().extension_end
    }
    fn nb_intervals(&self) -> i32 {
        self.state.read().start_intervals.len()
    }
    fn interval_first(&self, index: i32) -> Result<f64, Failure> {
        self.state.read().start_intervals.value(index)
    }
    fn interval_last(&self, index: i32) -> Result<f64, Failure> {
        self.state.read().end_intervals.value(index)
    }
}
