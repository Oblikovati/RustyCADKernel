// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom (TransformParameters and ParametricTransformation idioms shared by the surfaces) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter changes of a transformed surface that several Geom surfaces
//! write with the same native statements.
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::{
    Axis2d, Direction2, GeneralTransform2, Transform3, dx2d, dy2d, origin2d,
};

/// `if (!Precision::IsInfinite(P)) P *= std::abs(T.ScaleFactor())`: a length
/// parameter scaled by the similarity ratio; `scaled_parameter(2.0, t)`.
pub fn scaled_parameter(parameter: f64, transform: Transform3) -> f64 {
    if precision::is_infinite(parameter) {
        return parameter;
    }
    parameter * transform.scale_factor().abs()
}

/// `gp_GTrsf2d T; T.SetAffinity(gp_Ax2d(gp::Origin2d(), direction), ratio)`: the
/// affinity that scales the parameter normal to `direction` by `ratio`.
fn affinity(direction: Direction2, ratio: f64) -> GeneralTransform2 {
    let mut result = GeneralTransform2::default();
    result.set_affinity(Axis2d::new(origin2d(), direction), ratio);
    result
}

/// The affinity about the U axis (`gp::DX2d()`), which scales V by `ratio`;
/// `v_affinity(2.0)`.
pub fn v_affinity(ratio: f64) -> GeneralTransform2 {
    affinity(dx2d(), ratio)
}

/// The affinity about the V axis (`gp::DY2d()`), which scales U by `ratio`;
/// `u_affinity(2.0)`.
pub fn u_affinity(ratio: f64) -> GeneralTransform2 {
    affinity(dy2d(), ratio)
}
