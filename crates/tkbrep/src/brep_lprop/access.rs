// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of the DirectAccess use of ModelingData/TKBRep/BRepLProp/BRepLProp_CLProps.hxx
// and BRepLProp_SLProps.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! How the GeomLProp templates evaluate the BRepAdaptor curve and surface: through
//! their own members (`LProp_CurveUtils::DirectAccess`, `LProp_SurfaceUtils::DirectAccess`).
use crate::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomSurfaceD1, GeomSurfaceD2};
use rustycad_tkgeombase::geom_lprop::surface_access::{
    DirectSurface, PropsBounds, get_surf_bounds_adaptor,
};
use rustycad_tkgeombase::lprop::curve_access::{CurveD1, CurveD2, CurveD3, DirectCurve};
use rustycad_tkmath::gp::{Point3, Vector3};

impl DirectCurve for &BRepAdaptorCurve {
    type Vector = Vector3;
    fn props_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.d0(u)
    }
    fn props_d1(&self, u: f64) -> Result<CurveD1<Vector3>, Failure> {
        self.d1(u)
    }
    fn props_d2(&self, u: f64) -> Result<CurveD2<Vector3>, Failure> {
        self.d2(u)
    }
    fn props_d3(&self, u: f64) -> Result<CurveD3<Vector3>, Failure> {
        self.d3(u)
    }
    fn props_first_parameter(&self) -> Result<f64, Failure> {
        self.first_parameter()
    }
    fn props_last_parameter(&self) -> Result<f64, Failure> {
        self.last_parameter()
    }
}

impl DirectSurface for &BRepAdaptorSurface {
    fn props_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.d0(u, v)
    }
    fn props_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.d1(u, v)
    }
    fn props_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.d2(u, v)
    }
    fn props_bounds(&self) -> Result<PropsBounds, Failure> {
        get_surf_bounds_adaptor(*self)
    }
}
