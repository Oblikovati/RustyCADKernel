// Rust translation of ProcessBounds and of the analytic-curve loops of IntCySp, IntCyCo,
// IntCoCo and IntCoSp of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytic lines of two quadrics (`IntAna_IntQuadQuad` curves) and their end
//! vertices, shared with the lines already found.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::super::IntPatchIType;
use super::super::aline::IntPatchALine;
use super::super::line::IntPatchLineTransitions;
use super::super::point::IntPatchPoint;
use super::plane::{Lines, Points, tangent_point};
use super::transitions::{by_sign_tol, dot_cross};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana::{Curve as IntAnaCurve, IntQuadQuad};
use rustycad_tkmath::gp::{Point3, Vector3};

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// The end of an analytic line: its point, its parameter and whether it is processed
/// (a vertex is already there).
pub(super) struct LineEnd {
    pub(super) proc: bool,
    pub(super) pt: Point3,
    pub(super) param: f64,
}

/// `ProcessBounds(alig, slin, Quad1, Quad2, procf, ptf, first, procl, ptl, last,
/// Multpoint, Tol)`: each end of the new line that is within `tol` of a vertex of an
/// analytic line already found takes that vertex (made multiple there, which sets
/// `multpoint`); the other ends get new vertices (one multiple vertex when both ends
/// are the same point). The vertex read last is the base of the new ones (its arcs and
/// transitions are kept by `SetValue`), as natively.
pub(super) fn process_bounds(
    alig: &IntPatchALine,
    slin: &Lines,
    quads: Quads<'_>,
    (first, last): (&mut LineEnd, &mut LineEnd),
    multpoint: &mut bool,
    tol: f64,
) -> Result<(), Failure> {
    let mut ptsol = IntPatchPoint::new();
    if !(first.proc && last.proc) {
        for line in slin {
            if line.arc_type() != IntPatchIType::Analytic {
                // Natively the loop does not advance on such a line (an endless loop);
                // no caller has one in its list.
                return Err(Failure::with_message(
                    "Standard_ProgramError",
                    "ProcessBounds() - a line of the list is not analytic; expected IntPatch_ALine only",
                ));
            }
            shared_vertices(
                alig,
                &line.cast(),
                (first, last),
                multpoint,
                tol,
                &mut ptsol,
            )?;
            if first.proc && last.proc {
                break;
            }
        }
    }
    new_end_vertices(alig, quads, (first, last), multpoint, tol, &mut ptsol)
}

/// The vertices of the analytic line `aligold` at the free ends of the new line.
fn shared_vertices(
    alig: &IntPatchALine,
    aligold: &Handle<IntPatchALine>,
    (first, last): (&mut LineEnd, &mut LineEnd),
    multpoint: &mut bool,
    tol: f64,
    ptsol: &mut IntPatchPoint,
) -> Result<(), Failure> {
    let mut k: i32 = 1;
    while k <= aligold.nb_vertex() {
        *ptsol = aligold.vertex(k)?;
        if !first.proc {
            share_end(alig, aligold, k, first, multpoint, tol, ptsol, true)?;
        }
        if !last.proc {
            share_end(alig, aligold, k, last, multpoint, tol, ptsol, false)?;
        }
        if first.proc && last.proc {
            break;
        }
        k += 1;
    }
    Ok(())
}

/// The vertex `k` of `aligold` near the end becomes the first (or last) vertex of the
/// new line; it is restored with its own parameter afterwards.
#[allow(clippy::too_many_arguments)]
fn share_end(
    alig: &IntPatchALine,
    aligold: &Handle<IntPatchALine>,
    k: i32,
    end: &mut LineEnd,
    multpoint: &mut bool,
    tol: f64,
    ptsol: &mut IntPatchPoint,
    is_first: bool,
) -> Result<(), Failure> {
    if !(end.pt.distance(&ptsol.value()) <= tol) {
        return Ok(());
    }
    ptsol.set_tolerance(tol);
    if !ptsol.is_multiple() {
        // The point of aligold is declared multiple on aligold.
        *multpoint = true;
        ptsol.set_multiple(true);
        aligold.replace(k, ptsol)?;
    }
    ptsol.set_parameter(end.param);
    alig.add_vertex(ptsol);
    if is_first {
        alig.set_first_point(alig.nb_vertex());
    } else {
        alig.set_last_point(alig.nb_vertex());
    }
    end.proc = true;
    // Restore the point with its parameter on aligold.
    *ptsol = aligold.vertex(k)?;
    Ok(())
}

/// The vertex of an end on both quadrics.
fn end_vertex(ptsol: &mut IntPatchPoint, quads: Quads<'_>, end: &LineEnd, tol: f64) {
    let (u1, v1) = quads.0.parameters(end.pt).unwrap_or((0.0, 0.0));
    let (u2, v2) = quads.1.parameters(end.pt).unwrap_or((0.0, 0.0));
    ptsol.set_value_tangent(end.pt, tol, false);
    ptsol.set_parameters(u1, v1, u2, v2);
    ptsol.set_parameter(end.param);
}

/// The new vertices of the ends that no vertex took.
fn new_end_vertices(
    alig: &IntPatchALine,
    quads: Quads<'_>,
    (first, last): (&LineEnd, &LineEnd),
    multpoint: &mut bool,
    tol: f64,
    ptsol: &mut IntPatchPoint,
) -> Result<(), Failure> {
    ptsol.set_tolerance(tol);
    if !first.proc && !last.proc {
        end_vertex(ptsol, quads, first, tol);
        if first.pt.distance(&last.pt) <= tol {
            ptsol.set_multiple(true); // to be seen
            *multpoint = true; // to be seen as well
            alig.add_vertex(ptsol);
            alig.set_first_point(alig.nb_vertex());
            ptsol.set_parameter(last.param);
        } else {
            alig.add_vertex(ptsol);
            alig.set_first_point(alig.nb_vertex());
            end_vertex(ptsol, quads, last, tol);
        }
        alig.add_vertex(ptsol);
        alig.set_last_point(alig.nb_vertex());
    } else if !first.proc {
        end_vertex(ptsol, quads, first, tol);
        alig.add_vertex(ptsol);
        alig.set_first_point(alig.nb_vertex());
    } else if !last.proc {
        end_vertex(ptsol, quads, last, tol);
        alig.add_vertex(ptsol);
        alig.set_last_point(alig.nb_vertex());
    }
    Ok(())
}

/// The variants of the analytic-curve loop of the pair solvers.
pub(super) struct CurveOptions {
    /// IntCoCo: a tangent under 1e-14 (squared) is no tangent (the normals make no sense).
    pub(super) check_tangent: bool,
    /// The threshold of the transitions (1e-8 or 1e-9).
    pub(super) tol_trans: f64,
    /// IntCySp does not test the open ends (`firstp = lastp = true`).
    pub(super) open_ends: bool,
    /// The native code evaluates the curve at the last parameter tried when no tangent
    /// was found (IntCyCo, IntCoCo, IntCoSp), which can raise.
    pub(super) eval_on_fail: bool,
}

/// The isolated points of the analytic intersection (tangent points).
pub(super) fn analytic_points(
    anaint: &IntQuadQuad,
    quads: Quads<'_>,
    tol: f64,
    spnt: &mut Points,
) -> Result<(), Failure> {
    for i in 1..=anaint.nb_pnt()? {
        spnt.push(tangent_point(anaint.point(i)?, quads, tol, true));
    }
    Ok(())
}

/// The analytic line of one curve: the transitions at the first of up to 5 parameters
/// (halving toward the first one) where the curve has a tangent, undecided without;
/// then `ProcessBounds` and the line appended.
pub(super) fn analytic_line(
    curvsol: &IntAnaCurve,
    quads: Quads<'_>,
    opts: &CurveOptions,
    (slin, multpoint): (&mut Lines, &mut bool),
    tol: f64,
    ends: &mut (Point3, Point3),
) -> Result<(), Failure> {
    let (first, last) = curvsol.domain()?;
    let firstp: bool = !opts.open_ends || !curvsol.is_first_open();
    let lastp: bool = !opts.open_ends || !curvsol.is_last_open();
    if firstp {
        ends.0 = curvsol.value(first)?;
    }
    if lastp {
        ends.1 = curvsol.value(last)?;
    }
    let transitions = curve_transitions(curvsol, quads, opts, (first, last))?;
    let alig = Handle::new(IntPatchALine::new(curvsol, false, transitions));
    let mut f = LineEnd {
        proc: !firstp,
        pt: ends.0,
        param: first,
    };
    let mut l = LineEnd {
        proc: !lastp,
        pt: ends.1,
        param: last,
    };
    process_bounds(&alig, slin, quads, (&mut f, &mut l), multpoint, tol)?;
    slin.push(alig.cast());
    Ok(())
}

/// The transitions of the analytic line from its tangent.
fn curve_transitions(
    curvsol: &IntAnaCurve,
    quads: Quads<'_>,
    opts: &CurveOptions,
    (first, last): (f64, f64),
) -> Result<IntPatchLineTransitions, Failure> {
    let (mut ptvalid, mut tgvalid) = (Point3::default(), Vector3::default());
    let mut para: f64 = last;
    let mut kount: i32 = 1;
    loop {
        para = (1.123 * first + para) / 2.123;
        let mut tgfound: bool = curvsol.d1u(para, &mut ptvalid, &mut tgvalid)?;
        if opts.check_tangent && tgvalid.square_magnitude() < 1e-14 {
            // The normals would make no sense here.
            tgfound = false;
        }
        if tgfound {
            break;
        }
        kount += 1;
        if kount > 5 {
            break;
        }
    }
    if kount <= 5 {
        let qwe: f64 = dot_cross(tgvalid, quads.0, quads.1, ptvalid)?;
        return Ok(by_sign_tol(qwe, opts.tol_trans));
    }
    if opts.eval_on_fail {
        curvsol.value(para)?;
    }
    Ok(IntPatchLineTransitions::Undecided)
}
