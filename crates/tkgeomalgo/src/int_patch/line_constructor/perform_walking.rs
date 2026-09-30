// Rust translation of the walking branch of IntPatch_LineConstructor::Perform
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_LineConstructor::Perform` on a walking line: each part between two
//! vertices whose middle point is inside both domains becomes a line, which is then
//! tested against the restrictions.
use super::helpers::{is_segment_small, recadre, surface_tolerance};
use super::perform_curves::Split;
use super::rline_coding::test_wline_to_rline;
use super::wline_rline::Domains;
use crate::int_patch::{IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchWLine};
use crate::intf::native_int_cast;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::Point2;

/// The reframed parameters of a point of the line.
fn reframed(domains: &Domains<'_>, params: (f64, f64, f64, f64)) -> Result<[f64; 4], Failure> {
    let mut uv: [f64; 4] = [params.0, params.1, params.2, params.3];
    recadre(domains.s1, domains.s2, &mut uv)?;
    Ok(uv)
}

/// The state of `(u, v)` in the domain of one side, with the parametric tolerance of
/// `tol_arc` there.
fn classify_on(
    domains: &Domains<'_>,
    on_first: bool,
    (u, v): (f64, f64),
    tol_arc: f64,
) -> Result<State, Failure> {
    let (s, d) = domains.side(on_first);
    let tolerance: f64 = surface_tolerance(s, tol_arc, u, v)?;
    d.classify(Point2::new(u, v), tolerance, false)
}

/// When the line has one segment between the vertices (a bad parametrization of the
/// line, sometimes), a side not IN is classified again at the middle of the middle
/// point and the farther vertex, as if the vertices were farther apart.
fn reclassify_virtual(
    domains: &Domains<'_>,
    (vtx_i, vtx_ip1): (&IntPatchPoint, &IntPatchPoint),
    mid: [f64; 4],
    (in1, in2): (&mut State, &mut State),
    tol_arc: f64,
) -> Result<(), Failure> {
    let vf: [f64; 4] = reframed(domains, vtx_i.pnt_on_2s().parameters())?;
    let vl: [f64; 4] = reframed(domains, vtx_ip1.pnt_on_2s().parameters())?;
    for (on_first, state) in [(true, in1), (false, in2)] {
        if *state == State::In {
            continue;
        }
        let k: usize = if on_first { 0 } else { 2 };
        let pv_f = Point2::new(vf[k], vf[k + 1]);
        let pv_l = Point2::new(vl[k], vl[k + 1]);
        let p_pm = Point2::new(mid[k], mid[k + 1]);
        let far: [f64; 4] = if pv_f.distance(p_pm) > pv_l.distance(p_pm) {
            vf
        } else {
            vl
        };
        let du: f64 = (far[k] + mid[k]) * 0.5;
        let dv: f64 = (far[k + 1] + mid[k + 1]) * 0.5;
        *state = classify_on(domains, on_first, (du, dv), tol_arc)?;
    }
    Ok(())
}

/// Whether the part is too small: its end points and the middle point have the same
/// parameters on the first surface.
fn is_too_small(
    domains: &Domains<'_>,
    wline: &IntPatchWLine,
    (firstp, lastp): (f64, f64),
    mid: [f64; 4],
) -> Result<bool, Failure> {
    let a: [f64; 4] = reframed(domains, wline.point(native_int_cast(firstp))?.parameters())?;
    let b: [f64; 4] = reframed(domains, wline.point(native_int_cast(lastp))?.parameters())?;
    if (a[0] - b[0]).abs() + (a[1] - b[1]).abs() < 1e-12 {
        return Ok((mid[0] - b[0]).abs() + (mid[1] - b[1]).abs() < 1e-12);
    }
    Ok(false)
}

/// Whether the walking part between the vertices `i` and `i + 1` is inside both domains
/// and long enough.
fn is_kept_part(
    domains: &Domains<'_>,
    wline: &IntPatchWLine,
    i: i32,
    tol_arc: f64,
) -> Result<bool, Failure> {
    let (vtx_i, vtx_ip1) = (wline.vertex(i)?, wline.vertex(i + 1)?);
    let (firstp, lastp) = (vtx_i.parameter_on_line(), vtx_ip1.parameter_on_line());
    if !(firstp != lastp && !is_segment_small(wline, i, i + 1)?) {
        return Ok(false);
    }
    let int_lastp: i32 = native_int_cast(lastp);
    let int_firstp: i32 = native_int_cast(firstp);
    let mut pmid: i32 = native_int_cast((firstp + lastp) / 2.0);
    if pmid == int_lastp {
        pmid = int_firstp;
    }
    let mid: [f64; 4] = reframed(domains, wline.point(pmid)?.parameters())?;
    let mut in1: State = classify_on(domains, true, (mid[0], mid[1]), tol_arc)?;
    let mut in2: State = State::Out;
    if in1 != State::Out {
        in2 = classify_on(domains, false, (mid[2], mid[3]), tol_arc)?;
    }
    if int_lastp.wrapping_sub(int_firstp).wrapping_abs() == 1 {
        reclassify_virtual(
            domains,
            (&vtx_i, &vtx_ip1),
            mid,
            (&mut in1, &mut in2),
            tol_arc,
        )?;
    }
    if in1 == State::Out || in2 == State::Out {
        return Ok(false);
    }
    Ok(!is_too_small(domains, wline, (firstp, lastp), mid)?)
}

/// `Perform` on a walking line; `slinref` holds the lines of the intersection, whose
/// restrictions the parts are tested against.
pub(super) fn perform_wline(
    split: &mut Split<'_>,
    slinref: &[Handle<dyn IntPatchLine>],
    domains: &Domains<'_>,
    tol_arc: f64,
) -> Result<(), Failure> {
    let wline: Handle<IntPatchWLine> = split.l.cast::<IntPatchWLine>();
    let nbvtx: i32 = wline.nb_vertex();
    for i in 1..nbvtx {
        if is_kept_part(domains, &wline, i, tol_arc)? {
            split.add_part(i, i + 1)?;
            // The last entry of slin is tested.
            test_wline_to_rline(slinref, split.slin, domains, tol_arc)?;
        }
    }
    split.add_isolated_vertices(nbvtx, |i| {
        let v = wline.vertex(i)?;
        Ok(v.is_on_dom_s1() && v.is_on_dom_s2())
    })
}
