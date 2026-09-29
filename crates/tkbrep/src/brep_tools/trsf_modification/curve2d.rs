// Created on: 1994-08-30
// Created by: Jacques GOUSSARD
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
// Rust translation of BRepTools_TrsfModification::NewCurve2d
// (ModelingData/TKBRep/BRepTools/BRepTools_TrsfModification.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_TrsfModification::NewCurve2d`: the p-curve on the transformed surface.
use super::{TrsfModification, local_transformation};
use crate::brep::BRepTool;
use crate::brep::errors::domain_error;
use crate::brep_tools::modification::BRepToolsModification;
use crate::topexp;
use crate::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_lib::{gtransform, same_range};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::TransformForm;

/// The p-curve range narrowed to a non-periodic basis curve: the curve bounds replace
/// the range bounds they exceed, and an empty range is widened back to one of them.
fn narrowed_range(curve: &dyn Geom2dCurve, (mut f, mut l): (f64, f64)) -> (f64, f64) {
    if curve.is_periodic() {
        return (f, l);
    }
    let (fc, lc) = (curve.first_parameter(), curve.last_parameter());
    let tolerance = precision::p_confusion();
    if fc - f > tolerance {
        f = fc;
    }
    if l - lc > tolerance {
        l = lc;
    }
    if (l - f).abs() < tolerance {
        if (f - fc).abs() < tolerance && !precision::is_infinite(lc) {
            l = lc;
        } else if !precision::is_infinite(fc) {
            f = fc;
        }
    }
    (f, l)
}

/// The basis of a `Geom2d_TrimmedCurve` (exact type), the curve otherwise.
fn untrimmed(curve: Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    let trimmed = curve.cast::<Geom2dTrimmedCurve>();
    match trimmed.get() {
        Some(trimmed) => trimmed.basis_curve(),
        None => curve,
    }
}

impl TrsfModification {
    /// `NewCurve2d(E, F, NewE, NewF, C, Tol)`: the scaled tolerance, and, off a plane,
    /// the p-curve (its basis, on a range narrowed to it) transformed by the parametric
    /// transformation of the surface for a scaling, then brought to the new vertex
    /// parameters by `GeomLib::SameRange`. False for a face without surface, a plane and
    /// an edge without p-curve.
    pub(super) fn transformed_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let scale = self.trsf().scale_factor();
        *tolerance = BRepTool::edge_tolerance(edge)? * scale.abs();
        let (surface, location) = BRepTool::surface_with_location(face)?;
        if surface.is_null() {
            return Ok(false);
        }
        if GeomAdaptorSurface::with_surface(&surface)?.get_type()? == SurfaceType::Plane {
            return Ok(false);
        }
        let found = BRepTool::curve_on_face(edge, face)?;
        if found.pcurve.is_null() {
            return Ok(false);
        }
        let mut pcurve = untrimmed(found.pcurve);
        let Some(basis) = pcurve.get() else {
            return Ok(false);
        };
        let (f, l) = narrowed_range(basis, (found.first, found.last));
        let (mut new_first, mut new_last) = (f, l);
        if scale.abs() != 1.0 {
            pcurve = Handle::new(Geom2dTrimmedCurve::new(&pcurve, f, l, true, true)?).cast();
            let local = local_transformation(self.trsf(), &location)?;
            let Some(stored) = surface.get() else {
                return Ok(false);
            };
            let gtrsf = stored.parametric_transformation(local)?;
            if gtrsf.form() != TransformForm::Identity {
                pcurve = gtransform(&pcurve, &gtrsf)?;
                let Some(transformed) = pcurve.get() else {
                    return Err(domain_error(
                        "TrsfModification:Error in NewCurve2d",
                        "GeomLib::GTransform gave no curve",
                    ));
                };
                (new_first, new_last) =
                    (transformed.first_parameter(), transformed.last_parameter());
            }
        }
        let (f, l) = self.vertex_range(edge, (f, l))?;
        same_range(
            precision::p_confusion(),
            &pcurve,
            (new_first, new_last),
            (f, l),
            curve,
        )?;
        Ok(true)
    }

    /// The new parameters of the two vertices on the FORWARD edge (a null vertex keeps
    /// the given bound).
    fn vertex_range(&self, edge: &Edge, (mut f, mut l): (f64, f64)) -> Result<(f64, f64), Failure> {
        let (v1, v2) = topexp::vertices(edge, false);
        let forward = topods::edge(&edge.oriented(Orientation::Forward))?;
        let mut vertex_tolerance = 0.0;
        self.new_parameter(&v1, &forward, &mut f, &mut vertex_tolerance)?;
        self.new_parameter(&v2, &forward, &mut l, &mut vertex_tolerance)?;
        Ok((f, l))
    }
}
