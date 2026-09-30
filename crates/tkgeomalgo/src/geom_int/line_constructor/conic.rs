// Created on: 1995-02-07
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
// Rust translation of the conic branch of GeomInt_LineConstructor::Perform, TreatCircle,
// the class GeomInt_Vertex and RejectDuplicates
// (ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_LineConstructor.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform` on the conic lines: the parts whose middle point is inside both domains;
//! the circles and ellipses are split at their vertices sorted over one turn.
use super::GeomIntLineConstructor;
use super::helpers::{gline_point, reject_micro_circle};
use crate::geom_int::line_tool::{GeomIntLineTool, line_as};
use crate::int_patch::{IntPatchGLine, IntPatchIType, IntPatchLine};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl::sort_by;
use rustycad_tkmath::elclib;
use std::f64::consts::PI;

/// `TwoPI`.
const TWO_PI: f64 = PI + PI;

/// `RejectDuplicates(theVtxArr)`: in the sorted parameters (the last one is the first
/// plus 2 Pi), a parameter within `1000 * PConfusion` after a kept one, or before the
/// last one, is set to `RealLast` (rejected). See `IntPatch_GLine::ComputeVertexParameters`
/// for the value of the tolerance.
fn reject_duplicates(prms: &mut [f64]) {
    let tol_pc: f64 = 1000.0 * precision::p_confusion();
    let n: usize = prms.len();
    for i in 0..n.saturating_sub(2) {
        let prmi: f64 = prms[i];
        if prmi == real_last() {
            continue;
        }
        for prmj in &mut prms[i + 1..n - 1] {
            if *prmj - prmi < tol_pc {
                *prmj = real_last();
            } else {
                break;
            }
        }
    }
    // The duplicates of the last element (the first plus 2 Pi).
    let max_prm: f64 = prms[n - 1];
    for i in (1..n - 1).rev() {
        let prmi: f64 = prms[i];
        if prmi == real_last() {
            continue;
        }
        if max_prm - prmi < tol_pc {
            prms[i] = real_last();
        } else {
            break;
        }
    }
}

impl GeomIntLineConstructor {
    /// `Perform` on a conic line: circles and ellipses through `TreatCircle`; the other
    /// conics part by part, or whole when no part is longer than `PConfusion`.
    pub(super) fn perform_gline(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        tol: f64,
    ) -> Result<(), Failure> {
        self.seqp.clear();
        let typl: IntPatchIType = l.arc_type();
        let gline: Handle<IntPatchGLine> =
            line_as::<IntPatchGLine>(l, "GeomInt_LineConstructor::Perform")?;
        if typl == IntPatchIType::Circle || typl == IntPatchIType::Ellipse {
            self.treat_circle(l, tol)?;
            self.done = true;
            return Ok(());
        }
        let mut intrvtested: bool = false;
        let nbvtx: i32 = GeomIntLineTool::nb_vertex(l)?;
        for i in 1..nbvtx {
            let firstp: f64 = GeomIntLineTool::vertex(l, i)?.parameter_on_line();
            let lastp: f64 = GeomIntLineTool::vertex(l, i + 1)?.parameter_on_line();
            if (firstp - lastp).abs() <= precision::p_confusion() {
                continue;
            }
            intrvtested = true;
            let pmid: f64 = (firstp + lastp) * 0.5;
            let uv = self.adjusted_parameters(gline_point(typl, &gline, pmid)?)?;
            if self.in_both(uv, tol)? {
                self.push_part(firstp, lastp);
            }
        }
        if !intrvtested {
            // Kept a priori: a 2d point on each surface is needed to decide, which the
            // caller does.
            self.push_part(
                GeomIntLineTool::first_parameter(l)?,
                GeomIntLineTool::last_parameter(l)?,
            );
        }
        self.done = true;
        Ok(())
    }

    /// `TreatCircle(aLine, aTol)`: the vertex parameters brought into `[0, 2 Pi)` and
    /// sorted, closed by the first plus 2 Pi, without duplicates; each arc between
    /// consecutive ones whose middle is inside both domains is a part. A circle (or an
    /// ellipse of major radius) smaller than the tolerance gives no part.
    pub fn treat_circle(&mut self, l: &Handle<dyn IntPatchLine>, tol: f64) -> Result<(), Failure> {
        let typ: IntPatchIType = l.arc_type();
        let gline: Handle<IntPatchGLine> =
            line_as::<IntPatchGLine>(l, "GeomInt_LineConstructor::TreatCircle")?;
        if reject_micro_circle(&gline, typ, tol)? {
            return Ok(());
        }
        let nb_vtx: i32 = gline.nb_vertex();
        // The native array has one more (default) vertex, of parameter 0.
        let mut prms: Vec<f64> = vec![0.0; nb_vtx as usize + 1];
        for i in 1..=nb_vtx {
            prms[(i - 1) as usize] =
                elclib::in_period(gline.vertex(i)?.parameter_on_line(), 0.0, TWO_PI);
        }
        sort_by(&mut prms[..nb_vtx as usize], |a, b| a < b);
        // The last vertex.
        let n: usize = prms.len();
        prms[n - 1] = prms[0] + TWO_PI;
        reject_duplicates(&mut prms);
        sort_by(&mut prms, |a, b| a < b);
        for i in 0..n - 1 {
            let (t1, t2) = (prms[i], prms[i + 1]);
            if t2 == real_last() {
                break;
            }
            let tmid: f64 = (t1 + t2) * 0.5;
            let uv = self.adjusted_parameters(gline_point(typ, &gline, tmid)?)?;
            if self.in_both(uv, tol)? {
                self.push_part(t1, t2);
            }
        }
        Ok(())
    }
}
