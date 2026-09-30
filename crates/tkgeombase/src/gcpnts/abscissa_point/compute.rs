// Rust translation of the static templates `computeType`, `Compute` and `AdvCompute` of
// ModelingData/TKGeomBase/GCPnts/GCPnts_AbscissaPoint.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The abscissa search of `GCPnts_AbscissaPoint` by curve classification.
//!
//! `Compute` and `AdvCompute` walk the continuity intervals with the same loop; the
//! port shares it, the differences (integration tolerance, `Perform` or `AdvPerform`,
//! the hit tolerance) being the [`SearchMode`].
use crate::cpnts::{self, PntsCurve};
use crate::gcpnts::intervals::{bound, hunt_interval, interval_bounds};
use crate::gcpnts::t_curve_types::live;
use crate::gcpnts::{AbscissaType, PoleCurve, TCurveTypes};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::{CurveType, Shape};

/// `computeType`: the abscissa type of the curve and, for a length-parametrized one,
/// the ratio `dL / dU` (1 otherwise); `compute_type(curve)?`.
pub(in crate::gcpnts) fn compute_type<C: TCurveTypes + ?Sized>(
    curve: &C,
) -> Result<(AbscissaType, f64), Failure> {
    let pnts: PntsCurve = curve.pnts_curve();
    if pnts.nb_intervals(Shape::CN)? > 1 {
        return Ok((AbscissaType::AbsComposite, 1.0));
    }
    match pnts.get_type()? {
        CurveType::Line => Ok((AbscissaType::LengthParametrized, 1.0)),
        CurveType::Circle => Ok((AbscissaType::LengthParametrized, pnts.circle_radius()?)),
        CurveType::BezierCurve => pole_curve_type(live(&curve.bezier_curve()?)?, false),
        CurveType::BSplineCurve => pole_curve_type(live(&curve.bspline_curve()?)?, true),
        _ => Ok((AbscissaType::Parametrized, 1.0)),
    }
}

// A non-rational two-pole Bezier or B-spline curve is a segment: its ratio is the
// derivative length at 0 (Bezier) or at FirstParameter (B-spline, `at_first`).
fn pole_curve_type(curve: &impl PoleCurve, at_first: bool) -> Result<(AbscissaType, f64), Failure> {
    if curve.pole_count() != 2 || curve.rational() {
        return Ok((AbscissaType::Parametrized, 1.0));
    }
    let u: f64 = if at_first {
        curve.start_parameter()
    } else {
        0.0
    };
    Ok((
        AbscissaType::LengthParametrized,
        curve.derivative_magnitude(u, 1)?,
    ))
}

/// The CPnts calls that differ between `Compute` and `AdvCompute`.
#[derive(Clone, Copy)]
enum SearchMode {
    /// `Compute`: lengths without tolerance, `Perform`, hits within Confusion.
    Plain,
    /// `AdvCompute`: lengths with the tolerance EPSILON, `AdvPerform`, hits within
    /// PConfusion.
    Advanced,
}

impl SearchMode {
    fn length(self, curve: PntsCurve<'_>, u1: f64, u2: f64, eps: f64) -> Result<f64, Failure> {
        match self {
            Self::Plain => cpnts::AbscissaPoint::length_between(curve, u1, u2),
            Self::Advanced => {
                cpnts::AbscissaPoint::length_between_with_tolerance(curve, u1, u2, eps)
            }
        }
    }
    fn hit_tolerance(self) -> f64 {
        match self {
            Self::Plain => precision::confusion(),
            Self::Advanced => precision::p_confusion(),
        }
    }
    // Init on [u1, u2] then Perform or AdvPerform.
    fn solve<'a>(
        self,
        computer: &mut cpnts::AbscissaPoint<'a>,
        curve: PntsCurve<'a>,
        (u1, u2): (f64, f64),
        (abscissa, u0, ui, eps): (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        match self {
            Self::Plain => {
                computer.init_between(curve, u1, u2)?;
                computer.perform_from(abscissa, u0, ui, eps)
            }
            Self::Advanced => {
                computer.init_between_with_tolerance(curve, u1, u2, eps)?;
                computer.adv_perform(abscissa, u0, ui, eps)
            }
        }
    }
}

/// The state of the interval walk: the remaining abscissa, its origin, the start of
/// the root search, the sign of the abscissa and the walk direction (1 forward,
/// 0 backward, the offset of the interval end reached).
#[derive(Clone, Copy)]
struct AbscissaWalk {
    abscis: f64,
    u0: f64,
    ui: f64,
    sign: f64,
    direction: i32,
    eps: f64,
}

impl AbscissaWalk {
    fn new(abscis: f64, u0: f64, ui: f64, eps: f64) -> Self {
        let backward: bool = abscis < 0.0;
        Self {
            abscis: if backward { -abscis } else { abscis },
            u0,
            ui,
            sign: if backward { -1.0 } else { 1.0 },
            direction: if backward { 0 } else { 1 },
            eps,
        }
    }
    // The start of the root search when `ui` is outside [T(index), T(index + 1)]:
    // the abscissa ratio of the interval, taken from U0 in the walk direction.
    // Written as the native test, so a NaN `ui` is kept.
    #[allow(clippy::nonminimal_bool)]
    fn restart(&mut self, low: f64, high: f64, length: f64) {
        if !(self.ui < low || self.ui > high) {
            return;
        }
        let step: f64 = (self.abscis / length) * (high - self.u0);
        self.ui = if self.direction != 0 {
            self.u0 + step
        } else {
            self.u0 - step
        };
    }
    fn target(&self) -> (f64, f64, f64, f64) {
        (self.sign * self.abscis, self.u0, self.ui, self.eps)
    }
}

/// `Compute`: the parameter at `abscis` from `u0`, the search started at `ui`, with
/// the parametric resolution `eps`; `compute(&mut computer, curve, 1.0, 0.0, 0.5, 1e-9)?`.
pub(in crate::gcpnts) fn compute<'a, C: TCurveTypes + ?Sized>(
    computer: &mut cpnts::AbscissaPoint<'a>,
    curve: &'a C,
    (abscis, u0, ui): (f64, f64, f64),
    eps: f64,
) -> Result<(), Failure> {
    // test for easy solution
    if abscis.abs() <= precision::confusion() {
        computer.set_parameter(u0);
        return Ok(());
    }
    let (kind, ratio) = compute_type(curve)?;
    match kind {
        AbscissaType::LengthParametrized => computer.set_parameter(u0 + abscis / ratio),
        AbscissaType::Parametrized => {
            computer.init(curve.pnts_curve())?;
            computer.perform_from(abscis, u0, ui, eps)?;
        }
        AbscissaType::AbsComposite => {
            let walk = AbscissaWalk::new(abscis, u0, ui, eps);
            composite(computer, curve.pnts_curve(), walk, SearchMode::Plain)?;
        }
    }
    Ok(())
}

/// `AdvCompute`: as [`compute`] with the tolerance `eps` in every CPnts call and no
/// zero-abscissa shortcut; `adv_compute(&mut computer, curve, 1.0, 0.0, 0.5, 1e-9)?`.
pub(in crate::gcpnts) fn adv_compute<'a, C: TCurveTypes + ?Sized>(
    computer: &mut cpnts::AbscissaPoint<'a>,
    curve: &'a C,
    (abscis, u0, ui): (f64, f64, f64),
    eps: f64,
) -> Result<(), Failure> {
    let (kind, ratio) = compute_type(curve)?;
    match kind {
        AbscissaType::LengthParametrized => computer.set_parameter(u0 + abscis / ratio),
        AbscissaType::Parametrized => {
            // theComputer.Init (theC); replaced by "rbv's modification"
            computer.init_with_tolerance(curve.pnts_curve(), eps)?;
            computer.adv_perform(abscis, u0, ui, eps)?;
        }
        AbscissaType::AbsComposite => {
            let walk = AbscissaWalk::new(abscis, u0, ui, eps);
            composite(computer, curve.pnts_curve(), walk, SearchMode::Advanced)?;
        }
    }
    Ok(())
}

// The GCPnts_AbsComposite branch: walk the CN intervals from the one of U0,
// consuming the abscissa, and solve in the interval where it ends.
fn composite<'a>(
    computer: &mut cpnts::AbscissaPoint<'a>,
    curve: PntsCurve<'a>,
    mut walk: AbscissaWalk,
    mode: SearchMode,
) -> Result<(), Failure> {
    let ti: Array1<'static, f64> = interval_bounds(curve, Shape::CN)?;
    let nb_intervals: i32 = ti.upper() - 1;
    let mut index: i32 = hunt_interval(&ti, walk.u0)?;
    if matches!(mode, SearchMode::Advanced) && index == 0 && walk.direction > 0 {
        if before_first_interval(computer, curve, &ti, &mut walk)? {
            return Ok(());
        }
        index += 1;
    }
    while index >= 1 && index <= nb_intervals {
        if walk_interval(computer, curve, (&ti, index), &mut walk, mode)? {
            return Ok(());
        }
        index += if walk.direction != 0 { 1 } else { -1 };
    }
    push_outside(computer, curve, &walk, mode)
}

// One interval of the walk; true when the solution is set (the abscissa ends at the
// interval end within the hit tolerance, or inside the interval).
fn walk_interval<'a>(
    computer: &mut cpnts::AbscissaPoint<'a>,
    curve: PntsCurve<'a>,
    (ti, index): (&Array1<'_, f64>, i32),
    walk: &mut AbscissaWalk,
    mode: SearchMode,
) -> Result<bool, Failure> {
    let end: f64 = bound(ti, index + walk.direction)?;
    let l: f64 = mode.length(curve, walk.u0, end, walk.eps)?;
    if (l - walk.abscis).abs() <= mode.hit_tolerance() {
        computer.set_parameter(end);
        return Ok(true);
    }
    if l > walk.abscis {
        let (low, high) = (bound(ti, index)?, bound(ti, index + 1)?);
        walk.restart(low, high, l);
        mode.solve(computer, curve, (low, high), walk.target())?;
        return Ok(true);
    }
    walk.u0 = end;
    walk.abscis -= l;
    Ok(false)
}

// AdvCompute before the first interval, walking forward: the span [U0, T(1)] is
// consumed first, with the tolerance EPSILON as hit tolerance.
fn before_first_interval<'a>(
    computer: &mut cpnts::AbscissaPoint<'a>,
    curve: PntsCurve<'a>,
    ti: &Array1<'_, f64>,
    walk: &mut AbscissaWalk,
) -> Result<bool, Failure> {
    let first: f64 = bound(ti, 1)?;
    let l: f64 = SearchMode::Advanced.length(curve, walk.u0, first, walk.eps)?;
    if (l - walk.abscis).abs() <= walk.eps {
        computer.set_parameter(first);
        return Ok(true);
    }
    if l > walk.abscis {
        if walk.ui > first {
            walk.ui = walk.u0 + (walk.abscis / l) * (first - walk.u0);
        }
        SearchMode::Advanced.solve(computer, curve, (walk.u0, first), walk.target())?;
        return Ok(true);
    }
    walk.u0 = first;
    walk.abscis -= l;
    Ok(false)
}

// The abscissa goes beyond the last interval: "Push a little bit outside the limits
// (hairy !!!)". Compute always pushes forward by 0.1 on [U0, U0 + 0.2]; AdvCompute
// pushes in the walk direction, clamped to the curve bounds when not periodic.
fn push_outside<'a>(
    computer: &mut cpnts::AbscissaPoint<'a>,
    curve: PntsCurve<'a>,
    walk: &AbscissaWalk,
    mode: SearchMode,
) -> Result<(), Failure> {
    let mut walk: AbscissaWalk = *walk;
    let u1: f64 = match mode {
        SearchMode::Plain => {
            walk.ui = walk.u0 + 0.1;
            walk.u0 + 0.2
        }
        SearchMode::Advanced => clamped_push(curve, &mut walk)?,
    };
    mode.solve(computer, curve, (walk.u0, u1), walk.target())
}

// AdvCompute's push: Ui = U0 + sign 0.1 and the bound U0 + sign 0.2, both clamped to
// [FirstParameter, LastParameter] on a non-periodic curve.
fn clamped_push(curve: PntsCurve<'_>, walk: &mut AbscissaWalk) -> Result<f64, Failure> {
    let is_non_periodic: bool = !curve.is_periodic()?;
    walk.ui = walk.u0 + walk.sign * 0.1;
    let mut u1: f64 = walk.u0 + walk.sign * 0.2;
    if !is_non_periodic {
        return Ok(u1);
    }
    if walk.sign > 0.0 {
        let last: f64 = curve.last_parameter()?;
        walk.ui = std_min(walk.ui, last);
        u1 = std_min(u1, last);
    } else {
        let first: f64 = curve.first_parameter()?;
        walk.ui = std_max(walk.ui, first);
        u1 = std_max(u1, first);
    }
    Ok(u1)
}
