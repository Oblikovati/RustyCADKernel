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
// Rust translation of IntPCo of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The plane against a cone.
use super::super::gline::IntPatchGLine;
use super::super::line::IntPatchLineTransitions;
use super::super::point::IntPatchPoint;
use super::plane::{Lines, Points, crossing_circle, crossing_ellipse, push, tangent_point};
use super::seam::adjust_to_seam;
use super::transitions::{by_sign, by_sign_tol, dot_cross};
use crate::int_surf::{IntSurfQuadric, IntSurfSituation};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Cone3, Hyperbola3, Line3, Parabola3, Plane3, Point3, Vector3};

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// The line oriented along the cone axis, the parameter of the apex on it and the point
/// 5 further.
fn oriented(mut linsol: Line3, co: &Cone3, apex: Point3) -> Result<(Line3, f64, Point3), Failure> {
    if Vector3::from(linsol.direction()).dot(Vector3::from(co.axis().direction())) < 0.0 {
        linsol.set_direction(linsol.direction().reversed());
    }
    let para: f64 = elclib::parameter(&linsol, apex)?;
    let ptbid: Point3 = elclib::value(para + 5.0, &linsol);
    Ok((linsol, para, ptbid))
}

/// `IntPCo(Quad1, Quad2, Tolang, TolTang, Reversed, Empty, Multpoint, slin, spnt)`: the
/// plane and the cone: the apex, one tangency line or two lines through the apex (each
/// in both directions from it), a circle (on the cone seam), an ellipse, a parabola or
/// two hyperbola branches. An empty result is a failure (false), as natively.
#[allow(clippy::too_many_arguments)]
pub(super) fn int_pco(
    quads: Quads<'_>,
    (tolang, tol_tang): (f64, f64),
    reversed: bool,
    empty: &mut bool,
    multpoint: &mut bool,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<bool, Failure> {
    let (pl, co): (Plane3, Cone3) = if reversed {
        (quads.1.plane(), quads.0.cone()?)
    } else {
        (quads.0.plane(), quads.1.cone()?)
    };
    let apex: Point3 = co.apex();
    let mut inter = QuadQuadGeo::new();
    inter.perform_plane_cone(pl, co, tolang, tol_tang)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    match typint {
        ResultType::Point => spnt.push(tangent_point(inter.point(1)?, quads, tol_tang, false)),
        ResultType::Line if nb_sol == 1 => tangency_lines(&inter, &co, apex, (quads, tol_tang, reversed), slin)?,
        ResultType::Line => {
            *multpoint = true;
            two_lines(&inter, &co, apex, (quads, tol_tang), slin)?;
        }
        ResultType::Circle => {
            let mut cirsol: Circle3 = inter.circle(1)?;
            adjust_to_seam(&co.position(), &mut cirsol)?;
            push(slin, crossing_circle(&cirsol, quads)?);
        }
        ResultType::Ellipse => push(slin, crossing_ellipse(&inter.ellipse(1)?, quads)?),
        ResultType::Parabola => push(slin, parabola_line(&inter.parabola(1)?, quads)?),
        ResultType::Hyperbola => {
            for i in 1..=2 {
                push(slin, hyperbola_line(&inter.hyperbola(i)?, quads)?);
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The apex vertex of the lines.
fn apex_point(apex: Point3, quads: Quads<'_>, tol_tang: f64, para: f64) -> IntPatchPoint {
    let mut ptsol: IntPatchPoint = tangent_point(apex, quads, tol_tang, false);
    ptsol.set_parameter(para);
    ptsol
}

/// The TOUCH situations `((plane, cone), (plane, cone) on the other side)` of a
/// tangency line.
fn cone_situations(normp: Vector3, normco: Vector3, test: Vector3) -> [(IntSurfSituation, IntSurfSituation); 2] {
    use IntSurfSituation::{Inside, Outside};
    let same_side: bool = normp.dot(normco) > 0.0;
    if normp.dot(test) > 0.0 {
        let (situp, situp_other) = if same_side { (Inside, Outside) } else { (Outside, Inside) };
        return [(situp, Outside), (situp_other, Inside)];
    }
    let (situp, situp_other) = if same_side { (Outside, Inside) } else { (Inside, Outside) };
    [(situp, Inside), (situp_other, Outside)]
}

/// The tangency line: from the apex along the cone direction, then along the opposite
/// one (the other side), both starting at the apex vertex.
fn tangency_lines(
    inter: &QuadQuadGeo,
    co: &Cone3,
    apex: Point3,
    (quads, tol_tang, reversed): (Quads<'_>, f64, bool),
    slin: &mut Lines,
) -> Result<(), Failure> {
    let (mut linsol, para, ptbid) = oriented(inter.line(1)?, co, apex)?;
    let ptsol: IntPatchPoint = apex_point(apex, quads, tol_tang, para);
    let ptbid2 = Point3::from(apex.xyz() + 5.0 * co.axis().direction().xyz());
    let test_curvature = Vector3::between(ptbid, ptbid2);
    let (normp, normco) = if reversed {
        (quads.1.normale_at(ptbid)?, quads.0.normale_at(ptbid)?)
    } else {
        (quads.0.normale_at(ptbid)?, quads.1.normale_at(ptbid)?)
    };
    for (situp, situco) in cone_situations(normp, normco, test_curvature) {
        let touch = if reversed {
            IntPatchLineTransitions::Touch(situco, situp)
        } else {
            IntPatchLineTransitions::Touch(situp, situco)
        };
        let glig = IntPatchGLine::from_line(&linsol, true, touch)?;
        glig.add_vertex(&ptsol)?;
        glig.set_first_point(1);
        push(slin, glig);
        // -Cone.Direction <------- Apex
        linsol.set_direction(linsol.direction().reversed());
    }
    Ok(())
}

/// Two lines through the apex, each in both directions from it; the transitions of
/// each come from its first direction (the other side keeps them: the line and both
/// normals are reversed there).
fn two_lines(
    inter: &QuadQuadGeo,
    co: &Cone3,
    apex: Point3,
    (quads, tol_tang): (Quads<'_>, f64),
    slin: &mut Lines,
) -> Result<(), Failure> {
    let (linsol, para, ptbid) = oriented(inter.line(1)?, co, apex)?;
    let transitions = by_sign(dot_cross(Vector3::from(linsol.direction()), quads.0, quads.1, ptbid)?);
    let mut ptsol: IntPatchPoint = apex_point(apex, quads, tol_tang, para);
    ptsol.set_multiple(true);
    both_directions(linsol, apex, transitions, &mut ptsol, slin)?;
    let (linsol, para, ptbid) = oriented(inter.line(2)?, co, apex)?;
    let transitions = by_sign(dot_cross(Vector3::from(linsol.direction()), quads.0, quads.1, ptbid)?);
    ptsol.set_parameter(para);
    both_directions(linsol, apex, transitions, &mut ptsol, slin)
}

/// The line and its reverse, each with the apex vertex at its own parameter.
fn both_directions(
    mut linsol: Line3,
    apex: Point3,
    transitions: IntPatchLineTransitions,
    ptsol: &mut IntPatchPoint,
    slin: &mut Lines,
) -> Result<(), Failure> {
    for side in 0..2 {
        if side == 1 {
            linsol.set_direction(linsol.direction().reversed());
        }
        let glig = IntPatchGLine::from_line(&linsol, false, transitions)?;
        ptsol.set_parameter(elclib::parameter(&linsol, apex)?);
        glig.add_vertex(ptsol)?;
        glig.set_first_point(1);
        push(slin, glig);
    }
    Ok(())
}

/// The parabola, oriented by the mixed product of its Y axis at its apex.
fn parabola_line(parabsol: &Parabola3, quads: Quads<'_>) -> Result<IntPatchGLine, Failure> {
    let tgtorig = Vector3::from(parabsol.y_axis().direction());
    let ptran: f64 = dot_cross(tgtorig, quads.0, quads.1, parabsol.location())?;
    Ok(IntPatchGLine::from_parabola(parabsol, false, by_sign_tol(ptran, 0.00000001)))
}

/// A hyperbola branch, oriented by the mixed product of its Y axis at its top.
fn hyperbola_line(hyprsol: &Hyperbola3, quads: Quads<'_>) -> Result<IntPatchGLine, Failure> {
    let tophypr: Point3 = elclib::value(hyprsol.major_radius(), &Line3::from_axis(hyprsol.x_axis()));
    let tgttop = Vector3::from(hyprsol.y_axis().direction());
    let qwe: f64 = dot_cross(tgttop, quads.0, quads.1, tophypr)?;
    Ok(IntPatchGLine::from_hyperbola(hyprsol, false, by_sign_tol(qwe, 0.00000001)))
}
