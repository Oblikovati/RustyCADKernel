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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineCurve_1.cxx (read access) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Read access: local evaluation, knots, multiplicities, poles and weights, through
//! the accessors shared with the other B-spline curve class.
use super::GeomBSplineCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};

impl GeomBSplineCurve {
    rustycad_tkg2d::bspline_curve_accessors!(Point3, Vector3);
}
