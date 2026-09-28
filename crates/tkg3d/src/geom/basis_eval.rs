// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of the template parameters of ModelingData/TKG3d/Geom/Geom_OffsetCurveUtils.pxx, Geom_ExtrusionUtils.pxx, Geom_RevolutionUtils.pxx and Geom_OffsetSurfaceUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The members that the Geom evaluation templates call on their basis.
//!
//! The native templates (`template <typename CurveType>`) are instantiated both with
//! a Geom curve or surface (the Geom classes) and with an adaptor (GeomAdaptor). The
//! Rust functions are generic over these two traits instead; every Geom curve and
//! surface implements them, and the adaptors implement them in `adaptor3d` and
//! `geom_adaptor`. A Geom member cannot fail where the adaptor one can (the
//! Adaptor3d base class raises Standard_NotImplemented), so every member returns
//! `Result`.
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use super::surface::{GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point3, Vector3};

/// A basis curve as the evaluation templates see it: bounds and `EvalD0`..`EvalDN`;
/// `basis.basis_d1(0.5)?.d1`.
pub trait BasisCurveEvaluator {
    /// `FirstParameter`; `basis.basis_first_parameter()?`.
    fn basis_first_parameter(&self) -> Result<f64, Failure>;
    /// `LastParameter`; `basis.basis_last_parameter()?`.
    fn basis_last_parameter(&self) -> Result<f64, Failure>;
    /// `EvalD0`; `basis.basis_d0(0.5)?`.
    fn basis_d0(&self, u: f64) -> Result<Point3, Failure>;
    /// `EvalD1`; `basis.basis_d1(0.5)?`.
    fn basis_d1(&self, u: f64) -> Result<GeomCurveD1, Failure>;
    /// `EvalD2`; `basis.basis_d2(0.5)?`.
    fn basis_d2(&self, u: f64) -> Result<GeomCurveD2, Failure>;
    /// `EvalD3`; `basis.basis_d3(0.5)?`.
    fn basis_d3(&self, u: f64) -> Result<GeomCurveD3, Failure>;
    /// `EvalDN`; `basis.basis_dn(0.5, 4)?`.
    fn basis_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure>;
}

impl<C: GeomCurve + ?Sized> BasisCurveEvaluator for C {
    fn basis_first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.first_parameter())
    }
    fn basis_last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.last_parameter())
    }
    fn basis_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.eval_d0(u)
    }
    fn basis_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        self.eval_d1(u)
    }
    fn basis_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        self.eval_d2(u)
    }
    fn basis_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        self.eval_d3(u)
    }
    fn basis_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        self.eval_dn(u, n)
    }
}

/// A basis surface as the offset-surface templates see it: bounds, periodicity and
/// `EvalD1`..`EvalDN`; `basis.basis_d1(0.5, 0.5)?.d1u`.
pub trait BasisSurfaceEvaluator {
    /// `Bounds`; `let (u1, u2, v1, v2) = basis.basis_bounds()?;`.
    fn basis_bounds(&self) -> Result<SurfaceBounds, Failure>;
    /// `IsUPeriodic`; `basis.basis_u_periodic()?`.
    fn basis_u_periodic(&self) -> Result<bool, Failure>;
    /// `IsVPeriodic`; `basis.basis_v_periodic()?`.
    fn basis_v_periodic(&self) -> Result<bool, Failure>;
    /// `EvalD1`; `basis.basis_d1(0.5, 0.5)?`.
    fn basis_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure>;
    /// `EvalD2`; `basis.basis_d2(0.5, 0.5)?`.
    fn basis_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure>;
    /// `EvalD3`; `basis.basis_d3(0.5, 0.5)?`.
    fn basis_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure>;
    /// `EvalDN`; `basis.basis_dn(0.5, 0.5, 1, 2)?`.
    fn basis_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure>;
}

impl<S: GeomSurface + ?Sized> BasisSurfaceEvaluator for S {
    fn basis_bounds(&self) -> Result<SurfaceBounds, Failure> {
        Ok(self.bounds())
    }
    fn basis_u_periodic(&self) -> Result<bool, Failure> {
        Ok(self.is_u_periodic())
    }
    fn basis_v_periodic(&self) -> Result<bool, Failure> {
        Ok(self.is_v_periodic())
    }
    fn basis_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.eval_d1(u, v)
    }
    fn basis_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.eval_d2(u, v)
    }
    fn basis_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.eval_d3(u, v)
    }
    fn basis_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        self.eval_dn(u, v, nu, nv)
    }
}
