// Rust translation of the static templates `PerformLinear`, `PerformCircular`,
// `GetDefType` and `PerformComposite`, repeated with the same body in
// ModelingData/TKGeomBase/GCPnts/GCPnts_UniformDeflection.cxx and
// GCPnts_QuasiUniformDeflection.cxx, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The distribution steps shared by `GCPnts_UniformDeflection` and
//! `GCPnts_QuasiUniformDeflection`.
use super::intervals::{bound, hunt_interval, interval_bounds};
use super::point_sequence::PointSequence;
use super::t_curve_types::live;
use super::{DeflectionType, PoleCurve, TCurveTypes};
use crate::cpnts::PntsCurve;
use rustycad_tkernel::ncollection::ElementCast;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::geom_abs::{CurveType, Shape};

/// `GetDefType`: composite above one interval of continuity `shape` (C2 for the
/// uniform, C1 for the quasi-uniform deflection), linear for lines and two-pole
/// curves (rational or not), circular for circles; `def_type(curve, Shape::C2)?`.
pub(super) fn def_type<C: TCurveTypes + ?Sized>(
    curve: &C,
    shape: Shape,
) -> Result<DeflectionType, Failure> {
    let pnts: PntsCurve = curve.pnts_curve();
    if pnts.nb_intervals(shape)? > 1 {
        return Ok(DeflectionType::DefComposite);
    }
    Ok(match pnts.get_type()? {
        CurveType::Line => DeflectionType::Linear,
        CurveType::Circle => DeflectionType::Circular,
        CurveType::BSplineCurve => two_pole_type(live(&curve.bspline_curve()?)?.pole_count()),
        CurveType::BezierCurve => two_pole_type(live(&curve.bezier_curve()?)?.pole_count()),
        _ => DeflectionType::Curved,
    })
}

fn two_pole_type(nb_poles: i32) -> DeflectionType {
    if nb_poles == 2 {
        DeflectionType::Linear
    } else {
        DeflectionType::Curved
    }
}

/// `PerformLinear`: the two ends; always true; `perform_linear(curve, &mut seq, 0.0, 1.0)?`.
pub(super) fn perform_linear(
    curve: PntsCurve<'_>,
    sequence: &mut PointSequence,
    u1: f64,
    u2: f64,
) -> Result<bool, Failure> {
    sequence.append(u1, curve.lifted_d0(u1)?);
    sequence.append(u2, curve.lifted_d0(u2)?);
    Ok(true)
}

/// `PerformCircular`: constant steps of the angle `2 acos(max(1 - Deflection / R, 0))`
/// rounded down to fit `[U1, U2]` plus two points; the last parameter is accumulated,
/// not set to U2; always true; `perform_circular(curve, &mut seq, 0.01, 0.0, 1.0)?`.
///
/// A null deflection makes the native `(int)` of an infinite count undefined; x86-64
/// gives INT_MIN, so no point is stored, as here.
pub(super) fn perform_circular(
    curve: PntsCurve<'_>,
    sequence: &mut PointSequence,
    deflection: f64,
    (u1, u2): (f64, f64),
) -> Result<bool, Failure> {
    let mut angle: f64 = std_max(1.0 - (deflection / curve.circle_radius()?), 0.0);
    angle = 2.0e0 * angle.acos();
    let mut nb_points: i32 = ElementCast::<i32>::cast((u2 - u1) / angle);
    nb_points = nb_points.wrapping_add(2);
    angle = (u2 - u1) / f64::from(nb_points.wrapping_sub(1));
    let mut u: f64 = u1;
    for _ in 1..=nb_points {
        sequence.append(u, curve.lifted_d0(u)?);
        u += angle;
    }
    Ok(true)
}

/// `PerformComposite`: `perform_curve` on each C2 interval met from U1 to U2 (the
/// native files both use C2 here), the duplicated junction point removed;
/// `perform_composite(curve, &mut seq, (0.0, 1.0), &mut |seq, a, b| ...)?`.
pub(super) fn perform_composite(
    curve: PntsCurve<'_>,
    sequence: &mut PointSequence,
    (u1, u2): (f64, f64),
    perform_curve: &mut dyn FnMut(&mut PointSequence, f64, f64) -> Result<bool, Failure>,
) -> Result<bool, Failure> {
    let ti = interval_bounds(curve, Shape::C2)?;
    let nb_intervals: i32 = ti.upper() - 1;
    let mut index: i32 = hunt_interval(&ti, u1)?;
    // iterate by continuous segments
    let mut ua: f64 = u1;
    loop {
        let ub: f64 = if index < ti.upper() {
            std_min(u2, bound(&ti, index + 1)?)
        } else {
            u2
        };
        if !perform_curve(sequence, ua, ub)? {
            return Ok(false);
        }
        index += 1;
        if index > nb_intervals || u2 < bound(&ti, index)? {
            return Ok(true);
        }
        // remove last point to avoid duplication
        sequence.remove_last()?;
        ua = ub;
    }
}
