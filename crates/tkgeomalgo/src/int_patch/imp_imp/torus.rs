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
// Rust translation of IntPTo, IntCyTo, IntCoTo, IntSpTo, IntToTo and TreatResultTorus
// of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The torus against a plane, a cylinder, a cone, a sphere or a torus: the circles of
//! the analytic intersection.
use super::super::gline::IntPatchGLine;
use super::plane::{Lines, crossing_circle, push};
use super::seam::adjust_to_seam;
use super::transitions::{by_sign_tol, dot_cross};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Plane3, Torus3};

type Quads<'a> = (&'a IntSurfQuadric, &'a IntSurfQuadric);

/// The torus of the pair and the other quadric, by `reversed`.
fn torus_of(quads: Quads<'_>, reversed: bool) -> Result<Torus3, Failure> {
    if reversed {
        quads.0.torus()
    } else {
        quads.1.torus()
    }
}

/// The other quadric of the pair.
fn other_of<'a>(quads: Quads<'a>, reversed: bool) -> &'a IntSurfQuadric {
    if reversed { quads.1 } else { quads.0 }
}

/// `IntPTo(theQuad1, theQuad2, theTolTang, bReversed, bEmpty, theSeqLin)`: the circles
/// of the plane and the torus, moved to the torus seam unless the plane axis is normal
/// to the torus axis.
pub(super) fn int_pto(
    quads: Quads<'_>,
    tol_tang: f64,
    reversed: bool,
    empty: &mut bool,
    slin: &mut Lines,
) -> Result<bool, Failure> {
    let pln: Plane3 = other_of(quads, reversed).plane();
    let torus: Torus3 = torus_of(quads, reversed)?;
    let mut inter = QuadQuadGeo::new();
    inter.perform_plane_torus(pln, torus, tol_tang)?;
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Circle => {
            for i in 1..=nb_sol {
                let mut c: Circle3 = inter.circle(i)?;
                if !pln.axis().is_normal(torus.axis(), precision::angular()) {
                    adjust_to_seam(&torus.position(), &mut c)?;
                }
                push(slin, crossing_circle(&c, quads)?);
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `IntCyTo`, `IntCoTo` and `IntSpTo`: the analytic intersection of the quadric and the
/// torus, treated by `TreatResultTorus`.
pub(super) fn int_quadric_torus(
    quads: Quads<'_>,
    tol_tang: f64,
    reversed: bool,
    empty: &mut bool,
    slin: &mut Lines,
) -> Result<bool, Failure> {
    let torus: Torus3 = torus_of(quads, reversed)?;
    let other: &IntSurfQuadric = other_of(quads, reversed);
    let mut inter = QuadQuadGeo::new();
    match other.type_quadric() {
        rustycad_tkmath::geom_abs::SurfaceType::Cylinder => {
            inter.perform_cylinder_torus(other.cylinder()?, torus, tol_tang)?
        }
        rustycad_tkmath::geom_abs::SurfaceType::Cone => {
            inter.perform_cone_torus(other.cone()?, torus, tol_tang)?
        }
        _ => inter.perform_sphere_torus(other.sphere()?, torus, tol_tang)?,
    }
    treat_result_torus(quads, &inter, empty, slin)
}

/// `IntToTo(theQuad1, theQuad2, theTolTang, bSameSurf, bEmpty, theSeqLin)`: two tori,
/// the same surface or the circles of `TreatResultTorus`.
pub(super) fn int_toto(
    quads: Quads<'_>,
    tol_tang: f64,
    same_surf: &mut bool,
    empty: &mut bool,
    slin: &mut Lines,
) -> Result<bool, Failure> {
    let mut inter = QuadQuadGeo::new();
    inter.perform_tori(quads.0.torus()?, quads.1.torus()?, tol_tang)?;
    if !inter.is_done() {
        return Ok(false);
    }
    if inter.type_inter()? == ResultType::Same {
        *empty = false;
        *same_surf = true;
        return Ok(true);
    }
    treat_result_torus(quads, &inter, empty, slin)
}

/// `TreatResultTorus(theQuad1, theQuad2, anInt, bEmpty, theSeqLin)`: the circles of the
/// intersection (on the seam of the first torus when both quadrics are tori), with
/// their transitions; any other result is a failure.
fn treat_result_torus(
    quads: Quads<'_>,
    inter: &QuadQuadGeo,
    empty: &mut bool,
    slin: &mut Lines,
) -> Result<bool, Failure> {
    if !inter.is_done() {
        return Ok(false);
    }
    let typint: ResultType = inter.type_inter()?;
    let nb_sol: i32 = inter.nb_solutions()?;
    *empty = false;
    match typint {
        ResultType::Empty => *empty = true,
        ResultType::Circle => {
            for i in 1..=nb_sol {
                let mut c: Circle3 = inter.circle(i)?;
                if quads.0.type_quadric() == quads.1.type_quadric() {
                    adjust_to_seam(&quads.0.torus()?.position(), &mut c)?;
                }
                let (ptref, tgt) = elclib::d1(0.0, &c);
                let qwe: f64 = dot_cross(tgt, quads.0, quads.1, ptref)?;
                push(
                    slin,
                    IntPatchGLine::from_circle(&c, false, by_sign_tol(qwe, 0.00000001)),
                );
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
