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
// Rust translation of GeomBndLib_SurfaceOfExtrusion at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_SurfaceOfExtrusion`: the basis curve box swept along the direction.
use super::live;
use super::swept::{
    BasisBox, basis_curve_box, clamped_basis_range, extrusion_box, other_surface_box,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSweptSurface};
use rustycad_tkmath::bnd::BoundingBox3;

/// `GeomBndLib_SurfaceOfExtrusion`: the box of a `Geom_SurfaceOfLinearExtrusion` patch.
pub struct SurfaceOfExtrusion {
    geom: Handle<GeomSurfaceOfLinearExtrusion>,
}

impl SurfaceOfExtrusion {
    /// The evaluator of `surface`; `SurfaceOfExtrusion::new(handle)`.
    pub fn new(surface: Handle<GeomSurfaceOfLinearExtrusion>) -> Self {
        Self { geom: surface }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomSurfaceOfLinearExtrusion> {
        &self.geom
    }
    fn surface(&self) -> Result<&GeomSurfaceOfLinearExtrusion, Failure> {
        live(&self.geom, "GeomBndLib_SurfaceOfExtrusion")
    }
    /// `Box(Tol)`: the natural bounds; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = self.surface()?.bounds();
        self.bounding_box_range(u1, u2, v1, v2, tolerance)
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`: the basis curve box of `[UMin, UMax]`
    /// (clamped on a non-periodic curve) at `VMin` and `VMax` along the direction, opened
    /// for infinite V; a void curve box or any raised exception falls back to
    /// GeomBndLib_OtherSurface (the native `catch (Standard_Failure)`);
    /// `evaluator.bounding_box_range(0.0, 1.0, 0.0, 2.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.guarded([u_min, u_max, v_min, v_max], tolerance, BasisBox::Box)
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`: as `Box` with the basis curve's optimal
    /// box and OtherSurface's optimal fallback;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0, 2.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        self.guarded([u_min, u_max, v_min, v_max], tolerance, BasisBox::Optimal)
    }
    /// `BoxOptimal(Tol)`: the natural bounds; `evaluator.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let (u1, u2, v1, v2) = self.surface()?.bounds();
        self.bounding_box_optimal_range(u1, u2, v1, v2, tolerance)
    }
    // The native try/catch: any failure of the sweep runs the OtherSurface fallback.
    fn guarded(
        &self,
        patch: [f64; 4],
        tolerance: f64,
        kind: BasisBox,
    ) -> Result<BoundingBox3, Failure> {
        self.swept(patch, tolerance, kind)
            .or_else(|_| other_surface_box(&self.geom, patch, tolerance, kind))
    }
    fn swept(
        &self,
        patch: [f64; 4],
        tolerance: f64,
        kind: BasisBox,
    ) -> Result<BoundingBox3, Failure> {
        let [u_min, u_max, v_min, v_max] = patch;
        let surface = self.surface()?;
        let basis = surface.basis_curve();
        let range = clamped_basis_range(
            live(&basis, "GeomBndLib_SurfaceOfExtrusion")?,
            (u_min, u_max),
        );
        let curve_box: BoundingBox3 = basis_curve_box(&basis, range, kind)?;
        if curve_box.is_void() {
            return other_surface_box(&self.geom, patch, tolerance, kind);
        }
        Ok(extrusion_box(
            &curve_box,
            surface.direction(),
            (v_min, v_max),
            tolerance,
        ))
    }
}
