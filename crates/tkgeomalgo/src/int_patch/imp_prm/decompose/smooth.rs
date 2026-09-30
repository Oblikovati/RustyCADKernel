// Rust translation of ToSmooth, IsPointOnBoundary, DetectOfBoundaryAchievement,
// MakeSplitWLine and SplitOnSegments of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The end points of a decomposed line near a pole or apex, the boundaries of the
//! quadric, and the split of a walking line at its inner vertices.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::super::super::line::IntPatchLine;
use super::super::super::point::IntPatchPoint;
use super::super::super::point_line::IntPatchPointLine;
use super::super::super::wline::{IntPatchWLType, IntPatchWLine};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfQuadric, IntSurfTypeTrans};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use std::f64::consts::PI;

/// The quadric U of the points `index` of the line.
fn quad_u(line: &IntSurfLineOn2S, index: i32, is_reversed: bool) -> Result<(f64, f64), Failure> {
    Ok(line.value(index)?.parameters_on_surface(!is_reversed))
}

/// The mean U change and 3d step of the test points (the second loop of the last end
/// does not run when its start is past the count, as natively).
fn mean_steps(
    line: &IntSurfLineOn2S,
    is_reversed: bool,
    is_first: bool,
    d3d: &mut f64,
) -> Result<f64, Failure> {
    *d3d = 0.0;
    let mut nb_test_pnts: i32 = line.nb_points() / 5;
    if nb_test_pnts < 5 {
        nb_test_pnts = 5;
    }
    let startp: i32 = if is_first {
        2
    } else {
        line.nb_points() - nb_test_pnts - 2
    };
    let mut ddu: f64 = 0.0;
    for ip in startp..=nb_test_pnts {
        let (uc, _) = quad_u(line, ip, is_reversed)?;
        let (un, _) = quad_u(line, ip + 1, is_reversed)?;
        ddu += (uc.abs() - un.abs()).abs();
        if ip > startp {
            *d3d += line
                .value(ip)?
                .value()
                .distance(&line.value(ip - 1)?.value());
        }
    }
    ddu /= f64::from(nb_test_pnts) + 1.0;
    *d3d /= f64::from(nb_test_pnts) + 1.0;
    Ok(ddu)
}

/// Whether the end point is in the pole of the sphere (a U jump over PI/16 at the pole)
/// or the apex of the cone (over PI/32 at the apex) and must be moved.
fn needs_smoothing(quad: &IntSurfQuadric, (u1, v1): (f64, f64), u2: f64) -> Result<bool, Failure> {
    let mut do_u: bool = false;
    if quad.type_quadric() == SurfaceType::Sphere {
        if (u1.abs() - u2.abs()).abs() > (PI / 16.0) {
            do_u = true;
        }
        if do_u
            && (u1.abs() <= 1.0e-9 || (u1 - 2.0 * PI).abs() <= 1.0e-9)
            && !((v1 - PI / 2.0).abs() <= 1.0e-9 || (v1 + PI / 2.0).abs() <= 1.0e-9)
        {
            do_u = false;
        }
    }
    if quad.type_quadric() == SurfaceType::Cone {
        let (_, vapx) = quad.parameters(quad.cone()?.apex()).unwrap_or((0.0, 0.0));
        if (u1.abs() - u2.abs()).abs() > PI / 32.0 {
            do_u = true;
        }
        if do_u
            && (u1.abs() <= 1.0e-9 || (u1 - 2.0 * PI).abs() <= 1.0e-9)
            && !((v1 - vapx).abs() <= 1.0e-9)
        {
            do_u = false;
        }
    }
    Ok(do_u)
}

/// `ToSmooth(Line, IsReversed, Quad, IsFirst, D3D)`: on a line of more than 10 points,
/// the quadric U of the end point in a pole or apex is set near the U of its neighbour
/// (by at most 5e-8); `d3d` gets the mean 3d step.
pub(super) fn to_smooth(
    line: &IntSurfLineOn2S,
    is_reversed: bool,
    quad: &IntSurfQuadric,
    is_first: bool,
    d3d: &mut f64,
) -> Result<(), Failure> {
    if line.nb_points() <= 10 {
        return Ok(());
    }
    let ddu: f64 = mean_steps(line, is_reversed, is_first, d3d)?;
    let n: i32 = line.nb_points();
    let (i1, i2, i3) = if is_first {
        (1, 2, 3)
    } else {
        (n, n - 1, n - 2)
    };
    let (u1, v1) = quad_u(line, i1, is_reversed)?;
    let (u2, _) = quad_u(line, i2, is_reversed)?;
    let (u3, _) = quad_u(line, i3, is_reversed)?;
    if needs_smoothing(quad, (u1, v1), u2)? {
        let du: f64 = std_min(ddu / 10.0, 5.0e-8);
        let u: f64 = if u2 > u3 { u2 + du } else { u2 - du };
        line.set_uv(i1, !is_reversed, u, v1)?;
    }
    Ok(())
}

/// `IsPointOnBoundary(theToler2D, theBoundary, thePeriod, theParam)`: the parameter is
/// on the bound (up to the period).
fn is_point_on_boundary(toler_2d: f64, boundary: f64, period: f64, param: f64) -> bool {
    let mut delta: f64 = (param - boundary).abs();
    if period != 0.0 {
        delta %= period;
        // 0 <= delta < period
        return delta < toler_2d || (period - delta) < toler_2d;
    }
    delta < toler_2d
}

/// `DetectOfBoundaryAchievement(theQSurf, theIsReversed, theSourceLine, thePointIndex,
/// theToler2D, theNewLine, theIsOnBoundary)`: whether the point reaches a bound of the
/// quadric that the previous point is not on (a line along the bound is ignored); then
/// the point, its parameters brought near the previous ones by a period, goes into the
/// new line.
pub(super) fn detect_of_boundary_achievement(
    q_surf: &Adaptor3dSurfaceHandle,
    is_reversed: bool,
    source: &IntSurfLineOn2S,
    point_index: i32,
    toler_2d: f64,
    new_line: &IntSurfLineOn2S,
    is_on_boundary: &mut bool,
) -> Result<(), Failure> {
    let u_period: f64 = if q_surf.is_u_periodic()? {
        q_surf.u_period()?
    } else {
        0.0
    };
    let v_period: f64 = if q_surf.is_v_periodic()? {
        q_surf.v_period()?
    } else {
        0.0
    };
    let (uf, ul, vf, vl) = (
        q_surf.first_u_parameter()?,
        q_surf.last_u_parameter()?,
        q_surf.first_v_parameter()?,
        q_surf.last_v_parameter()?,
    );
    let pprev: IntSurfPntOn2S = source.value(point_index - 1)?;
    let pcurr: IntSurfPntOn2S = source.value(point_index)?;
    let (uprev, vprev) = pprev.parameters_on_surface(!is_reversed);
    let (mut ucurr, mut vcurr) = pcurr.parameters_on_surface(!is_reversed);
    let reaches = |b: f64, p: f64, cur: f64, prev: f64| {
        is_point_on_boundary(toler_2d, b, p, cur) && !is_point_on_boundary(toler_2d, b, p, prev)
    };
    if reaches(uf, u_period, ucurr, uprev)
        || reaches(ul, u_period, ucurr, uprev)
        || reaches(vf, v_period, vcurr, vprev)
        || reaches(vl, v_period, vcurr, vprev)
    {
        *is_on_boundary = true;
    }
    if !*is_on_boundary {
        return Ok(());
    }
    // Adjusted, to avoid a bad jump of the line.
    let (du, dv) = (uprev - ucurr, vprev - vcurr);
    if u_period > 0.0 && (2.0 * du.abs() > u_period) {
        ucurr += u_period.copysign(du);
    }
    if v_period > 0.0 && (2.0 * dv.abs() > v_period) {
        vcurr += v_period.copysign(dv);
    }
    let mut point: IntSurfPntOn2S = pcurr;
    point.set_parameters_on(!is_reversed, ucurr, vcurr);
    new_line.add(&point);
    Ok(())
}

/// A vertex at the point of rank `index` of the line (parameter `param`).
pub(super) fn end_vertex(
    line: &IntSurfLineOn2S,
    index: i32,
    arc_tol: f64,
    param: f64,
) -> Result<IntPatchPoint, Failure> {
    let p: IntSurfPntOn2S = line.value(index)?;
    let (u1, v1, u2, v2) = p.parameters();
    let mut pt = IntPatchPoint::new();
    pt.set_value_tangent(p.value(), arc_tol, false);
    pt.set_parameters(u1, v1, u2, v2);
    pt.set_parameter(param);
    Ok(pt)
}

/// `MakeSplitWLine(WLine, Tang, Trans1, Trans2, ArcTol, ParFirst, ParLast)`: the part
/// of the walking line between two point indices, with its end vertices.
fn make_split_wline(
    wline: &IntPatchWLine,
    (tang, trans1, trans2): (bool, IntSurfTypeTrans, IntSurfTypeTrans),
    arc_tol: f64,
    (par_first, par_last): (i32, i32),
) -> Result<Handle<IntPatchWLine>, Failure> {
    let sline_src: Handle<IntSurfLineOn2S> = wline.curve();
    let sline = Handle::new(IntSurfLineOn2S::new(None));
    for ip in par_first..=par_last {
        sline.add(&sline_src.value(ip)?);
    }
    let w = IntPatchWLine::new(&sline, tang, trans1, trans2);
    w.set_creating_way_info(IntPatchWLType::ImpPrm);
    w.add_vertex(&end_vertex(&sline, 1, arc_tol, 1.0)?, false);
    w.set_first_point(1);
    let n: i32 = sline.nb_points();
    w.add_vertex(&end_vertex(&sline, n, arc_tol, f64::from(n))?, false);
    w.set_last_point(w.nb_vertex());
    Ok(Handle::new(w))
}

/// `SplitOnSegments(WLine, Tang, Trans1, Trans2, ArcTol, Segments)` (`transitions` is
/// `(Tang, Trans1, Trans2)`): a line of more than
/// 3 vertices is split between consecutive vertices more than one point apart.
pub(super) fn split_on_segments(
    wline: &IntPatchWLine,
    transitions: (bool, IntSurfTypeTrans, IntSurfTypeTrans),
    arc_tol: f64,
    segments: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<bool, Failure> {
    let mut result: bool = false;
    segments.clear();
    let nbv: i32 = wline.nb_vertex();
    if nbv <= 3 {
        return Ok(false);
    }
    for iv in 1..nbv {
        let first_par: i32 = wline.vertex(iv)?.parameter_on_line() as i32;
        let last_par: i32 = wline.vertex(iv + 1)?.parameter_on_line() as i32;
        if (last_par - first_par) <= 1 {
            continue;
        }
        segments.push(make_split_wline(wline, transitions, arc_tol, (first_par, last_par))?.cast());
        result = true;
    }
    Ok(result)
}
