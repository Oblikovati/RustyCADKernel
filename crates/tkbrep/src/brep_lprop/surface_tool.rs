// Created on: 1994-02-24
// Created by: Laurent BOURESCHE
// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRepLProp/BRepLProp_SurfaceTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLProp_SurfaceTool`: the static evaluation interface of a BRepAdaptor surface.
use crate::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomSurfaceD1, GeomSurfaceD2};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// `BRepLProp_SurfaceTool`: static members over a [`BRepAdaptorSurface`]; the output
/// parameters of `D1` and `D2` are returned as the result records.
///
/// ```
/// use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
/// use rustycad_tkbrep::brep_lprop::BRepLPropSurfaceTool;
/// let surface = BRepAdaptorSurface::default();
/// assert!(BRepLPropSurfaceTool::value(&surface, 0.0, 0.0).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepLPropSurfaceTool;

impl BRepLPropSurfaceTool {
    /// `Value(S, U, V, P)`; `BRepLPropSurfaceTool::value(&surface, 0.5, 0.5)?`.
    pub fn value(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<Point3, Failure> {
        surface.value(u, v)
    }

    /// `D1(S, U, V, P, D1U, D1V)`; `BRepLPropSurfaceTool::d1(&surface, 0.5, 0.5)?`.
    pub fn d1(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        surface.d1(u, v)
    }

    /// `D2(S, U, V, P, D1U, D1V, D2U, D2V, DUV)`; `BRepLPropSurfaceTool::d2(&surface, 0.5, 0.5)?`.
    pub fn d2(surface: &BRepAdaptorSurface, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        surface.d2(u, v)
    }

    /// `DN(S, U, V, IU, IV)`; `BRepLPropSurfaceTool::dn(&surface, 0.5, 0.5, 1, 1)?`.
    pub fn dn(
        surface: &BRepAdaptorSurface,
        u: f64,
        v: f64,
        iu: i32,
        iv: i32,
    ) -> Result<Vector3, Failure> {
        surface.dn(u, v, iu, iv)
    }

    /// `Continuity(S)`: the order (0 to 3) of the lower of the U and V continuities, G1
    /// and G2 counting as 0 and CN as 3; `BRepLPropSurfaceTool::continuity(&surface)?`.
    pub fn continuity(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        let shape = surface.u_continuity()?.min(surface.v_continuity()?);
        Ok(match shape {
            Shape::C0 | Shape::G1 | Shape::G2 => 0,
            Shape::C1 => 1,
            Shape::C2 => 2,
            Shape::C3 | Shape::CN => 3,
        })
    }

    /// `Bounds(S, U1, V1, U2, V2)`: `(U1, V1, U2, V2)`;
    /// `let (u1, v1, u2, v2) = BRepLPropSurfaceTool::bounds(&surface)?;`.
    pub fn bounds(surface: &BRepAdaptorSurface) -> Result<(f64, f64, f64, f64), Failure> {
        Ok((
            surface.first_u_parameter()?,
            surface.first_v_parameter()?,
            surface.last_u_parameter()?,
            surface.last_v_parameter()?,
        ))
    }
}
