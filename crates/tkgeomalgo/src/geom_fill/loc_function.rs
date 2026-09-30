// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_LocFunction.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_LocFunction`: a location law as the 12 reals `V, M(1), M(2), M(3)`
//! (and their derivatives) that `AdvApprox_ApproxAFunction` approximates.
use super::location_law::{
    GeomFillLocationLaw, LocationD0, LocationD1, LocationD2, LocationPoles2d,
};
use super::sweep_failures::{live, null_object};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkmath::gp::{Matrix3, Point2, Vector2, Vector3};

/// `GeomFill_LocFunction`: the evaluator of a location law.
///
/// `V`, `DV` and `D2V` keep the last evaluation (the translation then the three
/// matrix columns), as the native members.
/// `let mut f = GeomFillLocFunction::new(&law); f.d0(0.5, 0.0, 1.0)?;`.
pub struct GeomFillLocFunction {
    my_law: Handle<dyn GeomFillLocationLaw>,
    v: [Vector3; 4],
    dv: [Vector3; 4],
    d2v: [Vector3; 4],
}

/// `V(1) = v`, `V(2..4)` the columns of `m`.
fn store(target: &mut [Vector3; 4], v: Vector3, m: &Matrix3) -> Result<(), Failure> {
    target[0] = v;
    for (index, slot) in target.iter_mut().enumerate().skip(1) {
        *slot = Vector3::from(m.column(index as i32)?);
    }
    Ok(())
}

impl GeomFillLocFunction {
    /// `GeomFill_LocFunction(Law)`; `GeomFillLocFunction::new(&law)`.
    pub fn new(law: &Handle<dyn GeomFillLocationLaw>) -> Self {
        Self {
            my_law: law.clone(),
            v: [Vector3::default(); 4],
            dv: [Vector3::default(); 4],
            d2v: [Vector3::default(); 4],
        }
    }

    fn law(&self) -> Result<&dyn GeomFillLocationLaw, Failure> {
        live(&self.my_law, "GeomFill_LocFunction", "myLaw")
    }

    /// `D0(Param, First, Last)`: evaluates `V` and `M` (the interval is unused).
    pub fn d0(&mut self, param: f64, _first: f64, _last: f64) -> Result<bool, Failure> {
        let mut placement = LocationD0 {
            v: self.v[0],
            ..LocationD0::default()
        };
        let b = self.law()?.d0(param, &mut placement)?;
        store(&mut self.v, placement.v, &placement.m)?;
        Ok(b)
    }

    /// `D1(Param, First, Last)`: with the first derivatives.
    pub fn d1(&mut self, param: f64, _first: f64, _last: f64) -> Result<bool, Failure> {
        let (mut t1, mut t2): (Array1<'static, Point2>, Array1<'static, Vector2>) =
            (one_based(1)?, one_based(1)?);
        let mut placement = LocationD1 {
            v: self.v[0],
            dv: self.dv[0],
            ..LocationD1::default()
        };
        let b = self.law()?.d1(param, &mut placement, &mut t1, &mut t2)?;
        store(&mut self.v, placement.v, &placement.m)?;
        store(&mut self.dv, placement.dv, &placement.dm)?;
        Ok(b)
    }

    /// `D2(Param, First, Last)`: with the second derivatives.
    pub fn d2(&mut self, param: f64, _first: f64, _last: f64) -> Result<bool, Failure> {
        let mut t1: Array1<'static, Point2> = one_based(1)?;
        let (mut t2, mut t3): (Array1<'static, Vector2>, Array1<'static, Vector2>) =
            (one_based(1)?, one_based(1)?);
        let mut placement = LocationD2 {
            v: self.v[0],
            dv: self.dv[0],
            d2v: self.d2v[0],
            ..LocationD2::default()
        };
        let poles = LocationPoles2d {
            poles2d: &mut t1,
            d_poles2d: &mut t2,
            d2_poles2d: &mut t3,
        };
        let b = self.law()?.d2(param, &mut placement, poles)?;
        store(&mut self.v, placement.v, &placement.m)?;
        store(&mut self.dv, placement.dv, &placement.dm)?;
        store(&mut self.d2v, placement.d2v, &placement.d2m)?;
        Ok(b)
    }

    /// `DN(Param, First, Last, Order, Result, Ier)`: the 12 reals of order 0, 1 or
    /// 2 into `result`, and `Ier = Order + 1` when the law failed. Another order
    /// reads a null pointer natively; here it is Standard_NullObject.
    pub fn dn(
        &mut self,
        param: f64,
        first: f64,
        last: f64,
        order: i32,
        result: &mut [f64; 12],
    ) -> Result<i32, Failure> {
        let (b, values) = match order {
            0 => (self.d0(param, first, last)?, self.v),
            1 => (self.d1(param, first, last)?, self.dv),
            2 => (self.d2(param, first, last)?, self.d2v),
            _ => {
                return Err(null_object(
                    &format!("GeomFill_LocFunction::DN (Order = {order}; expected 0, 1 or 2)"),
                    "LocalResult",
                ));
            }
        };
        for (index, v) in values.iter().enumerate() {
            result[3 * index] = v.x();
            result[3 * index + 1] = v.y();
            result[3 * index + 2] = v.z();
        }
        Ok(if b { 0 } else { order + 1 })
    }
}
