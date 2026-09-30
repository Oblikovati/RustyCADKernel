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
// Rust translation of IntCySp of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cylinder against a sphere.
use super::super::gline::IntPatchGLine;
use super::super::line::IntPatchLineTransitions;
use super::bounds::{CurveOptions, analytic_line, analytic_points};
use super::plane::{Lines, Points, push, tangent_point};
use super::transitions::{by_sign, by_sign_tol, dot_cross};
use crate::int_surf::{IntSurfQuadric, IntSurfSituation};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{IntQuadQuad, QuadQuadGeo, Quadric, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Cylinder3, Point3, Sphere3, Vector3};

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// `IntCySp(Quad1, Quad2, Tol, Reversed, Empty, Multpoint, slin, spnt)`: the cylinder
/// and the sphere: a tangent point, one tangency circle or two circles, or the analytic
/// curves of `IntAna_IntQuadQuad`.
#[allow(clippy::too_many_arguments)]
pub(super) fn int_cysp(
    quads: Quads<'_>,
    tol: f64,
    reversed: bool,
    empty: &mut bool,
    multpoint: &mut bool,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<bool, Failure> {
    let (cy, sp): (Cylinder3, Sphere3) = if reversed {
        (quads.1.cylinder()?, quads.0.sphere()?)
    } else {
        (quads.0.cylinder()?, quads.1.sphere()?)
    };
    let mut inter = QuadQuadGeo::new();
    inter.perform_cylinder_sphere(cy, sp, tol)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Point => spnt.push(tangent_point(inter.point(1)?, quads, tol, true)),
        ResultType::Circle if nb_sol == 1 => push(slin, tangency_circle(&inter.circle(1)?, &sp, quads, reversed)?),
        ResultType::Circle => {
            let c1: Circle3 = inter.circle(1)?;
            let (ptref, tgt) = elclib::d1(0.0, &c1);
            push(slin, IntPatchGLine::from_circle(&c1, false, by_sign(dot_cross(tgt, quads.0, quads.1, ptref)?)));
            let c2: Circle3 = inter.circle(2)?;
            let (ptref, tgt) = elclib::d1(0.0, &c2);
            let qwe: f64 = dot_cross(tgt, quads.0, quads.1, ptref)?;
            push(slin, IntPatchGLine::from_circle(&c2, false, by_sign_tol(qwe, 0.0000001)));
        }
        ResultType::NoGeometricSolution => {
            let anaint = IntQuadQuad::cylinder(cy, &Quadric::from_sphere(sp), tol)?;
            return analytic_curves(&anaint, quads, tol, (empty, multpoint), (slin, spnt));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The tangency circle, with TOUCH transitions from the curvature toward the sphere
/// center.
fn tangency_circle(cirsol: &Circle3, sp: &Sphere3, quads: Quads<'_>, reversed: bool) -> Result<IntPatchGLine, Failure> {
    use IntSurfSituation::{Inside, Outside};
    let (ptref, _) = elclib::d1(0.0, cirsol);
    let test_curvature = Vector3::between(ptref, sp.location());
    let (normcyl, normsp) = if reversed {
        (quads.1.normale_at(ptref)?, quads.0.normale_at(ptref)?)
    } else {
        (quads.0.normale_at(ptref)?, quads.1.normale_at(ptref)?)
    };
    let same_side: bool = normsp.dot(normcyl) > 0.0;
    let (situsp, situcyl) = if normcyl.dot(test_curvature) > 0.0 {
        (Outside, if same_side { Inside } else { Outside })
    } else {
        (Inside, if same_side { Outside } else { Inside })
    };
    let touch = if reversed {
        IntPatchLineTransitions::Touch(situsp, situcyl)
    } else {
        IntPatchLineTransitions::Touch(situcyl, situsp)
    };
    Ok(IntPatchGLine::from_circle(cirsol, true, touch))
}

/// The points and curves of the analytic intersection (no open-end test, 1e-8 for the
/// transitions); empty when there is none.
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
    let opts = CurveOptions { check_tangent: false, tol_trans: 0.00000001, open_ends: false, eval_on_fail: false };
    let mut ends: (Point3, Point3) = (Point3::default(), Point3::default());
    for i in 1..=anaint.nb_curve()? {
        analytic_line(anaint.curve(i)?, quads, &opts, (slin, multpoint), tol, &mut ends)?;
    }
    Ok(true)
}
