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
// Rust translation of the analytic and walking branches of GeomInt_LineConstructor::Perform
// (ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_LineConstructor.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform` on the analytic and the walking lines: each part between two vertices
//! whose middle (or whose ends) is inside both domains.
use super::GeomIntLineConstructor;
use super::helpers::{adjust_periodic, parameters};
use crate::geom_int::line_tool::{GeomIntLineTool, line_as};
use crate::int_patch::{
    IntPatchALine, IntPatchLine, IntPatchPointLine, IntPatchWLType, IntPatchWLine,
};
use crate::intf::native_int_cast;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::SurfaceType;

impl GeomIntLineConstructor {
    /// The quadric parameters of `p` on both surfaces, adjusted to their periods.
    pub(super) fn adjusted_parameters(
        &self,
        p: rustycad_tkmath::gp::Point3,
    ) -> Result<[f64; 4], Failure> {
        let mut uv: [f64; 4] = parameters(&self.my_hs1, &self.my_hs2, p)?;
        adjust_periodic(&self.my_hs1, &self.my_hs2, &mut uv)?;
        Ok(uv)
    }

    /// The parameters of a point line, adjusted to the periods of the quadrics.
    fn adjusted(&self, (u1, v1, u2, v2): (f64, f64, f64, f64)) -> Result<[f64; 4], Failure> {
        let mut uv: [f64; 4] = [u1, v1, u2, v2];
        adjust_periodic(&self.my_hs1, &self.my_hs2, &mut uv)?;
        Ok(uv)
    }

    /// `Perform` on an analytic line: the parts whose middle point is inside.
    pub(super) fn perform_aline(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        tol: f64,
    ) -> Result<(), Failure> {
        let aline: Handle<IntPatchALine> =
            line_as::<IntPatchALine>(l, "GeomInt_LineConstructor::Perform")?;
        self.seqp.clear();
        let nbvtx: i32 = GeomIntLineTool::nb_vertex(l)?;
        for i in 1..nbvtx {
            let firstp: f64 = GeomIntLineTool::vertex(l, i)?.parameter_on_line();
            let lastp: f64 = GeomIntLineTool::vertex(l, i + 1)?.parameter_on_line();
            if firstp == lastp {
                continue;
            }
            let pmid: f64 = (firstp + lastp) * 0.5;
            let uv: [f64; 4] = self.adjusted_parameters(aline.value(pmid)?)?;
            if self.in_both(uv, tol)? {
                self.push_part(firstp, lastp);
            }
        }
        self.done = true;
        Ok(())
    }

    /// `Perform` on a walking line: a part of more than one step is tested at its
    /// middle point, a part of one step at its ends (at the middle of its ends for the
    /// implicit-parametric lines, fix #29972); the parts are then merged on a plane
    /// with an extrusion or a revolution.
    pub(super) fn perform_wline(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        tol: f64,
    ) -> Result<(), Failure> {
        let wline: Handle<IntPatchWLine> =
            line_as::<IntPatchWLine>(l, "GeomInt_LineConstructor::Perform")?;
        self.seqp.clear();
        let nbvtx: i32 = GeomIntLineTool::nb_vertex(l)?;
        for i in 1..nbvtx {
            let firstp: f64 = GeomIntLineTool::vertex(l, i)?.parameter_on_line();
            let lastp: f64 = GeomIntLineTool::vertex(l, i + 1)?.parameter_on_line();
            if firstp == lastp {
                continue;
            }
            if self.wline_part_inside(&wline, (firstp, lastp), tol)? {
                self.push_part(firstp, lastp);
            }
        }
        self.merge_on_plane()?;
        self.done = true;
        Ok(())
    }

    /// Whether the part `firstp..lastp` of the walking line is inside both domains.
    fn wline_part_inside(
        &self,
        wline: &IntPatchWLine,
        (firstp, lastp): (f64, f64),
        tol: f64,
    ) -> Result<bool, Failure> {
        if lastp != firstp + 1.0 {
            let pmid: i32 = native_int_cast((firstp + lastp) / 2.0);
            let uv = self.adjusted(wline.point(pmid)?.parameters())?;
            return self.in_both(uv, tol);
        }
        let first = self.adjusted(wline.point(native_int_cast(firstp))?.parameters())?;
        if wline.get_creating_way() == IntPatchWLType::ImpPrm {
            // Fix #29972: the implicit-parametric intersector does not respect the
            // domain of the quadric, so a line of 2 points can have one outside it;
            // the middle point (interpolated) is classified.
            let last = self.adjusted(wline.point(native_int_cast(lastp))?.parameters())?;
            let mid: [f64; 4] = [
                0.5 * (first[0] + last[0]),
                0.5 * (first[1] + last[1]),
                0.5 * (first[2] + last[2]),
                0.5 * (first[3] + last[3]),
            ];
            return self.in_both(mid, tol);
        }
        // The other intersectors respect both domains: both points are checked.
        if !self.in_both(first, tol)? {
            return Ok(false);
        }
        let last = self.adjusted(wline.point(native_int_cast(lastp))?.parameters())?;
        self.in_both(last, tol)
    }

    /// The 7 connected segments of one curve become one segment on a plane with an
    /// extrusion or a revolution (PKV 22.Apr.2002): the parameters met twice (as
    /// integers) are removed.
    fn merge_on_plane(&mut self) -> Result<(), Failure> {
        if self.seqp.len() / 2 <= 1 {
            return Ok(());
        }
        let swept = |t: SurfaceType| {
            matches!(
                t,
                SurfaceType::SurfaceOfExtrusion | SurfaceType::SurfaceOfRevolution
            )
        };
        let (st1, st2) = (self.my_hs1.get_type()?, self.my_hs2.get_type()?);
        let cond: bool = if st1 == SurfaceType::Plane {
            swept(st2)
        } else {
            st2 == SurfaceType::Plane && swept(st1)
        };
        if !cond {
            return Ok(());
        }
        let mut map: Vec<i32> = Vec::new();
        let mut seq_tmp: Vec<f64> = Vec::new();
        for &lastp in &self.seqp {
            let index: i32 = native_int_cast(lastp);
            if !map.contains(&index) {
                map.push(index);
                seq_tmp.push(lastp);
            } else if seq_tmp.pop().is_none() {
                // `aSeqTmp.Remove(0)` on an empty sequence.
                return Err(rustycad_tkernel::standard::Failure::with_message(
                    "Standard_OutOfRange",
                    &format!(
                        "NCollection_Sequence::Remove: index 0 of an empty sequence (parameter {lastp})"
                    ),
                ));
            }
        }
        self.seqp.clear();
        for [first, last] in seq_tmp.as_chunks::<2>().0 {
            self.push_part(*first, *last);
        }
        Ok(())
    }
}
