// Created on: 1997-03-24
// Created by: Laurent BUCHARD
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
// Rust translation of BRepTopAdaptor_TopolTool.cxx (samples) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sample points of `BRepTopAdaptor_TopolTool`: counts by surface kind (at
//! least 10 each, the pole grid of a spline analysed above 10), on a regular grid
//! strictly inside the (finite) surface range.
use super::errors::{c_int_cast, null_object};
use super::topol_tool::BRepTopAdaptorTopolTool;
use rustycad_tkernel::precision::{is_negative_infinite, is_positive_infinite};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};

impl BRepTopAdaptorTopolTool {
    // The surface, read without keeping the lock; natively a null one is dereferenced.
    fn live_surface(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.state.read().surface.clone();
        if surface.is_null() {
            return Err(null_object(
                "BRepTopAdaptor_TopolTool: no surface; expected Initialize(S) first",
            ));
        }
        Ok(surface)
    }

    /// `ComputeSamplePoints()`: the sample counts (a plane 2, a quadric 8 per unit of
    /// U and 7 per unit of V within [5, 30] x [5, 15], a revolution or extrusion
    /// 25 x 15, a Bezier 3 + poles, a B-spline knots x degree, at least 10 each; a
    /// spline above 10 is analysed by its pole grid), the grid origin and steps;
    /// `tool.compute_sample_points()?`.
    pub fn compute_sample_points(&self) -> Result<(), Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        let adapted: &dyn Adaptor3dSurface = &*surface;
        let (uinf, usup, vinf, vsup) = Adaptor3dTopolTool::sampled_box(adapted)?;
        let kind: SurfaceType = adapted.get_type()?;
        let (mut nbsu, mut nbsv) = kind_counts(adapted, kind, (usup - uinf, vsup - vinf))?;
        //-- If the number of points is too great, analyze
        nbsu = nbsu.max(10);
        nbsv = nbsv.max(10);
        if (nbsu > 10 || nbsv > 10)
            && let Some((u, v)) = analysed_counts(adapted, kind)?
        {
            (nbsu, nbsv) = (u, v);
        }
        nbsu = nbsu.max(10);
        nbsv = nbsv.max(10);
        let mut state = self.state.write();
        (state.nb_samples_u, state.nb_samples_v) = (nbsu, nbsv);
        (state.u0, state.v0) = (uinf, vinf);
        state.du = (usup - uinf) / f64::from(nbsu + 1);
        state.dv = (vsup - vinf) / f64::from(nbsv + 1);
        Ok(())
    }

    // The counts, computed on first use (`myNbSamplesU < 0`).
    fn counts(&self) -> Result<(i32, i32), Failure> {
        if self.state.read().nb_samples_u < 0 {
            self.compute_sample_points()?;
        }
        let state = self.state.read();
        Ok((state.nb_samples_u, state.nb_samples_v))
    }

    /// `NbSamplesU()`; `tool.nb_samples_u()?`.
    pub fn nb_samples_u(&self) -> Result<i32, Failure> {
        Ok(self.counts()?.0)
    }

    /// `NbSamplesV()`; `tool.nb_samples_v()?`.
    pub fn nb_samples_v(&self) -> Result<i32, Failure> {
        Ok(self.counts()?.1)
    }

    /// `NbSamples()`: `NbSamplesU * NbSamplesV`; `tool.nb_samples()?`.
    pub fn nb_samples(&self) -> Result<i32, Failure> {
        let (nbsu, nbsv) = self.counts()?;
        Ok(nbsu.wrapping_mul(nbsv))
    }

    /// `SamplePoint(Index, P2d, P3d)`: the grid point `(U0 + iu dU, V0 + iv dV)` with
    /// `iv = 1 + Index / NbSamplesU` and `iu = 1 + Index - (iv - 1) NbSamplesU`, and its
    /// surface point; it reads the counts as they are (Standard_DivideByZero natively
    /// for a zero count is a Standard_ProgramError here);
    /// `let (p2d, p3d) = tool.sample_point(1)?;`.
    pub fn sample_point(&self, index: i32) -> Result<(Point2, Point3), Failure> {
        let (u, v) = {
            let state = self.state.read();
            let nbu: i32 = state.nb_samples_u;
            if nbu == 0 {
                return Err(Failure::with_message(
                    "Standard_ProgramError",
                    "BRepTopAdaptor_TopolTool::SamplePoint: NbSamplesU is 0; expected a sample count",
                ));
            }
            let iv: i32 = 1 + index / nbu;
            let iu: i32 = 1 + index - (iv - 1) * nbu;
            (
                state.u0 + f64::from(iu) * state.du,
                state.v0 + f64::from(iv) * state.dv,
            )
        };
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        Ok((Point2::new(u, v), surface.value(u, v)?))
    }

    /// `DomainIsInfinite()`: whether a bound of the surface range is infinite;
    /// `tool.domain_is_infinite()?`.
    pub fn domain_is_infinite(&self) -> Result<bool, Failure> {
        let surface: Handle<dyn Adaptor3dSurface> = self.live_surface()?;
        Ok(is_negative_infinite(surface.first_u_parameter()?)
            || is_positive_infinite(surface.last_u_parameter()?)
            || is_negative_infinite(surface.first_v_parameter()?)
            || is_positive_infinite(surface.last_v_parameter()?))
    }
}

// The counts of ComputeSamplePoints before the minimum of 10, by surface kind.
fn kind_counts(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    (u_length, v_length): (f64, f64),
) -> Result<(i32, i32), Failure> {
    Ok(match kind {
        SurfaceType::Plane => (2, 2),
        SurfaceType::BezierSurface => (3 + surface.nb_u_poles()?, 3 + surface.nb_v_poles()?),
        SurfaceType::BSplineSurface => {
            let nbsv: i32 = surface.nb_v_knots()? * surface.v_degree()?;
            let nbsu: i32 = surface.nb_u_knots()? * surface.u_degree()?;
            (nbsu.max(4), nbsv.max(4))
        }
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus => {
            //-- Set 15 for 2pi
            //-- Not enough ->25 for 2pi
            let nbsu: i32 = c_int_cast(8.0 * u_length);
            let nbsv: i32 = c_int_cast(7.0 * v_length);
            // modif HRT buc60462: at most 30 in U.
            (nbsu.clamp(5, 30), nbsv.clamp(5, 15))
        }
        SurfaceType::SurfaceOfRevolution | SurfaceType::SurfaceOfExtrusion => (25, 15),
        _ => (10, 10),
    })
}

// The counts of the pole grid of a B-spline or Bezier surface (`Analyse`), None for
// the other kinds.
fn analysed_counts(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
) -> Result<Option<(i32, i32)>, Failure> {
    let poles = match kind {
        SurfaceType::BSplineSurface => surface.bspline()?.get().map(|bspl| bspl.poles()),
        SurfaceType::BezierSurface => surface.bezier()?.get().map(|bez| bez.poles()),
        _ => return Ok(None),
    };
    let Some(poles) = poles else {
        return Err(null_object(
            "BRepTopAdaptor_TopolTool::ComputeSamplePoints: no spline behind the surface; expected one",
        ));
    };
    Adaptor3dTopolTool::analyse_pole_grid(&poles).map(Some)
}
