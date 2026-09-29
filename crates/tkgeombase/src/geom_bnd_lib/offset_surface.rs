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
// Rust translation of GeomBndLib_OffsetSurface at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_OffsetSurface`: the box of the analytic equivalent surface when there is
//! one, otherwise the basis box enlarged by the offset (`Box`) or the sampled offset
//! surface (`BoxOptimal`).
use super::live;
use super::other_surface::OtherSurface;
use super::spline_surface::surface_adaptor_of;
use super::surface::Surface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;

/// `GeomBndLib_OffsetSurface`: the box of a `Geom_OffsetSurface` patch.
pub struct OffsetSurface {
    geom: Handle<GeomOffsetSurface>,
}

impl OffsetSurface {
    /// The evaluator of `surface`; `OffsetSurface::new(handle)`.
    pub fn new(surface: Handle<GeomOffsetSurface>) -> Self {
        Self { geom: surface }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomOffsetSurface> {
        &self.geom
    }
    fn offset(&self) -> Result<&GeomOffsetSurface, Failure> {
        live(&self.geom, "GeomBndLib_OffsetSurface")
    }
    // The dispatcher of `Surface()` (the equivalent analytic surface) unless it is null
    // or only an OtherSurface.
    fn equivalent(&self) -> Result<Option<Surface<'static>>, Failure> {
        let equivalent: Handle<dyn GeomSurface> = self.offset()?.surface()?;
        if equivalent.is_null() {
            return Ok(None);
        }
        let evaluator: Surface<'static> = Surface::from_surface(&equivalent)?;
        Ok((evaluator.get_type() != SurfaceType::OtherSurface).then_some(evaluator))
    }
    /// `Box(Tol)`: the natural bounds; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = self.offset()?.bounds();
        self.bounding_box_range(u1, u2, v1, v2, tolerance)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: the equivalent surface's box, otherwise the
    /// basis box `Enlarge(|offset|)` then `Enlarge(tol)`: `Enlarge` keeps the larger gap,
    /// so the result is enlarged by `max(|offset|, tol)`, not their sum (native);
    /// `evaluator.bounding_box_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        if let Some(equivalent) = self.equivalent()? {
            return equivalent.bounding_box_range(u_min, u_max, v_min, v_max, tolerance);
        }
        let offset = self.offset()?;
        let basis: Surface<'static> = Surface::from_surface(&offset.basis_surface())?;
        let mut bounds: BoundingBox3 = basis.bounding_box_range(u_min, u_max, v_min, v_max, 0.0)?;
        bounds.enlarge(offset.offset().abs());
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
    /// `BoxOptimal(Tol)`: the natural bounds; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = self.offset()?.bounds();
        self.bounding_box_optimal_range(u1, u2, v1, v2, tolerance)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: the equivalent surface's optimal box,
    /// otherwise GeomBndLib_OtherSurface on the offset surface itself;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        if let Some(equivalent) = self.equivalent()? {
            return equivalent.bounding_box_optimal_range(u_min, u_max, v_min, v_max, tolerance);
        }
        let adaptor: GeomAdaptorSurface = surface_adaptor_of(&self.geom)?;
        let adaptor: &dyn Adaptor3dSurface = &adaptor;
        OtherSurface::new(adaptor).bounding_box_optimal_range(u_min, u_max, v_min, v_max, tolerance)
    }
}
