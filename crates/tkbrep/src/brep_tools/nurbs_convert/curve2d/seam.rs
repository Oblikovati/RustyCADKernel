// Created on: 1996-07-12
// Created by: Stagiaire Mary FABIEN
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of the seam branch of BRepTools_NurbsConvertModification::NewCurve2d
// (ModelingData/TKBRep/BRepTools/BRepTools_NurbsConvertModification.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two p-curves of a seam: the first call projects both and keeps the second one
//! for the second call.
use super::{
    Curve2dContext, RangedCurve, adaptor_2d, adaptor_3d, copy_2d, is_spline_2d, to_bspline_2d,
    trimmed_2d, widened_adaptor,
};
use crate::brep::BRepTool;
use crate::brep_tools::nurbs_convert::NurbsConvertModification;
use crate::topods;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::proj_lib::ComputeApproxOnPolarSurface;

impl NurbsConvertModification {
    /// A seam of the face. The first time the edge is met, both p-curves (trimmed to
    /// their ranges) are projected together on the new face's surface and the second
    /// result is stored in `mylcu`; the second time, the stored curve at the rank of the
    /// edge in `myled` is converted (false when it is not a spline). The ranks of
    /// `myled` and `mylcu` are not aligned when a first pass returned early, as
    /// natively.
    pub(in crate::brep_tools::nurbs_convert) fn project_seam(
        &self,
        context: &Curve2dContext<'_>,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let rank = {
            let state = self.state.read();
            state.led.iter().position(|edge| edge.is_same(context.edge))
        };
        if let Some(rank) = rank {
            return self.second_pass(context, rank, curve);
        }
        self.first_pass(context, curve, tolerance)
    }

    /// `C2d = itlcu.Value()` at the rank of the edge, converted when it is a spline.
    fn second_pass(
        &self,
        context: &Curve2dContext<'_>,
        rank: usize,
        curve: &mut Handle<dyn Geom2dCurve>,
    ) -> Result<bool, Failure> {
        let stored = self.state.read().lcu.get(rank).cloned().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                &format!("BRepTools_NurbsConvertModification::NewCurve2d: no stored p-curve at rank {rank}; expected one per processed seam"),
                "",
            )
        })?;
        let c2d = stored.cast::<dyn Geom2dCurve>();
        if !is_spline_2d(&c2d) {
            return Ok(false);
        }
        *curve = to_bspline_2d(&c2d)?;
        self.record(context.base.cast(), curve.cast());
        Ok(true)
    }

    /// The first pass over a seam.
    fn first_pass(
        &self,
        context: &Curve2dContext<'_>,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let c2d = trimmed_2d(&context.base, context.range)?;
        let reversed = topods::edge(&context.edge.reversed())?;
        let bis = BRepTool::curve_on_face(&reversed, context.face)?;
        let c2d_bis = trimmed_2d(&bis.pcurve, (bis.first, bis.last))?;
        if context.raw_3d.curve.is_null() {
            return self.convert_without_3d(context, &c2d, curve, tolerance);
        }
        let mut c3d = RangedCurve {
            curve: context.raw_3d.curve.clone(),
            range: context.raw_3d.range,
        };
        if !context.new_edge.is_null() {
            let (new_curve, f, l) = BRepTool::curve(context.new_edge)?;
            c3d = RangedCurve {
                curve: new_curve,
                range: (f, l),
            };
        }
        let curve_adaptor = adaptor_3d(&c3d)?;
        let mut surface = BRepTool::surface(context.face)?;
        if context.new_face.is_null() {
            self.state.write().lcu.push(c2d_bis.cast());
            if is_spline_2d(&c2d) {
                // Never met natively: `C2d` is a Geom2d_TrimmedCurve here.
                return self.copy_if_convert(context, (&c2d, &c3d, &surface), curve, tolerance);
            }
        } else {
            surface = BRepTool::surface(context.new_face)?;
        }
        let adapted = widened_adaptor(&surface)?;
        self.state
            .write()
            .led
            .push(context.edge.clone().into_shape());
        let projection = ComputeApproxOnPolarSurface::with_seam_curves(
            &adaptor_2d(&c2d, context.range)?,
            &adaptor_2d(&c2d_bis, (bis.first, bis.last))?,
            &curve_adaptor,
            &adapted,
            *tolerance,
        )?;
        if projection.is_done() {
            *curve = projection.bspline().cast();
            self.state.write().lcu.push(projection.curve2d()?.cast());
            return self.accept(context, (&c3d, &surface), curve, tolerance);
        }
        *curve = to_bspline_2d(&c2d)?;
        let accepted = self.accept(context, (&c3d, &surface), curve, tolerance)?;
        self.state
            .write()
            .lcu
            .push(c2d_bis.cast::<StandardTransient>());
        Ok(accepted)
    }

    /// Without a 3D curve: a copy of the p-curve when a conversion is needed (its
    /// tolerance evaluation dereferences the null 3D curve natively; Standard_NullObject
    /// here), else false.
    fn convert_without_3d(
        &self,
        context: &Curve2dContext<'_>,
        c2d: &Handle<dyn Geom2dCurve>,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        if !context.is_convert_2d {
            return Ok(false);
        }
        *curve = copy_2d(c2d)?;
        let surface = if context.new_face.is_null() {
            BRepTool::surface(context.face)?
        } else {
            BRepTool::surface(context.new_face)?
        };
        let c3d = RangedCurve {
            curve: context.raw_3d.curve.clone(),
            range: context.raw_3d.range,
        };
        self.accept(context, (&c3d, &surface), curve, tolerance)
    }

    /// The copy of a spline p-curve on an unchanged face when a conversion is needed.
    fn copy_if_convert(
        &self,
        context: &Curve2dContext<'_>,
        (c2d, c3d, surface): (
            &Handle<dyn Geom2dCurve>,
            &RangedCurve,
            &Handle<dyn GeomSurface>,
        ),
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        if !context.is_convert_2d {
            return Ok(false);
        }
        *curve = copy_2d(c2d)?;
        self.accept(context, (c3d, surface), curve, tolerance)
    }
}
