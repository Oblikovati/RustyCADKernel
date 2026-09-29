// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of the Adaptor2d_Curve2d members of ProjLib_CompProjectedCurve
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Adaptor2d_Curve2d` interface of `ProjLib_CompProjectedCurve`. The members
//! the class does not override (the conic accessors, D3, Resolution, IsClosed,
//! IsPeriodic, Period) keep the Standard_NotImplemented of the base class.
use super::eval::{eval_d0, eval_d1, eval_d2, eval_dn};
use super::intervals::build_intervals;
use super::{CompProjectedCurve, CompState};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurveD1, Geom2dCurveD2};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Point2, Vector2};

impl CompProjectedCurve {
    // The intervals, built and cached when not cached yet.
    fn cached_intervals(&self, shape: Shape) -> Result<Vec<f64>, Failure> {
        if let Some(cached) = self.state.read().tab_int.clone() {
            return Ok(cached);
        }
        let built: Vec<f64> = build_intervals(&self.snapshot(), shape)?;
        self.state.write().tab_int = Some(built.clone());
        Ok(built)
    }
}

impl Adaptor2dCurve2d for CompProjectedCurve {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        Ok(Handle::new(self.shallow_projection()?).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        self.snapshot().pair()?.curve.first_parameter()
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        self.snapshot().pair()?.curve.last_parameter()
    }
    /// The lowest of the curve continuity and the surface U and V continuities.
    fn continuity(&self) -> Result<Shape, Failure> {
        let state: CompState = self.snapshot();
        let pair = state.pair()?;
        let mut cont: Shape = pair.curve.continuity()?;
        let cont_su: Shape = pair.surface.u_continuity()?;
        if cont_su < cont {
            cont = cont_su;
        }
        let cont_sv: Shape = pair.surface.v_continuity()?;
        if cont_sv < cont {
            cont = cont_sv;
        }
        Ok(cont)
    }
    /// Rebuilds the cached intervals of `shape`.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.state.write().tab_int = None;
        Ok(self.cached_intervals(shape)?.len() as i32 - 1)
    }
    /// Copies the cached intervals (built for `shape` when none are cached);
    /// Standard_DimensionMismatch when `bounds` has another length.
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let values: Vec<f64> = self.cached_intervals(shape)?;
        if bounds.len() != values.len() {
            return Err(failure(
                "Standard_DimensionMismatch",
                &format!(
                    "ProjLib_CompProjectedCurve::Intervals: array of {} values; expected {}",
                    bounds.len(),
                    values.len()
                ),
            ));
        }
        for (index, value) in values.iter().enumerate() {
            bounds.as_mut_slice()[index] = *value;
        }
        Ok(())
    }
    /// A copy of the projection (every field, as the native copy constructor) on
    /// the curve trimmed to `[first, last]`.
    fn trim(
        &self,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let state: CompState = self.snapshot();
        let trimmed = state.pair()?.curve.trim(first, last, tol)?;
        let copy: CompProjectedCurve = CompProjectedCurve::from_state(state);
        copy.load_curve(&trimmed);
        Ok(Handle::new(copy).cast())
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        Ok(CurveType::OtherCurve)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        eval_d0(&self.snapshot(), u)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        eval_d1(&self.snapshot(), u)
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        eval_d2(&self.snapshot(), u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        eval_dn(&self.snapshot(), u, n)
    }
}
