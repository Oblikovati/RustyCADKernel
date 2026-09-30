// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Tool2d.cxx (TrimBisector, Distance and BisecFusion) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trims of the geometric bisectors and their fusion.
use super::{MAT2D_TOLCONF, Mat2dTool2d};
use crate::bisector::{Bisec, BisecAna, BisecCC, BisectorCurve};
use crate::mat::MatBisector;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve, errors};
use rustycad_tkmath::gp::{Point2, Vector2};
use std::f64::consts::PI;

// A bisector basis of the expected class; another one is a native null dereference.
fn basis_as<T: rustycad_tkernel::standard::RuntimeTyped + ?Sized>(
    curve: &Handle<dyn Geom2dCurve>,
    class: &str,
) -> Result<Handle<T>, Failure> {
    let basis: Handle<T> = curve.cast();
    if basis.is_null() {
        return Err(errors::null_object(&format!(
            "MAT2d_Tool2d: the bisector is not a {class}; expected that class"
        )));
    }
    Ok(basis)
}

impl Mat2dTool2d {
    /// `TrimBisector(abisector)`: the geometric bisector ends at the first parameter
    /// of the MAT bisector (a turn after its start on a periodic curve when it is
    /// infinite); false when that is its start; `tool.trim_bisector(&bisector)?`.
    pub fn trim_bisector(&mut self, abisector: &Handle<MatBisector>) -> Result<bool, Failure> {
        let mut param: f64 = abisector.first_parameter();
        let bisector: Handle<Geom2dTrimmedCurve> = self
            .change_geom_bis(abisector.bisector_number())?
            .change_value();
        let basis: Handle<dyn Geom2dCurve> = bisector.basis_curve();
        if basis.is_periodic() && param == precision::infinite() {
            param = bisector.first_parameter() + 2. * PI;
        }
        if param > basis.last_parameter() {
            param = basis.last_parameter();
        }
        if bisector.first_parameter() == param {
            return Ok(false);
        }
        bisector.set_trim(bisector.first_parameter(), param, true, true)?;
        Ok(true)
    }

    /// `TrimBisector(abisector, apoint)`: the geometric bisector ends at the point
    /// `apoint` when it is after its start and within it;
    /// `tool.trim_bisector_at(&bisector, 3)?`.
    pub fn trim_bisector_at(
        &mut self,
        abisector: &Handle<MatBisector>,
        apoint: i32,
    ) -> Result<bool, Failure> {
        let point: Point2 = self.geom_pnt(apoint)?;
        let bisector: Handle<Geom2dTrimmedCurve> = self
            .change_geom_bis(abisector.bisector_number())?
            .change_value();
        let bis: Handle<dyn BisectorCurve> = basis_as(&bisector.basis_curve(), "Bisector_Curve")?;
        let mut param: f64 = bis.parameter(point)?;
        if bisector.basis_curve().is_periodic() && bisector.first_parameter() > param {
            param += 2. * PI;
        }
        if bisector.first_parameter() >= param || bisector.last_parameter() < param {
            return Ok(false);
        }
        bisector.set_trim(bisector.first_parameter(), param, true, true)?;
        Ok(true)
    }

    /// `Distance(abisector, param1, param2)`: the distance between the points of two
    /// parameters of the geometric bisector, infinite when one is;
    /// `tool.distance(&bisector, 0.0, 1.0)?`.
    pub fn distance(
        &self,
        bis: &Handle<MatBisector>,
        param1: f64,
        param2: f64,
    ) -> Result<f64, Failure> {
        if param1 == precision::infinite() || param2 == precision::infinite() {
            return Ok(precision::infinite());
        }
        let value: Handle<Geom2dTrimmedCurve> = self.geom_bis(bis.bisector_number())?.value();
        let p1: Point2 = value.value(param1)?;
        let p2: Point2 = value.value(param2)?;
        Ok(p1.distance(p2))
    }

    /// `BisecFusion(Index1, Index2)`: the bisector `index1` is extended by the length
    /// of the bisector `index2`; a curve-curve bisector is rebuilt from the end of the
    /// second one (it cannot be trimmed before its start point);
    /// `tool.bisec_fusion(1, 2)?`.
    pub fn bisec_fusion(&mut self, i1: i32, i2: i32) -> Result<(), Failure> {
        let bisector1: Handle<Geom2dTrimmedCurve> = self.geom_bis(i1)?.value();
        let bisector2: Handle<Geom2dTrimmedCurve> = self.geom_bis(i2)?.value();
        let (uf1, ul1) = (bisector1.first_parameter(), bisector1.last_parameter());
        if bisector1.basis_curve().cast::<BisecCC>().is_some() {
            return self.rebuild_curves_bisector(i1, &bisector1, &bisector2);
        }
        let du: f64 = bisector2.last_parameter() - bisector2.first_parameter();
        let uf1: f64 = uf1 - du;
        let b_ana: Handle<BisecAna> = basis_as(&bisector1.basis_curve(), "Bisector_BisecAna")?;
        b_ana.set_trim(uf1, ul1)?;
        bisector1.set_trim(uf1, ul1, true, true)
    }

    // The curve-curve bisector of the fusion: the bisector of the two curves from the
    // end of the second bisector, trimmed at the end of the first one.
    fn rebuild_curves_bisector(
        &mut self,
        i1: i32,
        bisector1: &Handle<Geom2dTrimmedCurve>,
        bisector2: &Handle<Geom2dTrimmedCurve>,
    ) -> Result<(), Failure> {
        let v_bid = Vector2::new(1., 0.);
        let p2: Point2 = bisector2.value(bisector2.last_parameter())?;
        let p1: Point2 = bisector1.value(bisector1.last_parameter())?;
        let bcc1: Handle<BisecCC> = basis_as(&bisector1.basis_curve(), "Bisector_BisecCC")?;
        let mut bis = Bisec::new();
        bis.perform_curves(
            &bcc1.curve(2)?,
            &bcc1.curve(1)?,
            p2,
            v_bid,
            v_bid,
            self.the_direction,
            self.the_join_type,
            MAT2D_TOLCONF,
            false,
        )?;
        let new_bisector1: Handle<Geom2dTrimmedCurve> = bis.value();
        let bcc1: Handle<BisecCC> = basis_as(&new_bisector1.basis_curve(), "Bisector_BisecCC")?;
        let uf1: f64 = bcc1.first_parameter();
        let ul1: f64 = bcc1.parameter(p1)?;
        new_bisector1.set_trim(uf1, ul1, true, true)?;
        self.the_geom_bisectors.insert(i1, bis);
        Ok(())
    }
}
