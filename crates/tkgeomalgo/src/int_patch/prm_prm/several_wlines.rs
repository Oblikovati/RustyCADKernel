// Rust translation of the static helper SeveralWlinesProcessing of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SeveralWlinesProcessing`: each vertex of a new walking line that is near a vertex
//! of an existing walking line (within the steps of the walking) is moved onto it, so
//! that the lines meet.
use super::helpers::make_new_point;
use crate::int_patch::{
    IntPatchIType, IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchWLType, IntPatchWLine,
};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTypeTrans};
use crate::intf::{native_int_cast, std_max};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::{Point2, Point3};

/// The inputs of `SeveralWlinesProcessing` besides the lines.
pub(super) struct SeveralWlinesInputs<'a> {
    pub(super) surf1: &'a Adaptor3dSurfaceHandle,
    pub(super) surf2: &'a Adaptor3dSurfaceHandle,
    pub(super) periods: [f64; 4],
    pub(super) trans: (IntSurfTypeTrans, IntSurfTypeTrans),
    pub(super) tol: f64,
    /// The maximal steps of the walking on both surfaces.
    pub(super) max_steps: (f64, f64),
}

/// The parameters of a vertex as two 2d points.
fn vertex_uv(vertex: &IntPatchPoint) -> (Point2, Point2) {
    let (u1, v1, u2, v2) = vertex.parameters();
    (Point2::new(u1, v1), Point2::new(u2, v2))
}

/// The nearest vertex of the existing walking lines to the vertex `c_vertex`, within
/// the steps: `(vertex rank, line rank)`, `(0, 0)` when none.
fn nearest_vertex(
    inputs: &SeveralWlinesInputs<'_>,
    lines: &[Handle<dyn IntPatchLine>],
    c_vertex: &IntPatchPoint,
) -> Result<(i32, i32), Failure> {
    let mut pnt_d_min: f64 = 1.0e+100;
    let (mut vd_min, mut wld_min) = (0, 0);
    let c_pv: Point3 = c_vertex.value();
    let (pcs1, pcs2) = vertex_uv(c_vertex);
    for (slot, line) in lines.iter().enumerate() {
        if line.arc_type() != IntPatchIType::Walking {
            continue;
        }
        let wline: Handle<IntPatchWLine> = line.cast::<IntPatchWLine>();
        for tiv in 1..=wline.nb_vertex() {
            let vertex: IntPatchPoint = wline.vertex(tiv)?;
            let (pts1, pts2) = vertex_uv(&vertex);
            let t_distance: f64 = c_pv.distance(&vertex.value());
            let rmax_s1: f64 = std_max(
                inputs.surf1.u_resolution(t_distance)?,
                inputs.surf1.v_resolution(t_distance)?,
            );
            let rmax_s2: f64 = std_max(
                inputs.surf2.u_resolution(t_distance)?,
                inputs.surf2.v_resolution(t_distance)?,
            );
            if !(rmax_s1 < inputs.max_steps.0 && rmax_s2 < inputs.max_steps.1) {
                continue;
            }
            if pnt_d_min > t_distance && t_distance > precision::p_confusion() {
                let sq_dist1: f64 = pcs1.square_distance(pts1);
                let sq_dist2: f64 = pcs2.square_distance(pts2);
                if sq_dist1 < rmax_s1 * rmax_s1 && sq_dist2 < rmax_s2 * rmax_s2 {
                    (pnt_d_min, vd_min, wld_min) = (t_distance, tiv, slot as i32 + 1);
                }
            }
        }
    }
    Ok((vd_min, wld_min))
}

/// The ranks of the vertices on the line (`VPold`), shifted for the points removed
/// around the vertex `civ` when its neighbours are more than 10 points away; returns
/// the ranks and whether the next and the previous points are removed.
fn shifted_ranks(
    wline: &IntPatchWLine,
    civ: i32,
    cnb_v: i32,
) -> Result<(Vec<i32>, bool, bool), Failure> {
    let mut vp_old: Vec<i32> = Vec::new();
    for ipo in 1..=cnb_v {
        vp_old.push(native_int_cast(wline.vertex(ipo)?.parameter_on_line()));
    }
    let at = |i: i32| -> Result<i32, Failure> {
        crate::int_start::failures::sequence_value(&vp_old, i, "NCollection_Sequence::Value")
            .copied()
    };
    let (mut remove_next, mut remove_prev) = (false, false);
    let mut shift: Vec<(i32, i32)> = Vec::new();
    if civ == 1 {
        if at(civ)?.wrapping_sub(at(civ + 1)?).wrapping_abs() > 10 {
            remove_next = true;
            shift.extend((civ + 1..=cnb_v).map(|i| (i, 1)));
        }
    } else if civ == cnb_v {
        if at(civ)?.wrapping_sub(at(civ - 1)?).wrapping_abs() > 10 {
            remove_prev = true;
            shift.push((civ, 1));
        }
    } else {
        let d_par_mi: i32 = at(civ)?.wrapping_sub(at(civ - 1)?).wrapping_abs();
        let d_par_ma: i32 = at(civ)?.wrapping_sub(at(civ + 1)?).wrapping_abs();
        if d_par_mi > 10 {
            remove_prev = true;
            shift.push((civ, 1));
        }
        if d_par_ma > 10 {
            remove_next = true;
            let by: i32 = if d_par_mi > 10 { 2 } else { 1 };
            shift.extend((civ + 1..=cnb_v).map(|i| (i, by)));
        } else if d_par_mi > 10 {
            shift.extend((civ + 1..=cnb_v).map(|i| (i, 1)));
        }
    }
    for (i, by) in shift {
        vp_old[(i - 1) as usize] -= by;
    }
    Ok((vp_old, remove_next, remove_prev))
}

/// The points of the line with the point `pi` replaced by `replace_pnt` (moved by the
/// periods toward it) and its neighbours removed as flagged.
fn replaced_points(
    wline: &IntPatchWLine,
    (pi, replace_pnt): (i32, &IntSurfPntOn2S),
    (remove_next, remove_prev): (bool, bool),
    periods: &[f64; 4],
) -> Result<Handle<IntSurfLineOn2S>, Failure> {
    let new_l2s: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    for ip in 1..=wline.nb_pnts()? {
        if pi == ip {
            new_l2s.add(&make_new_point(replace_pnt, &wline.point(ip)?, periods));
        } else if (remove_next && ip == pi + 1) || (remove_prev && ip == pi - 1) {
            continue;
        } else {
            new_l2s.add(&wline.point(ip)?);
        }
    }
    Ok(new_l2s)
}

/// The vertex `civ` of `wline` moved onto the vertex `vd_min` of the line `wld_min`.
fn moved_line(
    inputs: &SeveralWlinesInputs<'_>,
    lines: &[Handle<dyn IntPatchLine>],
    wline: &IntPatchWLine,
    (civ, vd_min, wld_min): (i32, i32, i32),
) -> Result<Handle<IntPatchWLine>, Failure> {
    let other: Handle<IntPatchWLine> = lines[(wld_min - 1) as usize].cast::<IntPatchWLine>();
    let tiv_par: i32 = native_int_cast(other.vertex(vd_min)?.parameter_on_line());
    let civ_par: i32 = native_int_cast(wline.vertex(civ)?.parameter_on_line());
    wline.point(civ_par)?;
    let (u21, v21, u22, v22) = other.point(tiv_par)?.parameters();
    let replace_pnt: IntSurfPntOn2S = other.point(tiv_par)?;
    let cnb_v: i32 = wline.nb_vertex();
    let (vp_old, remove_next, remove_prev) = shifted_ranks(wline, civ, cnb_v)?;
    let new_l2s = replaced_points(
        wline,
        (civ_par, &replace_pnt),
        (remove_next, remove_prev),
        &inputs.periods,
    )?;
    let mut new_vtx = IntPatchPoint::new();
    new_vtx.set_value_tangent(other.vertex(vd_min)?.value(), inputs.tol, false);
    new_vtx.set_parameters(u21, v21, u22, v22);
    new_vtx.set_parameter(f64::from(vp_old[(civ - 1) as usize]));
    let nw_line = IntPatchWLine::new(&new_l2s, false, inputs.trans.0, inputs.trans.1);
    nw_line.set_creating_way_info(IntPatchWLType::PrmPrm);
    for iv in 1..=cnb_v {
        if iv == civ {
            nw_line.add_vertex(&new_vtx, false);
            continue;
        }
        let mut the_vtx: IntPatchPoint = wline.vertex(iv)?;
        the_vtx.set_parameter(f64::from(vp_old[(iv - 1) as usize]));
        nw_line.add_vertex(&the_vtx, false);
    }
    Ok(Handle::new(nw_line))
}

/// `SeveralWlinesProcessing(theSurf1, theSurf2, theSLin, thePeriodsArr, theTrans1,
/// theTrans2, theTol, theMaxStepS1, theMaxStepS2, theWLline)`.
pub(super) fn several_wlines_processing(
    inputs: &SeveralWlinesInputs<'_>,
    lines: &[Handle<dyn IntPatchLine>],
    wline: &mut Handle<IntPatchWLine>,
) -> Result<(), Failure> {
    if lines.is_empty() {
        return Ok(());
    }
    // The count of the first line is kept when the line is replaced, as natively.
    let cnb_v: i32 = wline.nb_vertex();
    for civ in 1..=cnb_v {
        let c_vertex: IntPatchPoint = wline.vertex(civ)?;
        let (vd_min, wld_min) = nearest_vertex(inputs, lines, &c_vertex)?;
        if vd_min != 0 {
            *wline = moved_line(inputs, lines, wline, (civ, vd_min, wld_min))?;
        }
    }
    Ok(())
}
