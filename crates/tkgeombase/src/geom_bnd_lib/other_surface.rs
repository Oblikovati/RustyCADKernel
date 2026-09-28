// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomBndLib_OtherSurface at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_OtherSurface`: the sampling fallback for any surface adaptor.
use super::patch::Patch;
use super::sampling::{compute_nb_u_samples, compute_nb_v_samples};
use super::surface_sampling::{grid_box, surface_box_optimal};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::bnd::BoundingBox3;

/// `GeomBndLib_OtherSurface`: grid-sampled boxes of an arbitrary `Adaptor3d_Surface`.
pub struct OtherSurface<'a> {
    surface: &'a dyn Adaptor3dSurface,
}

impl<'a> OtherSurface<'a> {
    /// The evaluator of `surface`, which it borrows; `OtherSurface::new(&adaptor)`.
    pub fn new(surface: &'a dyn Adaptor3dSurface) -> Self {
        Self { surface }
    }
    fn bounds(&self) -> Result<Patch, Failure> {
        Ok(Patch {
            u_min: self.surface.first_u_parameter()?,
            u_max: self.surface.last_u_parameter()?,
            v_min: self.surface.first_v_parameter()?,
            v_max: self.surface.last_v_parameter()?,
        })
    }
    /// `Box(Tol)`: the adaptor bounds; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let p: Patch = self.bounds()?;
        self.bounding_box_range(p.u_min, p.u_max, p.v_min, p.v_max, tolerance)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: the grid of ComputeNbU/VSamples of the range;
    /// `evaluator.bounding_box_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let nu: i32 = compute_nb_u_samples(self.surface, u_min, u_max)?;
        let nv: i32 = compute_nb_v_samples(self.surface, v_min, v_max)?;
        grid_box(
            self.surface,
            Patch {
                u_min,
                u_max,
                v_min,
                v_max,
            },
            (nu, nv),
            tolerance,
        )
    }
    /// `BoxOptimal(Tol)`: the adaptor bounds; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let p: Patch = self.bounds()?;
        self.bounding_box_optimal_range(p.u_min, p.u_max, p.v_min, p.v_max, tolerance)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: the doubled grid refined by PSO +
    /// Powell; `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        surface_box_optimal(
            self.surface,
            Patch {
                u_min,
                u_max,
                v_min,
                v_max,
            },
            tolerance,
        )
    }
}
