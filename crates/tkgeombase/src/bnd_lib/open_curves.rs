// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of the line, parabola and hyperbola members of BndLib.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BndLib::Add` for the open curves: lines, parabolas and hyperbolas, possibly with
//! infinite parameters. Unlike GeomBndLib they add to the box they are given.
use crate::geom_bnd_lib::ranges::{add_branch_range, add_line_range};
use crate::geom_bnd_lib::{HyperbolaArc, add_parabola_range};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Hyperbola2, Hyperbola3, Line2, Line3, Parabola2, Parabola3};

// The native message of every BndLib `bad parameter` raise.
const BAD_PARAMETER: &str = "BndLib::bad parameter";

/// `BndLib::Add(gp_Lin, P1, P2, Tol, B)`: the segment ends added to `B`; an infinite end
/// opens the sides the direction reaches; two infinite ends of the same sign raise
/// Standard_Failure; `add_lin(&line, 0.0, 10.0, 0.0, &mut bounds)?`.
pub fn add_lin(
    line: &Line3,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let value = |u: f64| elclib::value(u, line);
    add_line_range(bounds, line.direction(), (p1, p2), &value, BAD_PARAMETER)?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Lin2d, P1, P2, Tol, B)`: as [`add_lin`] in 2D;
/// `add_lin2d(&line, 0.0, 10.0, 0.0, &mut bounds)?`.
pub fn add_lin2d(
    line: &Line2,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox2,
) -> Result<(), Failure> {
    let value = |u: f64| elclib::value(u, line);
    add_line_range(bounds, line.direction(), (p1, p2), &value, BAD_PARAMETER)?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Parab, P1, P2, Tol, B)`: P1, P2 and the vertex when the arc crosses 0;
/// an infinite end opens every min or max side; two infinite ends of the same sign raise
/// Standard_Failure; `add_parab(&parabola, -4.0, 4.0, 0.0, &mut bounds)?`.
pub fn add_parab(
    parabola: &Parabola3,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    add_parabola_range(bounds, parabola, (p1, p2), BAD_PARAMETER)?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Parab2d, P1, P2, Tol, B)`: as [`add_parab`] in 2D;
/// `add_parab2d(&parabola, -4.0, 4.0, 0.0, &mut bounds)?`.
pub fn add_parab2d(
    parabola: &Parabola2,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox2,
) -> Result<(), Failure> {
    add_parabola_range(bounds, parabola, (p1, p2), BAD_PARAMETER)?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Hypr, P1, P2, Tol, B)`: a finite arc adds P1 and runs `ComputeBox`
/// (both ends, the vertex, the coordinate extrema in the arc); infinite ends as for the
/// parabola; `add_hypr(&hyperbola, -1.0, 1.0, 0.0, &mut bounds)?`.
pub fn add_hypr(
    hyperbola: &Hyperbola3,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let value = |u: f64| elclib::value(u, hyperbola);
    let position = hyperbola.position();
    let arc = HyperbolaArc {
        major: hyperbola.major_radius(),
        minor: hyperbola.minor_radius(),
        x_dir: *position.x_direction().coordinates(),
        y_dir: *position.y_direction().coordinates(),
        eval: &value,
    };
    let mut finite = |b: &mut BoundingBox3| {
        arc.add_arc(b, p1, p2);
    };
    add_branch_range(bounds, (p1, p2), &value, &mut finite, BAD_PARAMETER)?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Hypr2d, P1, P2, Tol, B)`: the parabola rule, P1, P2 and the vertex
/// only — no coordinate extrema, unlike the 3D member and GeomBndLib_Hyperbola2d (a
/// native looseness: an arc whose extremum lies inside is under-bounded);
/// `add_hypr2d(&hyperbola, -1.0, 1.0, 0.0, &mut bounds)?`.
pub fn add_hypr2d(
    hyperbola: &Hyperbola2,
    p1: f64,
    p2: f64,
    tolerance: f64,
    bounds: &mut BoundingBox2,
) -> Result<(), Failure> {
    add_parabola_range(bounds, hyperbola, (p1, p2), BAD_PARAMETER)?;
    bounds.enlarge(tolerance);
    Ok(())
}
