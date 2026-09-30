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
// Rust translation of IntCoSp of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cone against a sphere.
use super::super::gline::IntPatchGLine;
use super::super::line::IntPatchLineTransitions;
use super::bounds::{CurveOptions, analytic_line, analytic_points};
use super::plane::{Lines, Points, push, tangent_point};
use super::transitions::{by_sign_tol, dot_cross};
use crate::int_surf::{IntSurfQuadric, IntSurfTypeTrans};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{IntQuadQuad, QuadQuadGeo, Quadric, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Cone3, Point3, Sphere3};

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// `IntCoSp(Quad1, Quad2, Tol, Reversed, Empty, Multpoint, slin, spnt)`: the cone and
/// the sphere: points (the apex, or tangent points on the cone half above it), circles,
/// the apex and a circle, or the analytic curves.
#[allow(clippy::too_many_arguments)]
pub(super) fn int_cosp(
    quads: Quads<'_>,
    tol: f64,
    reversed: bool,
    empty: &mut bool,
    multpoint: &mut bool,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<bool, Failure> {
    let (co, sp): (Cone3, Sphere3) = if reversed {
        (quads.1.cone()?, quads.0.sphere()?)
    } else {
        (quads.0.cone()?, quads.1.sphere()?)
    };
    let mut inter = QuadQuadGeo::new();
    inter.perform_sphere_cone(sp, co, tol)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Point => contact_points(&inter, &co, quads, tol, nb_sol, spnt)?,
        ResultType::Circle => {
            for i in 1..=nb_sol {
                let cirsol: Circle3 = inter.circle(i)?;
                let (ptref, tgt) = elclib::d1(0.0, &cirsol);
                let qwe: f64 = dot_cross(tgt, quads.0, quads.1, ptref)?;
                push(slin, IntPatchGLine::from_circle(&cirsol, false, by_sign_tol(qwe, 0.00000001)));
            }
        }
        ResultType::PointAndCircle => {
            // The point is necessarily the apex.
            spnt.push(tangent_point(co.apex(), quads, tol, false));
            push(slin, apex_side_circle(&inter.circle(1)?, &co, quads)?);
        }
        ResultType::NoGeometricSolution => {
            let anaint = IntQuadQuad::cone(co, &Quadric::from_sphere(sp), tol)?;
            return analytic_curves(&anaint, quads, tol, (empty, multpoint), (slin, spnt));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The contact points: at the apex (not tangent), or tangent on the cone above it.
fn contact_points(
    inter: &QuadQuadGeo,
    co: &Cone3,
    quads: Quads<'_>,
    tol: f64,
    nb_sol: i32,
    spnt: &mut Points,
) -> Result<(), Failure> {
    let apex: Point3 = co.apex();
    let paramapex: f64 = elclib::line_parameter(co.axis(), apex);
    for i in 1..=nb_sol {
        let ptcontact: Point3 = inter.point(i)?;
        let param: f64 = elclib::line_parameter(co.axis(), ptcontact);
        if apex.distance(&ptcontact) <= tol {
            spnt.push(tangent_point(ptcontact, quads, tol, false));
        } else if param >= paramapex {
            spnt.push(tangent_point(ptcontact, quads, tol, true));
        }
    }
    Ok(())
}

/// The circle of a point-and-circle result: its transitions are reversed below the
/// apex (PConfusion as threshold).
fn apex_side_circle(cirsol: &Circle3, co: &Cone3, quads: Quads<'_>) -> Result<IntPatchGLine, Failure> {
    let paramapex: f64 = elclib::line_parameter(co.axis(), co.apex());
    let param: f64 = elclib::line_parameter(co.axis(), cirsol.location());
    let (ptref, tgt) = elclib::d1(0.0, cirsol);
    let qwe: f64 = dot_cross(tgt, quads.0, quads.1, ptref)?;
    let transitions = if param >= paramapex {
        by_sign_tol(qwe, precision::p_confusion())
    } else {
        let (out, inn, und) = (IntSurfTypeTrans::Out, IntSurfTypeTrans::In, IntSurfTypeTrans::Undecided);
        let (t1, t2) = if qwe < -precision::p_confusion() {
            (out, inn)
        } else if qwe > precision::p_confusion() {
            (inn, out)
        } else {
            (und, und)
        };
        IntPatchLineTransitions::Types(t1, t2)
    };
    Ok(IntPatchGLine::from_circle(cirsol, false, transitions))
}

/// The points and curves of the analytic intersection (open ends tested, 1e-9 for the
/// transitions, the curve evaluated when no tangent is found).
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
    let opts = CurveOptions { check_tangent: false, tol_trans: 0.000000001, open_ends: true, eval_on_fail: true };
    let mut ends: (Point3, Point3) = (Point3::default(), Point3::default());
    for i in 1..=anaint.nb_curve()? {
        analytic_line(anaint.curve(i)?, quads, &opts, (slin, multpoint), tol, &mut ends)?;
    }
    Ok(true)
}
