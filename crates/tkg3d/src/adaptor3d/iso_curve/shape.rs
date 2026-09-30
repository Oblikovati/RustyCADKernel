// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_IsoCurve.cxx (Continuity, NbIntervals, Intervals, IsClosed, IsPeriodic, Period, EvalD0..EvalDN) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Continuity, intervals, closure and evaluation of `Adaptor3d_IsoCurve`: each
//! reads the surface member of the curve direction (V for an iso-U curve).
use super::{IsoState, none_iso};
use crate::geom::errors::{array_bounds, no_such_object};
use crate::geom::{GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::{IsoType, Shape};
use rustycad_tkmath::gp::{Point3, Vector3};

/// `Continuity`; `continuity(&state)?`.
pub(super) fn continuity(state: &IsoState) -> Result<Shape, Failure> {
    match state.iso {
        IsoType::IsoU => state.surface()?.v_continuity(),
        IsoType::IsoV => state.surface()?.u_continuity(),
        IsoType::NoneIso => Err(none_iso()),
    }
}

// The surface intervals of the curve direction, into `1..=NbIntervals + 1`.
fn surface_intervals(state: &IsoState, shape: Shape) -> Result<Array1<'static, f64>, Failure> {
    if state.iso == IsoType::NoneIso {
        return Err(no_such_object("Adaptor3d_IsoCurve::NbIntervals: NoneIso"));
    }
    let surface = state.surface()?;
    let u_iso: bool = state.iso == IsoType::IsoU;
    let count: i32 = if u_iso {
        surface.nb_v_intervals(shape)?
    } else {
        surface.nb_u_intervals(shape)?
    };
    let mut bounds: Array1<'static, f64> =
        Array1::with_bounds(1, count + 1).map_err(array_bounds)?;
    if u_iso {
        surface.v_intervals(&mut bounds, shape)?;
    } else {
        surface.u_intervals(&mut bounds, shape)?;
    }
    Ok(bounds)
}

// The first index past `first` and the last before `last` (natively two unchecked
// scans, here bounded by the array: a scan leaving it raises Standard_OutOfRange).
fn scan(bounds: &Array1<'_, f64>, (first, last): (f64, f64)) -> Result<(i32, i32), Failure> {
    let mut lower: i32 = 1;
    while *bounds.value(lower).map_err(array_bounds)? <= first {
        lower += 1;
    }
    let mut upper: i32 = bounds.upper();
    while *bounds.value(upper).map_err(array_bounds)? >= last {
        upper -= 1;
    }
    Ok((lower, upper))
}

/// `NbIntervals`: the surface intervals that overlap `[First, Last]`;
/// `nb_intervals(&state, Shape::C2)?`.
pub(super) fn nb_intervals(state: &IsoState, shape: Shape) -> Result<i32, Failure> {
    let bounds = surface_intervals(state, shape)?;
    let count: i32 = bounds.upper() - 1;
    if count == 1 {
        return Ok(count);
    }
    let (lower, upper) = scan(&bounds, (state.first, state.last))?;
    Ok(upper - lower + 2)
}

/// `Intervals`: the overlapping surface bounds, the ends replaced by `First` and
/// `Last`; `intervals(&state, &mut out, Shape::C2)?`.
pub(super) fn intervals(
    state: &IsoState,
    out: &mut Array1<'_, f64>,
    shape: Shape,
) -> Result<(), Failure> {
    let bounds = surface_intervals(state, shape)?;
    let base: i32 = out.lower();
    if bounds.upper() - 1 == 1 {
        out.set_value(base, state.first).map_err(array_bounds)?;
        return out.set_value(base + 1, state.last).map_err(array_bounds);
    }
    let (lower, upper) = scan(&bounds, (state.first, state.last))?;
    for (offset, index) in (lower - 1..=upper + 1).enumerate() {
        let value: f64 = *bounds.value(index).map_err(array_bounds)?;
        out.set_value(base + offset as i32, value)
            .map_err(array_bounds)?;
    }
    out.set_value(base, state.first).map_err(array_bounds)?;
    out.set_value(base + upper - lower + 2, state.last)
        .map_err(array_bounds)
}

/// `IsClosed`; `is_closed(&state)?`.
pub(super) fn is_closed(state: &IsoState) -> Result<bool, Failure> {
    match state.iso {
        IsoType::IsoU => state.surface()?.is_v_closed(),
        IsoType::IsoV => state.surface()?.is_u_closed(),
        IsoType::NoneIso => Err(none_iso()),
    }
}
/// `IsPeriodic`; `is_periodic(&state)?`.
pub(super) fn is_periodic(state: &IsoState) -> Result<bool, Failure> {
    match state.iso {
        IsoType::IsoU => state.surface()?.is_v_periodic(),
        IsoType::IsoV => state.surface()?.is_u_periodic(),
        IsoType::NoneIso => Err(none_iso()),
    }
}
/// `Period`; `period(&state)?`.
pub(super) fn period(state: &IsoState) -> Result<f64, Failure> {
    match state.iso {
        IsoType::IsoU => state.surface()?.v_period(),
        IsoType::IsoV => state.surface()?.u_period(),
        IsoType::NoneIso => Err(none_iso()),
    }
}

/// `EvalD0`; `d0(&state, 0.5)?`.
pub(super) fn d0(state: &IsoState, u: f64) -> Result<Point3, Failure> {
    match state.iso {
        IsoType::IsoU => state.surface()?.eval_d0(state.parameter, u),
        IsoType::IsoV => state.surface()?.eval_d0(u, state.parameter),
        IsoType::NoneIso => Err(none_iso()),
    }
}
/// `EvalD1`: the V (iso-U) or U (iso-V) derivative; `d1(&state, 0.5)?`.
pub(super) fn d1(state: &IsoState, u: f64) -> Result<GeomCurveD1, Failure> {
    match state.iso {
        IsoType::IsoU => {
            let r = state.surface()?.eval_d1(state.parameter, u)?;
            Ok(GeomCurveD1 {
                point: r.point,
                d1: r.d1v,
            })
        }
        IsoType::IsoV => {
            let r = state.surface()?.eval_d1(u, state.parameter)?;
            Ok(GeomCurveD1 {
                point: r.point,
                d1: r.d1u,
            })
        }
        IsoType::NoneIso => Err(none_iso()),
    }
}
/// `EvalD2`; `d2(&state, 0.5)?`.
pub(super) fn d2(state: &IsoState, u: f64) -> Result<GeomCurveD2, Failure> {
    match state.iso {
        IsoType::IsoU => {
            let r = state.surface()?.eval_d2(state.parameter, u)?;
            Ok(GeomCurveD2 {
                point: r.point,
                d1: r.d1v,
                d2: r.d2v,
            })
        }
        IsoType::IsoV => {
            let r = state.surface()?.eval_d2(u, state.parameter)?;
            Ok(GeomCurveD2 {
                point: r.point,
                d1: r.d1u,
                d2: r.d2u,
            })
        }
        IsoType::NoneIso => Err(none_iso()),
    }
}
/// `EvalD3`; `d3(&state, 0.5)?`.
pub(super) fn d3(state: &IsoState, u: f64) -> Result<GeomCurveD3, Failure> {
    match state.iso {
        IsoType::IsoU => {
            let r = state.surface()?.eval_d3(state.parameter, u)?;
            Ok(GeomCurveD3 {
                point: r.point,
                d1: r.d1v,
                d2: r.d2v,
                d3: r.d3v,
            })
        }
        IsoType::IsoV => {
            let r = state.surface()?.eval_d3(u, state.parameter)?;
            Ok(GeomCurveD3 {
                point: r.point,
                d1: r.d1u,
                d2: r.d2u,
                d3: r.d3u,
            })
        }
        IsoType::NoneIso => Err(none_iso()),
    }
}
/// `EvalDN`: `DN(Param, U, 0, N)` or `DN(U, Param, N, 0)`; `dn(&state, 0.5, 2)?`.
pub(super) fn dn(state: &IsoState, u: f64, n: i32) -> Result<Vector3, Failure> {
    match state.iso {
        IsoType::IsoU => state.surface()?.eval_dn(state.parameter, u, 0, n),
        IsoType::IsoV => state.surface()?.eval_dn(u, state.parameter, n, 0),
        IsoType::NoneIso => Err(none_iso()),
    }
}
