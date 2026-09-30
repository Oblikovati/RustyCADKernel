// Rust translation of IntPatch_WLineTool::ExtendTwoWLines and the static helper IsNeedSkipWL
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLineTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtendTwoWLines`: every pair of walking lines whose ends are close is extended to a
//! common point, and joined when it can be.
use super::IntPatchWLineTool;
use super::extend::{
    DIS_FIRST_FIRST, DIS_FIRST_LAST, DIS_LAST_FIRST, DIS_LAST_LAST, EndPair, PairState, PointRefs,
    extend_two_wl,
};
use super::extend_checks::{ExtendInputs, is_out_of_domain};
use super::join::wline_at;
use crate::int_patch::{IntPatchLine, IntPatchPointLine, IntPatchWLine};
use crate::int_surf::IntSurfPntOn2S;
use crate::intf::native_int_cast;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `IsNeedSkipWL`: whether the middle of a segment between two vertices of the line is
/// out of the domains.
fn is_need_skip_wl(wl: &IntPatchWLine, inputs: &ExtendInputs<'_>) -> Result<bool, Failure> {
    let nb_vtx: i32 = wl.nb_vertex();
    for i in 1..nb_vtx {
        let firstp: f64 = wl.vertex(i)?.parameter_on_line();
        let lastp: f64 = wl.vertex(i + 1)?.parameter_on_line();
        let pmid: IntSurfPntOn2S = wl.point(native_int_cast((firstp + lastp) / 2.0))?;
        if is_out_of_domain(inputs.box1, inputs.box2, &pmid, &inputs.periods)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether the first and last vertices of the line are its first and last points.
fn has_end_vertices(wl: &IntPatchWLine) -> Result<bool, Failure> {
    if wl.vertex(1)?.parameter_on_line() != 1.0 {
        return Ok(false);
    }
    Ok(wl.vertex(wl.nb_vertex())?.parameter_on_line() == f64::from(wl.nb_pnts()?))
}

/// The references to the first, second, last and last but one points of a line.
struct Ends {
    first: usize,
    second: usize,
    last: usize,
    before_last: usize,
}

impl Ends {
    fn new(wl: &IntPatchWLine, refs: &mut PointRefs) -> Result<Self, Failure> {
        let nb: i32 = wl.nb_pnts()?;
        Ok(Self {
            first: refs.add(wl, 1)?,
            second: refs.add(wl, 2)?,
            last: refs.add(wl, nb)?,
            before_last: refs.add(wl, nb - 1)?,
        })
    }
}

/// Disables in `check` the extensions of the first line blocked by the ends of the line
/// `wl2` (matching in 3d only) and by the critical points; the tests read the bits
/// already set by the previous lines, as natively.
fn disable_ends(
    inputs: &ExtendInputs<'_>,
    (pnt_f1, pnt_l1): (&IntSurfPntOn2S, &IntSurfPntOn2S),
    wl2: &IntPatchWLine,
    (critical, check): (&[Point3], &mut u32),
) -> Result<(), Failure> {
    let (pnt_f2, pnt_l2) = (wl2.point(1)?, wl2.point(wl2.nb_pnts()?)?);
    let tol: f64 = inputs.toler3d;
    let same_2d =
        |a: &IntSurfPntOn2S, b: &IntSurfPntOn2S| a.is_same(b, tol, precision::p_confusion());
    if !same_2d(pnt_f1, &pnt_f2)
        && !same_2d(pnt_f1, &pnt_l2)
        && (pnt_f1.is_same(&pnt_f2, tol, -1.0) || pnt_f1.is_same(&pnt_l2, tol, -1.0))
    {
        *check |= DIS_FIRST_FIRST | DIS_FIRST_LAST;
    }
    if !same_2d(pnt_l1, &pnt_f2)
        && !same_2d(pnt_l1, &pnt_l2)
        && (pnt_l1.is_same(&pnt_f2, tol, -1.0) || pnt_l1.is_same(&pnt_l2, tol, -1.0))
    {
        *check |= DIS_LAST_FIRST | DIS_LAST_LAST;
    }
    for pt in critical {
        let near = |p: &IntSurfPntOn2S| pt.square_distance(&p.value()) < precision::confusion();
        let rules: [(u32, &IntSurfPntOn2S, u32); 4] = [
            (
                DIS_FIRST_FIRST | DIS_FIRST_LAST,
                pnt_f1,
                DIS_FIRST_FIRST | DIS_FIRST_LAST,
            ),
            (
                DIS_LAST_FIRST | DIS_LAST_LAST,
                pnt_l1,
                DIS_LAST_FIRST | DIS_LAST_LAST,
            ),
            (
                DIS_FIRST_FIRST | DIS_LAST_FIRST,
                &pnt_f2,
                DIS_FIRST_FIRST | DIS_LAST_FIRST,
            ),
            (
                DIS_FIRST_LAST | DIS_LAST_LAST,
                &pnt_l2,
                DIS_FIRST_LAST | DIS_LAST_LAST,
            ),
        ];
        for (mask, p, set) in rules {
            if *check & mask == 0 && near(p) {
                *check |= set;
            }
        }
    }
    Ok(())
}

/// The vector from the point `a` to the point `b` of the references.
fn between(refs: &PointRefs, a: usize, b: usize) -> Result<Vector3, Failure> {
    let (pa, pb) = (refs.get(a)?.value(), refs.get(b)?.value());
    Ok(Vector3::from(pb.xyz() - pa.xyz()))
}

/// The four extensions of the pair of lines, in the native order, each unless disabled.
fn extend_pair(
    inputs: &ExtendInputs<'_>,
    pair: &mut PairState<'_>,
    (e1, e2): (&Ends, &Ends),
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    type Step = (u32, EndPair, (usize, usize), [(usize, usize); 3]);
    let steps: [Step; 4] = [
        (
            DIS_FIRST_FIRST,
            EndPair::FirstFirst,
            (e1.first, e2.first),
            [
                (e1.first, e1.second),
                (e2.second, e2.first),
                (e2.first, e1.first),
            ],
        ),
        (
            DIS_FIRST_LAST,
            EndPair::FirstLast,
            (e1.first, e2.last),
            [
                (e1.first, e1.second),
                (e2.before_last, e2.last),
                (e2.last, e1.first),
            ],
        ),
        (
            DIS_LAST_FIRST,
            EndPair::LastFirst,
            (e1.last, e2.first),
            [
                (e1.before_last, e1.last),
                (e2.first, e2.second),
                (e1.last, e2.first),
            ],
        ),
        (
            DIS_LAST_LAST,
            EndPair::LastLast,
            (e1.last, e2.last),
            [
                (e1.before_last, e1.last),
                (e2.last, e2.before_last),
                (e1.last, e2.last),
            ],
        ),
    ];
    for (mask, ends, pts, vec_ends) in steps {
        if *pair.check_result & mask != 0 {
            continue;
        }
        let vecs: [Vector3; 3] = [
            between(refs, vec_ends[0].0, vec_ends[0].1)?,
            between(refs, vec_ends[1].0, vec_ends[1].1)?,
            between(refs, vec_ends[2].0, vec_ends[2].1)?,
        ];
        extend_two_wl(inputs, pair, ends, pts, vecs, refs)?;
    }
    Ok(())
}

/// The second pass of the line `n1`: its extensions with the next lines; returns the
/// number of lines joined to it.
fn extend_with_next(
    inputs: &ExtendInputs<'_>,
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
    (n1, wline1, e1): (i32, &IntPatchWLine, &Ends),
    (check_result, refs): (&mut u32, &mut PointRefs),
) -> Result<u32, Failure> {
    let mut joined_count: u32 = 0;
    let mut n2: i32 = n1 + 1;
    while n2 <= slin.len() as i32 {
        let wline2: Handle<IntPatchWLine> = wline_at(slin, n2)?;
        if wline2.is_null() || !has_end_vertices(&wline2)? {
            n2 += 1;
            continue;
        }
        let e2 = Ends::new(&wline2, refs)?;
        if is_need_skip_wl(&wline2, inputs)? {
            n2 += 1;
            continue;
        }
        let mut has_been_joined = false;
        let mut pair = PairState {
            wline1,
            wline2: &wline2,
            check_result,
            has_been_joined: &mut has_been_joined,
        };
        extend_pair(inputs, &mut pair, (e1, &e2), refs)?;
        if has_been_joined {
            joined_count += 1;
            slin.remove((n2 - 1) as usize);
            n2 -= 1;
        }
        n2 += 1;
    }
    Ok(joined_count)
}

/// The extensions of the line `n1` disabled by all the next lines; `None` when the line
/// is skipped.
fn first_pass(
    inputs: &ExtendInputs<'_>,
    slin: &[Handle<dyn IntPatchLine>],
    (n1, wline1, e1): (i32, &IntPatchWLine, &Ends),
    (refs, critical): (&PointRefs, &[Point3]),
) -> Result<Option<u32>, Failure> {
    if is_need_skip_wl(wline1, inputs)? {
        return Ok(None);
    }
    let mut check: u32 = 0;
    for n2 in n1 + 1..=slin.len() as i32 {
        let wline2: Handle<IntPatchWLine> = wline_at(slin, n2)?;
        if wline2.is_null() {
            continue;
        }
        let (f1, l1) = (refs.get(e1.first)?, refs.get(e1.last)?);
        disable_ends(inputs, (&f1, &l1), &wline2, (critical, &mut check))?;
    }
    let all: u32 = DIS_FIRST_FIRST | DIS_FIRST_LAST | DIS_LAST_FIRST | DIS_LAST_LAST;
    Ok((check != all).then_some(check))
}

impl IntPatchWLineTool {
    /// `ExtendTwoWLines(theSlin, theS1, theS2, theToler3D, theArrPeriods, theBoxS1,
    /// theBoxS2, theListOfCriticalPoints)`: extends the walking lines of `slin` (both
    /// surfaces must be quadrics) to meet in a common point where their ends are close,
    /// so that the gaps between two lines vanish, and joins them there unless the point
    /// is on a boundary. `arr_periods` holds the U and V periods of both surfaces (0 on a
    /// non periodic direction); no line is extended to a critical point.
    ///
    /// ```ignore
    /// IntPatchWLineTool::extend_two_wlines(&mut lines, &s1, &s2, 1e-7, &periods, &box1, &box2, &[])?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn extend_two_wlines(
        slin: &mut Vec<Handle<dyn IntPatchLine>>,
        s1: &Adaptor3dSurfaceHandle,
        s2: &Adaptor3dSurfaceHandle,
        toler3d: f64,
        arr_periods: &[f64; 4],
        box_s1: &BoundingBox2,
        box_s2: &BoundingBox2,
        list_of_critical_points: &[Point3],
    ) -> Result<(), Failure> {
        let inputs = ExtendInputs {
            s1,
            s2,
            toler3d,
            periods: *arr_periods,
            box1: box_s1,
            box2: box_s2,
        };
        extend_two_wlines(slin, &inputs, list_of_critical_points)
    }
}

/// `IntPatch_WLineTool::ExtendTwoWLines` (see
/// [`IntPatchWLineTool::extend_two_wlines`]).
fn extend_two_wlines(
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
    inputs: &ExtendInputs<'_>,
    critical: &[Point3],
) -> Result<(), Failure> {
    if slin.len() < 2 {
        return Ok(());
    }
    let mut joined_counter: u32 = 0;
    let mut n1: i32 = 1;
    while n1 <= slin.len() as i32 {
        if joined_counter > 0 {
            n1 -= 1;
        }
        joined_counter = 0;
        let wline1: Handle<IntPatchWLine> = wline_at(slin, n1)?;
        if wline1.is_some() && has_end_vertices(&wline1)? {
            let mut refs = PointRefs::default();
            let e1 = Ends::new(&wline1, &mut refs)?;
            if let Some(mut check) =
                first_pass(inputs, slin, (n1, &wline1, &e1), (&refs, critical))?
            {
                joined_counter =
                    extend_with_next(inputs, slin, (n1, &wline1, &e1), (&mut check, &mut refs))?;
            }
        }
        n1 += 1;
    }
    Ok(())
}
