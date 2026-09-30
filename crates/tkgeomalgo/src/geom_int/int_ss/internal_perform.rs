// Created on: 1995-01-27
// Created by: Jacques GOUSSARD
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
// Rust translation of GeomInt_IntSS::InternalPerform and the dispatch of
// GeomInt_IntSS::MakeCurve (ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_IntSS.cxx and
// GeomInt_IntSS_1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `InternalPerform`: `IntPatch_Intersection` on each pair of the prepared surfaces,
//! and the curves of its lines (`MakeCurve`).
use super::{GeomIntIntSS, GeomIntIntSSApprox};
use crate::geom_int::line_tool::line_as;
use crate::int_patch::{IntPatchIType, IntPatchIntersection, IntPatchLine, IntPatchWLine};
use crate::int_start::TheTopolTool;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point2;
use std::rc::Rc;

/// The domain (`Adaptor3d_TopolTool`) of an adaptor.
fn domain(s: &Adaptor3dSurfaceHandle) -> Result<Rc<dyn TheTopolTool>, Failure> {
    Ok(Rc::new(Adaptor3dTopolTool::with_surface(s)?))
}

/// The inputs of `MakeCurve`: the domains and the tolerance of the pair.
pub(super) struct CurveInputs<'a> {
    pub(super) dom1: &'a dyn TheTopolTool,
    pub(super) dom2: &'a dyn TheTopolTool,
    pub(super) tol: f64,
    pub(super) approx: GeomIntIntSSApprox,
}

impl GeomIntIntSS {
    /// `InternalPerform(Tol, Approx, ApproxS1, ApproxS2, useStart, U1, V1, U2, V2)`: the
    /// results are reset (the 2d curve sequences are not, as natively), each pair of
    /// the prepared surfaces (a flat cone is split at its apex) is intersected and
    /// its lines become curves.
    pub(super) fn internal_perform(
        &mut self,
        tol: f64,
        approx: GeomIntIntSSApprox,
        start: Option<[f64; 4]>,
    ) -> Result<(), Failure> {
        self.my_tol_reached2d = 0.0;
        self.my_tol_reached3d = 0.0;
        self.my_nbrestr = 0;
        self.sline.clear();
        let (tol_arc, tol_tang) = (tol, tol);
        let mut deflection: f64 = 0.1;
        if self.my_hs1.get_type()? == SurfaceType::BSplineSurface
            && self.my_hs2.get_type()? == SurfaceType::BSplineSurface
        {
            deflection /= 10.0;
        }
        let hs1: Adaptor3dSurfaceHandle = self.my_hs1.cast();
        let hs2: Adaptor3dSurfaceHandle = self.my_hs2.cast();
        let dom1 = domain(&hs1)?;
        let dom2 = domain(&hs2)?;
        let (mut vec_hs1, mut vec_hs2): (Vec<Adaptor3dSurfaceHandle>, Vec<Adaptor3dSurfaceHandle>) =
            (Vec::new(), Vec::new());
        if self.my_hs1 == self.my_hs2 {
            vec_hs1.push(hs1.clone());
            vec_hs2.push(hs2.clone());
        } else {
            IntPatchIntersection::prepare_surfaces(
                &hs1,
                &*dom1,
                &hs2,
                &*dom2,
                tol,
                &mut vec_hs1,
                &mut vec_hs2,
            )?;
        }
        for a_hs1 in &vec_hs1 {
            for a_hs2 in &vec_hs2 {
                self.perform_pair(
                    (a_hs1, a_hs2),
                    (tol_arc, tol_tang, deflection),
                    (tol, approx),
                    start,
                )?;
            }
        }
        Ok(())
    }

    /// The intersection of one pair of prepared surfaces and the curves of its lines.
    fn perform_pair(
        &mut self,
        (a_hs1, a_hs2): (&Adaptor3dSurfaceHandle, &Adaptor3dSurfaceHandle),
        (tol_arc, tol_tang, deflection): (f64, f64, f64),
        (tol, approx): (f64, GeomIntIntSSApprox),
        start: Option<[f64; 4]>,
    ) -> Result<(), Failure> {
        let a_dom1 = domain(a_hs1)?;
        let a_dom2 = domain(a_hs2)?;
        let g1: Handle<GeomAdaptorSurface> = a_hs1.cast();
        let g2: Handle<GeomAdaptorSurface> = a_hs2.cast();
        self.my_lconstruct
            .load(a_dom1.clone(), a_dom2.clone(), &g1, &g2);
        let uv_max_step: f64 =
            IntPatchIntersection::define_uv_max_step(a_hs1, &*a_dom1, a_hs2, &*a_dom2)?;
        self.my_intersector
            .set_tolerances(tol_arc, tol_tang, uv_max_step, deflection);
        let tols: (f64, f64) = (tol_arc, tol_tang);
        if a_hs1 == a_hs2 {
            self.my_intersector
                .perform_self(a_hs1, &*a_dom1, tol_arc, tol_tang)?;
        } else if let Some(start) = start {
            let state1: State = a_dom1.classify(Point2::new(start[0], start[1]), tol, true)?;
            let state2: State = a_dom2.classify(Point2::new(start[2], start[3]), tol, true)?;
            let inside = |s: State| s == State::In || s == State::On;
            if inside(state1) && inside(state2) {
                self.my_intersector.perform_from_start(
                    (a_hs1, &*a_dom1),
                    (a_hs2, &*a_dom2),
                    start,
                    tols,
                )?;
            }
        } else {
            let options = crate::int_patch::IntPatchIntersectionOptions::default();
            self.my_intersector
                .perform(a_hs1, &*a_dom1, a_hs2, &*a_dom2, tols, options)?;
        }
        if !self.my_intersector.is_done() {
            return Ok(());
        }
        let inputs = CurveInputs {
            dom1: &*a_dom1,
            dom2: &*a_dom2,
            tol,
            approx,
        };
        for i in 1..=self.my_intersector.nb_lines()? {
            self.make_curve(i, &inputs)?;
        }
        Ok(())
    }

    /// `MakeCurve(Index, D1, D2, Tol, Approx, Approx1, Approx2)`: the line `index` of
    /// the intersector split by the line constructor, and the curves of its parts by
    /// line kind (the analytic lines were processed in `IntPatch_Intersection`).
    pub(super) fn make_curve(
        &mut self,
        index: i32,
        inputs: &CurveInputs<'_>,
    ) -> Result<(), Failure> {
        let mut l: Handle<dyn IntPatchLine> = self.my_intersector.line(index)?;
        let typl: IntPatchIType = l.arc_type();
        if typl == IntPatchIType::Walking {
            let wline: Handle<IntPatchWLine> = l.cast();
            if wline.is_null() {
                return Ok(());
            }
            l = wline.cast();
        }
        self.my_lconstruct.perform(&l)?;
        if !self.my_lconstruct.is_done() || self.my_lconstruct.nb_parts()? <= 0 {
            return Ok(());
        }
        let mut tolpc: f64 = inputs.tol;
        match typl {
            IntPatchIType::Lin | IntPatchIType::Parabola | IntPatchIType::Hyperbola => {
                self.open_conic_curves(&l, inputs, &mut tolpc)
            }
            IntPatchIType::Circle | IntPatchIType::Ellipse => {
                self.closed_conic_curves(&l, inputs, &mut tolpc)
            }
            IntPatchIType::Analytic => Ok(()),
            IntPatchIType::Walking => {
                let wline: Handle<IntPatchWLine> =
                    line_as::<IntPatchWLine>(&l, "GeomInt_IntSS::MakeCurve")?;
                self.walking_curves(wline, inputs.approx)
            }
            IntPatchIType::Restriction => self.restriction_curves(&l, inputs.approx),
        }
    }
}
