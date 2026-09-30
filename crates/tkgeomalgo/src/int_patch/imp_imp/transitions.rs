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
// Rust translation of the transition tests repeated through
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The transitions of a line from the sign of the mixed product of its tangent and the
//! two normals, and the transitions of a point on a restriction.
use super::super::line::IntPatchLineTransitions;
use crate::int_surf::{IntSurfQuadric, IntSurfTransition, IntSurfTypeTrans, make_transition};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

/// `if (x > 0.) {Out, In} else {In, Out}`: the transitions of a crossing line.
pub(super) fn by_sign(x: f64) -> IntPatchLineTransitions {
    if x > 0.0 {
        return IntPatchLineTransitions::Types(IntSurfTypeTrans::Out, IntSurfTypeTrans::In);
    }
    IntPatchLineTransitions::Types(IntSurfTypeTrans::In, IntSurfTypeTrans::Out)
}

/// `if (x > tol) {Out, In} else if (x < -tol) {In, Out} else {Undecided, Undecided}`.
pub(super) fn by_sign_tol(x: f64, tol: f64) -> IntPatchLineTransitions {
    let (t1, t2) = if x > tol {
        (IntSurfTypeTrans::Out, IntSurfTypeTrans::In)
    } else if x < -tol {
        (IntSurfTypeTrans::In, IntSurfTypeTrans::Out)
    } else {
        (IntSurfTypeTrans::Undecided, IntSurfTypeTrans::Undecided)
    };
    IntPatchLineTransitions::Types(t1, t2)
}

/// `Tgt.DotCross(Quad2.Normale(P), Quad1.Normale(P))`: the mixed product of a tangent at
/// `p` and the two quadric normals there.
pub(super) fn dot_cross(
    tgt: Vector3,
    quad1: &IntSurfQuadric,
    quad2: &IntSurfQuadric,
    p: Point3,
) -> Result<f64, Failure> {
    Ok(tgt.dot_cross(quad2.normale_at(p)?, quad1.normale_at(p)?))
}

/// The transitions of the line and of the arc at a point on a restriction: undecided
/// on a null normal (the apex of a cone), `IntSurf::MakeTransition` otherwise.
pub(super) fn line_arc_transitions(
    vtgint: Vector3,
    vtgrst: Vector3,
    normale: Vector3,
    transline: &mut IntSurfTransition,
    transarc: &mut IntSurfTransition,
) -> Result<(), Failure> {
    if normale.square_magnitude() < 1e-16 {
        transline.set_type(true, IntSurfTypeTrans::Undecided);
        transarc.set_type(true, IntSurfTypeTrans::Undecided);
        return Ok(());
    }
    make_transition(vtgint, vtgrst, Direction3::from_vector(normale)?, transline, transarc);
    Ok(())
}
