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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Cone.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Cone`: `P(u, v) = C + (R + v * sin(A)) * DirU + v * cos(A) * Z` on a grid.
use super::quadric::{
    QuadricFrame, QuadricTerms, UContext, quadric_grid, quadric_grid_members, scaled, zero,
};
use super::surface_grid::{any_empty, empty_grid, invalid_orders};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::GeomConicalSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Cone3, Point3, Vector3};

/// Batch evaluator of a `Geom_ConicalSurface` (`GeomGridEval_Cone`).
///
/// The cone is read once per call (`Cone()`, with `sin` and `cos` of the semi-angle),
/// the U terms once per row; a null surface or empty parameters give an empty grid;
/// `GeomGridEvalCone::new(cone).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalCone {
    geom: Handle<GeomConicalSurface>,
}

// The cone `Data`: the frame, the reference radius and the semi-angle terms.
struct ConeData {
    frame: QuadricFrame,
    ref_radius: f64,
    sin_ang: f64,
    cos_ang: f64,
}

impl ConeData {
    // `K(v) = RefRadius + v * sin(Ang)`.
    fn radius_at(&self, v: f64) -> f64 {
        self.ref_radius + v * self.sin_ang
    }
    // `CX + K * dirUX + ZOff * ZX` with `ZOff = v * cos(Ang)`.
    fn point(&self, ctx: &UContext, v: f64) -> Point3 {
        let (f, k, z_off) = (&self.frame, self.radius_at(v), v * self.cos_ang);
        let at = |i: usize| f.c[i] + k * ctx.dir[i] + z_off * f.z[i];
        Point3::new(at(0), at(1), at(2))
    }
    // `D(0,1) = sin(Ang) * DirU + cos(Ang) * Z`.
    fn d1v(&self, ctx: &UContext) -> Vector3 {
        let (s, c, z) = (self.sin_ang, self.cos_ang, self.frame.z);
        let at = |i: usize| s * ctx.dir[i] + c * z[i];
        Vector3::new(at(0), at(1), at(2))
    }
}

impl QuadricTerms for ConeData {
    fn frame(&self) -> &QuadricFrame {
        &self.frame
    }
    fn d0(&self, ctx: &UContext, v: f64) -> Point3 {
        self.point(ctx, v)
    }
    fn d1(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD1 {
        GeomGridEvalSurfD1 {
            point: self.point(ctx, v),
            d1u: scaled(self.radius_at(v), ctx.ddir),
            d1v: self.d1v(ctx),
        }
    }
    // D2U = -K * DirU, D2V = 0, D2UV = sin(Ang) * dDirU.
    fn d2(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD2 {
        let first: GeomGridEvalSurfD1 = self.d1(ctx, v);
        GeomGridEvalSurfD2 {
            point: first.point,
            d1u: first.d1u,
            d1v: first.d1v,
            d2u: scaled(-self.radius_at(v), ctx.dir),
            d2v: zero(),
            d2uv: scaled(self.sin_ang, ctx.ddir),
        }
    }
    // D3U = -K * dDirU, D3UUV = -sin(Ang) * DirU, D3V = D3UVV = 0.
    fn d3(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD3 {
        let second: GeomGridEvalSurfD2 = self.d2(ctx, v);
        GeomGridEvalSurfD3 {
            point: second.point,
            d1u: second.d1u,
            d1v: second.d1v,
            d2u: second.d2u,
            d2v: second.d2v,
            d2uv: second.d2uv,
            d3u: scaled(-self.radius_at(v), ctx.ddir),
            d3v: zero(),
            d3uuv: scaled(-self.sin_ang, ctx.dir),
            d3uvv: zero(),
        }
    }
    // V is linear: NV >= 2 is zero; D(nu,1) = sin(Ang) * d^nu(DirU) for nu >= 1;
    // D(nu,0) = K(v) * d^nu(DirU).
    fn dn(&self, ctx: &UContext, v: f64, nu: i32, nv: i32) -> Vector3 {
        if nv > 1 {
            return zero();
        }
        let dir: [f64; 3] = ctx.dir_derivative(nu);
        match (nu, nv) {
            (0, 1) => self.d1v(ctx),
            (_, 1) => scaled(self.sin_ang, dir),
            _ => scaled(self.radius_at(v), dir),
        }
    }
}

impl GeomGridEvalCone {
    /// Evaluator of `cone` (`GeomGridEval_Cone(theCone)`); a null handle is accepted;
    /// `GeomGridEvalCone::new(cone)`.
    pub fn new(cone: Handle<GeomConicalSurface>) -> Self {
        Self { geom: cone }
    }
    // `extractData` when there is something to evaluate.
    fn terms(&self, u: &Array1<'_, f64>, v: &Array1<'_, f64>) -> Result<Option<ConeData>, Failure> {
        let Some(surface) = self.geom.get() else {
            return Ok(None);
        };
        if any_empty(u, v) {
            return Ok(None);
        }
        let cone: Cone3 = surface.cone()?;
        Ok(Some(ConeData {
            frame: QuadricFrame::new(cone.location(), cone.position()),
            ref_radius: cone.ref_radius(),
            sin_ang: cone.semi_angle().sin(),
            cos_ang: cone.semi_angle().cos(),
        }))
    }
    quadric_grid_members!(GeomConicalSurface);
}
