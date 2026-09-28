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
// Rust translation of ModelingData/TKG3d/Geom (the ElSLib results as Geom_Surface::ResD2/ResD3 of the elementary surfaces) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Conversions of the ElSLib derivative bundles into the Geom surface results,
//! field for field, as the native `ElSLib::*D2/D3(..., aResult.Point, ...)` calls fill them.
use super::surface::{GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkmath::elslib::{self, SurfaceD3};

/// The native D2/D3 result from an ElSLib third-order evaluation.
pub fn surface_d3(values: SurfaceD3) -> GeomSurfaceD3 {
    let second = values.second;
    GeomSurfaceD3 {
        point: second.point,
        d1u: second.du,
        d1v: second.dv,
        d2u: second.duu,
        d2v: second.dvv,
        d2uv: second.duv,
        d3u: values.duuu,
        d3v: values.dvvv,
        d3uuv: values.duuv,
        d3uvv: values.duvv,
    }
}

/// The native D2 result from an ElSLib second-order evaluation.
pub fn surface_d2(values: elslib::SurfaceD2) -> GeomSurfaceD2 {
    GeomSurfaceD2 {
        point: values.point,
        d1u: values.du,
        d1v: values.dv,
        d2u: values.duu,
        d2v: values.dvv,
        d2uv: values.duv,
    }
}
