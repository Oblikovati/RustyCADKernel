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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SnglrFunc.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_SnglrFunc`: the curve `ratio * C'(t) ^ C''(t)` of a curve `C`, whose
//! zeros are the points where the Frenet frame is singular.
use super::sweep_failures::{live, not_implemented, sweep_failure};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurveD1, GeomCurveD2, GeomCurveD3};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point3, Vector3};

/// `GeomFill_SnglrFunc`: an `Adaptor3d_Curve` evaluating `ratio * C' ^ C''` and its
/// derivatives on the curve `C` it holds.
///
/// Natively the class declares no RTTI, so its dynamic type is `Adaptor3d_Curve`; the
/// port registers it under its own name (docs/tkgeomalgo-progress.md).
/// `let func = GeomFillSnglrFunc::new(&curve); func.set_ratio(0.5); func.value(0.0)?`.
pub struct GeomFillSnglrFunc {
    parent: StandardTransient,
    my_h_curve: Handle<dyn Adaptor3dCurve>,
    ratio: SharedState<f64>,
}
impl_transient!(
    GeomFillSnglrFunc,
    "GeomFill_SnglrFunc",
    dyn Adaptor3dCurve,
    parent,
    interfaces: [dyn Adaptor3dCurve]
);

/// The derivative orders of `S` that `GetType`-independent members map `S` to:
/// C0 -> C2, C1 -> C3, C2 and above -> CN (`NbIntervals`, `Intervals`).
fn shifted_shape(shape: Shape) -> Shape {
    if shape == Shape::C0 {
        return Shape::C2;
    }
    if shape == Shape::C1 {
        return Shape::C3;
    }
    if shape >= Shape::C2 {
        return Shape::CN;
    }
    Shape::C0
}

/// The value `gp_Pnt(v.XYZ())` of a vector.
fn point_of(v: Vector3) -> Point3 {
    Point3::new(v.x(), v.y(), v.z())
}

impl GeomFillSnglrFunc {
    /// `GeomFill_SnglrFunc(HC)`: the function of `curve` with ratio 1;
    /// `GeomFillSnglrFunc::new(&curve)`.
    pub fn new(curve: &Handle<dyn Adaptor3dCurve>) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_h_curve: curve.clone(),
            ratio: SharedState::new(1.0),
        }
    }

    /// `SetRatio(Ratio)`: the scale of the function; `func.set_ratio(2.0)`.
    pub fn set_ratio(&self, ratio: f64) {
        *self.ratio.write() = ratio;
    }

    fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        live(&self.my_h_curve, "GeomFill_SnglrFunc", "myHCurve")
    }

    fn ratio(&self) -> f64 {
        *self.ratio.read()
    }
}

impl Adaptor3dCurve for GeomFillSnglrFunc {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let copy = GeomFillSnglrFunc::new(&self.curve()?.shallow_copy()?);
        copy.set_ratio(self.ratio());
        Ok(Handle::new(copy).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        self.curve()?.first_parameter()
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        self.curve()?.last_parameter()
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.curve()?.nb_intervals(shifted_shape(shape))
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.curve()?.intervals(bounds, shifted_shape(shape))
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        self.curve()?.is_periodic()
    }
    fn period(&self) -> Result<f64, Failure> {
        self.curve()?.period()
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let res = self.curve()?.eval_d2(u)?;
        Ok(point_of((res.d1 * self.ratio()).crossed(res.d2)))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let res = self.curve()?.eval_d3(u)?;
        let dc = res.d1 * self.ratio();
        Ok(GeomCurveD1 {
            point: point_of(dc.crossed(res.d2)),
            d1: dc.crossed(res.d3),
        })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let curve = self.curve()?;
        let (res, d4, ratio) = (curve.eval_d3(u)?, curve.eval_dn(u, 4)?, self.ratio());
        Ok(GeomCurveD2 {
            point: point_of(res.d1.crossed(res.d2) * ratio),
            d1: res.d1.crossed(res.d3) * ratio,
            d2: (res.d2.crossed(res.d3) + res.d1.crossed(d4)) * ratio,
        })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let curve = self.curve()?;
        let res = curve.eval_d3(u)?;
        let (d4, d5, ratio) = (curve.eval_dn(u, 4)?, curve.eval_dn(u, 5)?, self.ratio());
        Ok(GeomCurveD3 {
            point: point_of(res.d1.crossed(res.d2) * ratio),
            d1: res.d1.crossed(res.d3) * ratio,
            d2: (res.d2.crossed(res.d3) + res.d1.crossed(d4)) * ratio,
            d3: (res.d1.crossed(d5) + res.d2.crossed(d4) * 2.0) * ratio,
        })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        if n < 1 {
            return Err(sweep_failure(
                "Standard_RangeError",
                &format!("Exception: Geom2d_OffsetCurve::DN(). N<1. (N = {n}; expected N >= 1)"),
            ));
        }
        match n {
            1 => Ok(self.eval_d1(u)?.d1),
            2 => Ok(self.eval_d2(u)?.d2),
            3 => Ok(self.eval_d3(u)?.d3),
            _ => Err(not_implemented(
                "Exception: Derivative order is greater than 3. Cannot compute of derivative.",
            )),
        }
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        Ok(precision::parametric(r3d))
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(CurveType::OtherCurve)
    }
}
