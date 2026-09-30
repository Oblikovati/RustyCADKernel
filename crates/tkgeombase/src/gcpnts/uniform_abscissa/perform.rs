// Rust translation of the static templates `GetParameterLengthRatio`, `GetAbsType`,
// `Perform` and `PerformLengthParametrized` of
// ModelingData/TKGeomBase/GCPnts/GCPnts_UniformAbscissa.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two distributions of `GCPnts_UniformAbscissa`.
use crate::gcpnts::t_curve_types::live;
use crate::gcpnts::{AbscissaPoint, AbscissaType, PoleCurve, TCurveTypes};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::{CurveType, Shape};

/// The inputs of one distribution: the spacing, the range, the range length and the
/// parametric resolution.
#[derive(Clone, Copy)]
pub(super) struct AbscissaRun {
    pub abscissa: f64,
    pub u1: f64,
    pub u2: f64,
    pub total: f64,
    pub eps: f64,
}

/// `GetAbsType`: composite above one C1 interval (C1, not CN as in
/// GCPnts_AbscissaPoint), length-parametrized for lines, circles and non-rational
/// two-pole curves; `abs_type(curve)?`.
pub(super) fn abs_type<C: TCurveTypes + ?Sized>(curve: &C) -> Result<AbscissaType, Failure> {
    let pnts = curve.pnts_curve();
    if pnts.nb_intervals(Shape::C1)? > 1 {
        return Ok(AbscissaType::AbsComposite);
    }
    Ok(match pnts.get_type()? {
        CurveType::Line | CurveType::Circle => AbscissaType::LengthParametrized,
        CurveType::BezierCurve => segment_type(live(&curve.bezier_curve()?)?),
        CurveType::BSplineCurve => segment_type(live(&curve.bspline_curve()?)?),
        _ => AbscissaType::Parametrized,
    })
}

fn segment_type(curve: &impl PoleCurve) -> AbscissaType {
    if curve.pole_count() == 2 && !curve.rational() {
        return AbscissaType::LengthParametrized;
    }
    AbscissaType::Parametrized
}

// `GetParameterLengthRatio`: dL = Ratio dU; the radius of a circle, 1 for a line, the
// derivative length at the parameter 0 (not FirstParameter) of a non-rational Bezier
// or B-spline curve, RealLast() otherwise.
fn parameter_length_ratio<C: TCurveTypes + ?Sized>(curve: &C) -> Result<f64, Failure> {
    let pnts = curve.pnts_curve();
    match pnts.get_type()? {
        CurveType::Circle => pnts.circle_radius(),
        CurveType::Line => Ok(1.0),
        CurveType::BezierCurve | CurveType::BSplineCurve if !pnts.is_rational()? => {
            pnts.dn_magnitude(0.0, 1)
        }
        _ => Ok(real_last()),
    }
}

fn at(params: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    params
        .value(index)
        .copied()
        .map_err(|error| error.into_failure("NCollection_Array1::Value"))
}

fn set(params: &mut Array1<'_, f64>, index: i32, value: f64) -> Result<(), Failure> {
    params
        .set_value(index, value)
        .map_err(|error| error.into_failure("NCollection_Array1::SetValue"))
}

/// `Perform`: each point from the previous one by a GCPnts_AbscissaPoint search
/// started one parametric step `Delta` ahead; a failed search shrinks `Delta` by a
/// tenth (down to PConfusion) and makes the whole result not done, even when a later
/// search succeeds; returns IsDone and sets `nb_points`.
pub(super) fn perform<C: TCurveTypes + ?Sized>(
    params: &mut Array1<'_, f64>,
    curve: &C,
    run: AbscissaRun,
    nb_points: &mut i32,
) -> Result<bool, Failure> {
    let mut is_local_done: bool = true;
    let (uu1, uu2) = (std_min(run.u1, run.u2), std_max(run.u1, run.u2));
    *nb_points = 0;
    // this initialization avoids the computation of the Length of the curve
    let mut delta: f64 = (run.abscissa / run.total) * (uu2 - uu1);
    let mut index: i32 = 1;
    set(params, index, uu1)?;
    loop {
        let start: f64 = at(params, index)?;
        if let Some(found) = search_next(curve, run, (start, delta), uu2)? {
            index += 1;
            if !store_found(params, index, found, (uu2, run.eps))? {
                break;
            }
            continue;
        }
        is_local_done = false;
        delta -= delta / 10.0;
        if delta <= precision::p_confusion() {
            break;
        }
    }
    *nb_points = index;
    Ok(is_local_done)
}

// The GCPnts_AbscissaPoint search of the point at the abscissa from `start`, started
// one step `delta` ahead; None when it is not done.
fn search_next<C: TCurveTypes + ?Sized>(
    curve: &C,
    run: AbscissaRun,
    (start, delta): (f64, f64),
    uu2: f64,
) -> Result<Option<f64>, Failure> {
    // MSV 21.04.2004: OCC5739 (GCPnts_UniformAbscissa gives incorrect distribution of
    // points): the start is clamped to U2 instead of ending the distribution.
    let ui: f64 = if start + delta > uu2 {
        uu2
    } else {
        start + delta
    };
    let finder = AbscissaPoint::from_start_with_tolerance(curve, run.abscissa, start, ui, run.eps)?;
    if !finder.is_done() {
        return Ok(None);
    }
    Ok(Some(finder.parameter()?))
}

// Store the found parameter at `index` (U2 when within EPSILON of it or beyond);
// true while the distribution goes on and the array has room for one more point.
fn store_found(
    params: &mut Array1<'_, f64>,
    index: i32,
    ui: f64,
    (uu2, eps): (f64, f64),
) -> Result<bool, Failure> {
    let is_not_done: bool = if (ui - uu2).abs() <= eps {
        set(params, index, uu2)?;
        false
    } else if ui < uu2 {
        set(params, index, ui)?;
        true
    } else {
        set(params, index, uu2)?;
        false
    };
    Ok(is_not_done && index < params.length())
}

/// `PerformLengthParametrized`: constant parametric steps `Abscissa / Ratio`; a
/// negative abscissa walks from U2 down (and stops at once); the last point snaps
/// the previous one to U2 when it is within a tenth of the abscissa; returns true and
/// sets `nb_points`.
pub(super) fn perform_length_parametrized<C: TCurveTypes + ?Sized>(
    params: &mut Array1<'_, f64>,
    curve: &C,
    run: AbscissaRun,
    nb_points: &mut i32,
) -> Result<bool, Failure> {
    let (mut uu1, mut uu2) = (std_min(run.u1, run.u2), std_max(run.u1, run.u2));
    // Ratio is defined as dl = Ratio * du
    let ratio: f64 = parameter_length_ratio(curve)?;
    if run.abscissa < 0.0e0 {
        uu2 = std_min(run.u1, run.u2);
        uu1 = std_max(run.u1, run.u2);
    }
    let delta: f64 = (run.abscissa / run.total) * (uu2 - uu1);
    let mut index: i32 = 1;
    *nb_points = 0;
    set(params, index, uu1)?;
    let mut is_not_done: bool = true;
    while is_not_done {
        index += 1;
        let ui: f64 = at(params, index - 1)? + delta;
        is_not_done = step_length_parametrized(params, &mut index, ui, (uu2, ratio, run))?;
        is_not_done = index < params.length() && is_not_done;
    }
    *nb_points = index;
    Ok(true)
}

// One step of PerformLengthParametrized storing `ui` at `index`; false when the
// distribution ends (at U2, or past it with the snap of the previous point).
fn step_length_parametrized(
    params: &mut Array1<'_, f64>,
    index: &mut i32,
    ui: f64,
    (uu2, ratio, run): (f64, f64, AbscissaRun),
) -> Result<bool, Failure> {
    if (ui - uu2).abs() <= run.eps {
        set(params, *index, uu2)?;
        return Ok(false);
    }
    if ui < uu2 {
        set(params, *index, ui)?;
        return Ok(true);
    }
    let previous: f64 = at(params, *index - 1)?;
    if (previous - uu2).abs() * ratio / run.abscissa < 0.1 {
        set(params, *index - 1, uu2)?;
        *index -= 1;
    } else {
        set(params, *index, uu2)?;
    }
    Ok(false)
}
