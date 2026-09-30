// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of IntCoCo of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Two cones.
use super::super::gline::IntPatchGLine;
use super::super::line::IntPatchLineTransitions;
use super::super::point::IntPatchPoint;
use super::bounds::{CurveOptions, analytic_line, analytic_points};
use super::plane::{Lines, Points, push, tangent_point};
use super::transitions::{by_sign, by_sign_tol, dot_cross};
use crate::int_surf::{IntSurfQuadric, IntSurfSituation, IntSurfTypeTrans};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gce::MakeLin;
use rustycad_tkgeombase::int_ana::{IntQuadQuad, QuadQuadGeo, Quadric, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Cone3, Line3, Point3, Vector3};

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// `IntCoCo(Quad1, Quad2, Tol, Empty, Same, Multpoint, slin, spnt)`: two cones: the same
/// cone, a tangency line or two lines through the first apex (each in both directions),
/// contact points, conics (with the characteristic point of a common generatrix as
/// vertex), or the analytic curves; a common generatrix through both apexes is added
/// as a tangent line (NIZNHY-PKV, 30 Nov 2005).
#[allow(clippy::too_many_arguments)]
pub(super) fn int_coco(
    quads: Quads<'_>,
    tol: f64,
    empty: &mut bool,
    same: &mut bool,
    multpoint: &mut bool,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<bool, Failure> {
    let (co1, co2): (Cone3, Cone3) = (quads.0.cone()?, quads.1.cone()?);
    let mut inter = QuadQuadGeo::new();
    inter.perform_cones(co1, co2, tol)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    *same = false;
    let cones = (&co1, &co2);
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Same => *same = true,
        ResultType::Line if nb_sol == 1 => tangency_line(&inter, cones, quads, tol, slin)?,
        ResultType::Line if nb_sol == 2 => {
            *multpoint = true;
            for i in 1..=2 {
                crossing_lines(&inter.line(i)?, co1.apex(), quads, tol, slin)?;
            }
        }
        ResultType::Line => {}
        ResultType::Point => contact_points(&inter, cones, quads, tol, nb_sol, spnt)?,
        ResultType::Circle | ResultType::Ellipse | ResultType::Hyperbola | ResultType::Parabola => {
            conics(&inter, typint, quads, tol, slin)?;
        }
        ResultType::NoGeometricSolution => {
            let anaint = IntQuadQuad::cone(co1, &Quadric::from_cone(co2), tol)?;
            if !analytic_curves(&anaint, quads, tol, (empty, multpoint), (slin, spnt))? {
                return Ok(false);
            }
        }
        _ => return Ok(false),
    }
    if inter.has_common_gen() {
        push(slin, common_generatrix(&inter, cones, quads, tol)?);
    }
    Ok(true)
}

/// The apex vertex at the parameter `para`, with the parameters of the first apex.
fn apex_vertex(apex: Point3, quads: Quads<'_>, tol: f64, para: f64) -> IntPatchPoint {
    let mut pt: IntPatchPoint = tangent_point(apex, quads, tol, false);
    pt.set_parameter(para);
    pt
}

/// The situations of the tangency line: both outside for opposite normals; otherwise
/// the cone farther from the point 5 along the line is outside (the line goes from the
/// first apex to the second, so the second cone is inside the first).
fn situations(
    cones: (&Cone3, &Cone3),
    quads: Quads<'_>,
    ptbid: Point3,
) -> Result<(IntSurfSituation, IntSurfSituation), Failure> {
    use IntSurfSituation::{Inside, Outside};
    let (norm_c1, norm_c2) = (quads.0.normale_at(ptbid)?, quads.1.normale_at(ptbid)?);
    if norm_c1.dot(norm_c2) < 0.0 {
        return Ok((Outside, Outside));
    }
    let lax1 = Line3::new(cones.0.apex(), cones.0.axis().direction());
    let lax2 = Line3::new(cones.1.apex(), cones.1.axis().direction());
    let (r1, r2) = (lax1.distance(ptbid), lax2.distance(ptbid));
    if r1 > r2 {
        return Ok((Outside, Inside));
    }
    Ok((Inside, Outside))
}

/// The tangency line from the first apex, then its reverse with swapped situations.
fn tangency_line(
    inter: &QuadQuadGeo,
    cones: (&Cone3, &Cone3),
    quads: Quads<'_>,
    tol: f64,
    slin: &mut Lines,
) -> Result<(), Failure> {
    let apex1: Point3 = cones.0.apex();
    let mut linsol: Line3 = inter.line(1)?;
    let para: f64 = elclib::parameter(&linsol, apex1)?;
    let ptbid: Point3 = elclib::value(para + 5.0, &linsol);
    let mut ptsol: IntPatchPoint = apex_vertex(apex1, quads, tol, para);
    let (situ_c1, situ_c2) = situations(cones, quads, ptbid)?;
    let glig = IntPatchGLine::from_line(
        &linsol,
        true,
        IntPatchLineTransitions::Touch(situ_c1, situ_c2),
    )?;
    glig.add_vertex(&ptsol)?;
    glig.set_first_point(1);
    push(slin, glig);
    linsol.set_direction(linsol.direction().reversed());
    ptsol.set_parameter(elclib::parameter(&linsol, apex1)?);
    let glig = IntPatchGLine::from_line(
        &linsol,
        true,
        IntPatchLineTransitions::Touch(situ_c2, situ_c1),
    )?;
    glig.add_vertex(&ptsol)?;
    glig.set_first_point(1);
    push(slin, glig);
    Ok(())
}

/// A crossing line through the first apex, in both directions with the same
/// transitions and a multiple apex vertex.
fn crossing_lines(
    linsol: &Line3,
    apex1: Point3,
    quads: Quads<'_>,
    tol: f64,
    slin: &mut Lines,
) -> Result<(), Failure> {
    let mut linsol: Line3 = *linsol;
    let para: f64 = elclib::parameter(&linsol, apex1)?;
    let ptbid: Point3 = elclib::value(para + 5.0, &linsol);
    let transitions = by_sign(dot_cross(
        Vector3::from(linsol.direction()),
        quads.0,
        quads.1,
        ptbid,
    )?);
    let mut ptsol: IntPatchPoint = apex_vertex(apex1, quads, tol, para);
    ptsol.set_multiple(true);
    for side in 0..2 {
        if side == 1 {
            linsol.set_direction(linsol.direction().reversed());
            ptsol.set_parameter(elclib::parameter(&linsol, apex1)?);
        }
        let glig = IntPatchGLine::from_line(&linsol, false, transitions)?;
        glig.add_vertex(&ptsol)?;
        glig.set_first_point(1);
        push(slin, glig);
    }
    Ok(())
}

/// The contact points: at both apexes (not tangent), or tangent above both apexes.
fn contact_points(
    inter: &QuadQuadGeo,
    (co1, co2): (&Cone3, &Cone3),
    quads: Quads<'_>,
    tol: f64,
    nb_sol: i32,
    spnt: &mut Points,
) -> Result<(), Failure> {
    let (apex1, apex2) = (co1.apex(), co2.apex());
    let paramapex1: f64 = elclib::line_parameter(co1.axis(), apex1);
    let paramapex2: f64 = elclib::line_parameter(co2.axis(), apex2);
    for i in 1..=nb_sol {
        let ptcontact: Point3 = inter.point(i)?;
        let param1: f64 = elclib::line_parameter(co1.axis(), ptcontact);
        let param2: f64 = elclib::line_parameter(co2.axis(), ptcontact);
        if apex1.distance(&ptcontact) <= tol && apex2.distance(&ptcontact) <= tol {
            spnt.push(tangent_point(ptcontact, quads, tol, false));
        } else if param1 >= paramapex1 && param2 >= paramapex2 {
            spnt.push(tangent_point(ptcontact, quads, tol, true));
        }
    }
    Ok(())
}

/// The circles, ellipse, hyperbola branches or parabola, each with the characteristic
/// point of a common generatrix as vertex at parameter 0.
fn conics(
    inter: &QuadQuadGeo,
    typint: ResultType,
    quads: Quads<'_>,
    tol: f64,
    slin: &mut Lines,
) -> Result<(), Failure> {
    let lines: Vec<IntPatchGLine> = match typint {
        ResultType::Circle => {
            let mut out = Vec::new();
            for i in 1..=inter.nb_solutions()? {
                let c: Circle3 = inter.circle(i)?;
                let (ptref, tgt) = elclib::d1(0.0, &c);
                out.push(IntPatchGLine::from_circle(
                    &c,
                    false,
                    conic_transitions(tgt, quads, ptref)?,
                ));
            }
            out
        }
        ResultType::Ellipse => {
            let e = inter.ellipse(1)?;
            let (ptref, tgt) = elclib::d1(0.0, &e);
            vec![IntPatchGLine::from_ellipse(
                &e,
                false,
                conic_transitions(tgt, quads, ptref)?,
            )]
        }
        ResultType::Hyperbola => {
            let mut out = Vec::new();
            for i in 1..=2 {
                let h = inter.hyperbola(i)?;
                let (ptref, tgt) = elclib::d1(0.0, &h);
                out.push(IntPatchGLine::from_hyperbola(
                    &h,
                    false,
                    conic_transitions(tgt, quads, ptref)?,
                ));
            }
            out
        }
        _ => {
            let p = inter.parabola(1)?;
            let tgtorig = Vector3::from(p.y_axis().direction());
            vec![IntPatchGLine::from_parabola(
                &p,
                false,
                conic_transitions(tgtorig, quads, p.location())?,
            )]
        }
    };
    for glig in lines {
        if inter.has_common_gen() {
            let mut pt: IntPatchPoint = tangent_point(inter.p_char(), quads, tol, false);
            pt.set_parameter(0.0);
            glig.add_vertex(&pt)?;
        }
        push(slin, glig);
    }
    Ok(())
}

fn conic_transitions(
    tgt: Vector3,
    quads: Quads<'_>,
    p: Point3,
) -> Result<IntPatchLineTransitions, Failure> {
    Ok(by_sign_tol(
        dot_cross(tgt, quads.0, quads.1, p)?,
        0.00000001,
    ))
}

/// The common generatrix through both apexes, undecided and tangent, with the
/// characteristic point as vertex.
fn common_generatrix(
    inter: &QuadQuadGeo,
    (co1, co2): (&Cone3, &Cone3),
    quads: Quads<'_>,
    tol: f64,
) -> Result<IntPatchGLine, Failure> {
    let linsol: Line3 = MakeLin::from_points(co1.apex(), co2.apex())?.value()?;
    let und = IntSurfTypeTrans::Undecided;
    let glig = IntPatchGLine::from_line(&linsol, true, IntPatchLineTransitions::Types(und, und))?;
    let pchar: Point3 = inter.p_char();
    let mut pt: IntPatchPoint = tangent_point(pchar, quads, tol, false);
    pt.set_parameter(elclib::parameter(&linsol, pchar)?);
    glig.add_vertex(&pt)?;
    Ok(glig)
}

/// The points and curves of the analytic intersection (a tangent under 1e-14 is no
/// tangent, 1e-9 for the transitions).
fn analytic_curves(
    anaint: &IntQuadQuad,
    quads: Quads<'_>,
    tol: f64,
    (empty, multpoint): (&mut bool, &mut bool),
    (slin, spnt): (&mut Lines, &mut Points),
) -> Result<bool, Failure> {
    if !anaint.is_done() {
        return Ok(false);
    }
    if anaint.nb_pnt()? == 0 && anaint.nb_curve()? == 0 {
        *empty = true;
        return Ok(true);
    }
    analytic_points(anaint, quads, tol, spnt)?;
    let opts = CurveOptions {
        check_tangent: true,
        tol_trans: 0.000000001,
        open_ends: true,
        eval_on_fail: true,
    };
    let mut ends: (Point3, Point3) = (Point3::default(), Point3::default());
    for i in 1..=anaint.nb_curve()? {
        analytic_line(
            anaint.curve(i)?,
            quads,
            &opts,
            (slin, multpoint),
            tol,
            &mut ends,
        )?;
    }
    Ok(true)
}
