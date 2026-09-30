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
// Rust translation of the restriction case of GeomInt_IntSS::MakeCurve
// (ModelingAlgorithms/TKGeomAlgo/GeomInt/GeomInt_IntSS_1.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curves of a restriction line: its 3d curve (`TreatRLine`) cut where its 2d
//! curves cross the boundaries of the surfaces, the pieces inside both boxes kept.
use super::treat_rline::GeomIntRLineCurves;
use super::{GeomIntIntSS, GeomIntIntSSApprox};
use crate::geom_int::line_tool::line_as;
use crate::int_patch::{IntPatchLine, IntPatchRLine};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point2;

/// The box of the parameter range of an adaptor.
fn domain_box(hs: &GeomAdaptorSurface) -> Result<BoundingBox2, Failure> {
    let (uf, vf) = (hs.first_u_parameter()?, hs.first_v_parameter()?);
    let (ul, vl) = (hs.last_u_parameter()?, hs.last_v_parameter()?);
    let mut b = BoundingBox2::default();
    b.add_point(Point2::new(uf, vf));
    b.add_point(Point2::new(ul, vl));
    Ok(b)
}

/// The 2d curve of a piece `par_f..par_l`: None when its middle point is out of the
/// box (the piece is dropped), else the trimmed curve when wanted.
fn piece_2d(
    c2d: &Handle<dyn Geom2dCurve>,
    bx: &BoundingBox2,
    (par_f, par_l): (f64, f64),
    wanted: bool,
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let Some(c) = c2d.get() else {
        return Ok(Some(Handle::default()));
    };
    let pt: Point2 = c.d0(0.5 * (par_f + par_l))?;
    if bx.is_out_point(pt) {
        return Ok(None);
    }
    if !wanted {
        return Ok(Some(Handle::default()));
    }
    Ok(Some(
        Handle::new(Geom2dTrimmedCurve::new(c2d, par_f, par_l, true, true)?).cast(),
    ))
}

impl GeomIntIntSS {
    /// The curves of a restriction line.
    pub(super) fn restriction_curves(
        &mut self,
        l: &Handle<dyn IntPatchLine>,
        approx: GeomIntIntSSApprox,
    ) -> Result<(), Failure> {
        let rl: Handle<IntPatchRLine> = line_as::<IntPatchRLine>(l, "GeomInt_IntSS::MakeCurve")?;
        let mut curves = GeomIntRLineCurves::default();
        Self::treat_rline(&rl, &self.my_hs1, &self.my_hs2, &mut curves)?;
        if curves.c3d.is_null() {
            return Ok(());
        }
        let box1: BoundingBox2 = domain_box(&self.my_hs1)?;
        let box2: BoundingBox2 = domain_box(&self.my_hs2)?;
        // The intersection line is taken as same-parameter.
        let mut params: Vec<f64> = vec![curves.c3d.first_parameter(), curves.c3d.last_parameter()];
        Self::trim_iline_on_surf_boundaries(&curves.c2d1, &curves.c2d2, &box1, &box2, &mut params)?;
        for pair in params.windows(2) {
            let (par_f, par_l) = (pair[0], pair[1]);
            if (par_l - par_f) <= precision::p_confusion() {
                continue;
            }
            let Some(c2d1) = piece_2d(&curves.c2d1, &box1, (par_f, par_l), approx.approx_s1)?
            else {
                continue;
            };
            let Some(c2d2) = piece_2d(&curves.c2d2, &box2, (par_f, par_l), approx.approx_s2)?
            else {
                continue;
            };
            let curv3d: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
                &curves.c3d,
                par_f,
                par_l,
                true,
                true,
            )?)
            .cast();
            self.sline.push(curv3d);
            self.sline_s1.push(c2d1);
            self.sline_s2.push(c2d2);
        }
        Ok(())
    }
}
