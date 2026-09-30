// Rust translation of the C0 B-spline p-curve branch of BRepLib::SameParameter
// (BRepLib.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A C0 B-spline p-curve of `BRepLib::SameParameter`: made C1 where its arcs allow,
//! reparametrized to its range, and reparametrized by length when its knots are
//! badly spread.
use super::edge::{Adaptors, PCurveInput};
use super::measure::eval_tol;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::approx::{
    CurvilinearOptions, CurvilinearParameter as ApproxCurvilinearParameter,
};
use rustycad_tkgeombase::geom2d_convert;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point2;

/// The p-curve being checked: `curPC`, `updatepc`, `error` and `goodpc`.
pub(super) struct PCurveState {
    pub(super) cur_pc: Handle<dyn Geom2dCurve>,
    pub(super) updatepc: bool,
    pub(super) error: f64,
    pub(super) goodpc: bool,
}

/// The B-spline under work (`bs2d`), the saved one (`bs2dsov`) and the range of the
/// saved one (`fC0`, `lC0`).
struct Work {
    bs2d: Handle<Geom2dBSplineCurve>,
    sov: Handle<Geom2dBSplineCurve>,
    range: (f64, f64),
}

/// The C0 B-spline branch: sets `isBSP`, and `isANA` when the reparametrized curve is
/// worse than the input; `goodpc` false when the curve stays C0 and the 3d curve does
/// not project; `flags` are `(isANA, isBSP)`.
pub(super) fn smooth_c0_pcurve(
    input: &PCurveInput<'_>,
    adaptors: &Adaptors,
    tolerance: f64,
    state: &mut PCurveState,
    flags: &mut (bool, bool),
) -> Result<(), Failure> {
    let gas = adaptors.gas.get().expect("an adaptor");
    let tol_conf2d: f64 = std_max(
        std_min(gas.u_resolution(tolerance)?, gas.v_resolution(tolerance)?),
        precision::p_confusion(),
    );
    let bs2d = adaptors.gac2d.get().expect("an adaptor").bspline()?;
    let b = bspline(&bs2d);
    let range = (b.first_parameter(), b.last_parameter());
    let origin: Point2 = b.value(range.0)?;
    let mut work = Work {
        range,
        sov: bs2d.clone(),
        bs2d,
    };
    geom2d_convert::c0_bspline_to_c1_bspline_curve(&mut work.bs2d, tol_conf2d)?;
    flags.1 = true;
    reset_origin(&work.bs2d, origin)?;
    let mut repar: bool = true;
    if bspline(&work.bs2d).continuity() == Continuity::C0 {
        match retry_with_bail(
            input,
            adaptors,
            (tolerance, tol_conf2d),
            &state.cur_pc,
            &mut work,
        )? {
            None => state.goodpc = false,
            Some(still_c0) => {
                if still_c0 {
                    state.goodpc = true;
                    work.bs2d = work.sov.clone();
                    repar = false;
                }
            }
        }
    }
    if !state.goodpc {
        return Ok(());
    }
    if repar {
        reparametrized(adaptors, &mut work, state, flags)?;
    }
    let bad_cont = bad_spread(&work.bs2d, state.error, tolerance)?;
    if let Some(cont) = bad_cont {
        by_length(input, adaptors, tolerance, cont, &mut work, state)?;
    }
    Ok(())
}

/// The live B-spline of a handle.
fn bspline(handle: &Handle<Geom2dBSplineCurve>) -> &Geom2dBSplineCurve {
    handle.get().expect("a B-spline p-curve")
}

/// "IFV, Jan 2000": a periodic B-spline whose first point moved away from `origin`
/// takes as origin the first knot at `origin`.
fn reset_origin(handle: &Handle<Geom2dBSplineCurve>, origin: Point2) -> Result<(), Failure> {
    let bs = bspline(handle);
    if !bs.is_periodic() {
        return Ok(());
    }
    let confusion: f64 = precision::p_confusion();
    let moved = |p: Point2| {
        (origin.x() - p.x()).abs() > confusion || (origin.y() - p.y()).abs() > confusion
    };
    if !moved(bs.value(bs.first_parameter())?) {
        return Ok(());
    }
    let knots = bs.knots();
    for index in 1..=bs.nb_knots() {
        if moved(bs.value(knots[index])?) {
            continue;
        }
        return bs.set_origin(index);
    }
    Ok(())
}

/// The retry of a curve left C0: with the tolerance EvalTol measures on `cur_pc` (None
/// when the 3d curve does not project), bounded by a tenth of the shortest pole
/// spacing, from the saved curve; Some(whether the result is still C0).
fn retry_with_bail(
    input: &PCurveInput<'_>,
    adaptors: &Adaptors,
    (tolerance, tol_conf2d): (f64, f64),
    cur_pc: &Handle<dyn Geom2dCurve>,
    work: &mut Work,
) -> Result<Option<bool>, Failure> {
    let gac = adaptors.gac.get().expect("an adaptor");
    let Some(tolbail) = eval_tol(cur_pc, input.surface, gac, tolerance)? else {
        return Ok(None);
    };
    work.bs2d = work.sov.clone();
    let gas = adaptors.gas.get().expect("an adaptor");
    let tol2dbail: f64 = std_min(gas.u_resolution(tolbail)?, gas.v_resolution(tolbail)?);
    let bs = bspline(&work.bs2d);
    let origin: Point2 = bs.value(bs.first_parameter())?;
    let poles = bs.poles();
    let mut d: f64 = precision::infinite();
    for ip in 2..=bs.nb_poles() {
        d = std_min(d, poles[ip - 1].square_distance(poles[ip]));
    }
    d = d.sqrt() * 0.1;
    let tol2dbail: f64 = std_max(std_min(tol2dbail, d), tol_conf2d);
    geom2d_convert::c0_bspline_to_c1_bspline_curve(&mut work.bs2d, tol2dbail)?;
    reset_origin(&work.bs2d, origin)?;
    Ok(Some(bspline(&work.bs2d).continuity() == Continuity::C0))
}

/// The knots of the curve moved to the saved range (`BSplCLib::Reparametrize(fC0,
/// lC0)`), set on the curve itself.
fn set_range_knots(bs: &Geom2dBSplineCurve, (first, last): (f64, f64)) -> Result<(), Failure> {
    let mut knots = bs.knots();
    bsplclib::reparametrize(first, last, &mut knots)?;
    bs.set_knots(&knots)
}

/// "repar": the curve on the saved range, kept when it does not measure worse than
/// before (else the saved curve comes back and `isANA` is set).
fn reparametrized(
    adaptors: &Adaptors,
    work: &mut Work,
    state: &mut PCurveState,
    flags: &mut (bool, bool),
) -> Result<(), Failure> {
    set_range_knots(bspline(&work.bs2d), work.range)?;
    state.cur_pc = work.bs2d.clone().cast();
    adaptors.load_pcurve(&state.cur_pc)?;
    let updatepc_sov: bool = state.updatepc;
    state.updatepc = true;
    let error1: f64 = adaptors.error()?;
    if error1 > state.error {
        work.bs2d = work.sov.clone();
        state.cur_pc = work.bs2d.clone().cast();
        adaptors.load_pcurve(&state.cur_pc)?;
        state.updatepc = updatepc_sov;
        flags.0 = true;
    } else {
        state.error = error1;
    }
    Ok(())
}

/// "check, if new BSpline good or not": for a smooth curve measured above
/// max(1e-3, tolerance), a ratio above 10 between neighbouring knot spans makes it bad
/// unless its shortest span exceeds the resolution; Some(the continuity, at most C2)
/// when bad.
fn bad_spread(
    handle: &Handle<Geom2dBSplineCurve>,
    error: f64,
    tolerance: f64,
) -> Result<Option<Continuity>, Failure> {
    let bs = bspline(handle);
    let cont = bs.continuity();
    let limit: f64 = std_max(1.0e-3, tolerance);
    if !(cont > Continuity::C0 && error > limit) {
        return Ok(None);
    }
    let knots = bs.knots();
    let nb_knots: i32 = bs.nb_knots();
    const CRITRATIO: f64 = 10.;
    let mut dtprev: f64 = knots[2] - knots[1];
    let mut dtmin: f64 = dtprev;
    let mut is_bad: bool = false;
    for j in 2..nb_knots {
        let dtcur: f64 = knots[j + 1] - knots[j];
        dtmin = std_min(dtmin, dtcur);
        if is_bad {
            continue;
        }
        let dtratio: f64 = if dtcur > dtprev {
            dtcur / dtprev
        } else {
            dtprev / dtcur
        };
        if dtratio > CRITRATIO {
            is_bad = true;
        }
        dtprev = dtcur;
    }
    // To avoid failures in Approx_CurvilinearParameter
    if !is_bad || dtmin < bs.resolution(limit)? {
        return Ok(None);
    }
    Ok(Some(if cont > Continuity::C2 {
        Continuity::C2
    } else {
        cont
    }))
}

/// "if BSpline bad, try to reparametrize it by its curve length": the curvilinear
/// approximation of the p-curve on the surface, on the saved range when its own range
/// differs by more than the SameRange tolerance.
fn by_length(
    input: &PCurveInput<'_>,
    adaptors: &Adaptors,
    tolerance: f64,
    cont: Continuity,
    work: &mut Work,
    state: &mut PCurveState,
) -> Result<(), Failure> {
    let mut max_degree: i32 = bspline(&work.bs2d).degree();
    if max_degree == 1 {
        max_degree = 14;
    }
    let options = CurvilinearOptions {
        order: cont,
        max_degree,
        max_segments: 10,
    };
    let surface: Handle<dyn Adaptor3dSurface> = adaptors.hs();
    let app = ApproxCurvilinearParameter::from_curve_on_surface(
        &adaptors.hc2d(),
        &surface,
        std_max(1.0e-3, tolerance),
        options,
    )?;
    if !(app.is_done() || app.has_result()) {
        return Ok(());
    }
    work.bs2d = app.curve_2d1();
    state.cur_pc = work.bs2d.clone().cast();
    adaptors.load_pcurve(&state.cur_pc)?;
    let bs = bspline(&work.bs2d);
    let (f_c0, l_c0) = work.range;
    if (bs.first_parameter() - f_c0).abs() > input.tol_same_range
        || (bs.last_parameter() - l_c0).abs() > input.tol_same_range
    {
        set_range_knots(bs, work.range)?;
        adaptors.load_pcurve(&state.cur_pc)?;
    }
    Ok(())
}
