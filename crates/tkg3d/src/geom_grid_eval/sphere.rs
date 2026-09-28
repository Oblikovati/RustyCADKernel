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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Sphere.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Sphere`: `P(u, v) = C + R * (cos(v) * DirU + sin(v) * Z)` on a grid.
use super::quadric::{
    QuadricFrame, QuadricTerms, UContext, quadric_grid, quadric_grid_members, scaled,
};
use super::surface_grid::{any_empty, empty_grid, invalid_orders};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::GeomSphericalSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Sphere3, Vector3};

/// Batch evaluator of a `Geom_SphericalSurface` (`GeomGridEval_Sphere`).
///
/// The sphere is read once per call (`Sphere()`), the U terms once per row and
/// `cos(v)`, `sin(v)` at every cell; a null surface or empty parameters give an empty
/// grid; `GeomGridEvalSphere::new(sphere).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalSphere {
    geom: Handle<GeomSphericalSurface>,
}

// The sphere `Data`: the frame and the radius.
struct SphereData {
    frame: QuadricFrame,
    radius: f64,
}

// `RcosV` and `RsinV` of one cell.
struct VTerms {
    r_cos: f64,
    r_sin: f64,
}

// `a * DirU[k] + b * Z[k]` for each coordinate, evaluated left to right.
fn mix(a: f64, dir: [f64; 3], b: f64, z: [f64; 3]) -> Vector3 {
    Vector3::new(
        a * dir[0] + b * z[0],
        a * dir[1] + b * z[1],
        a * dir[2] + b * z[2],
    )
}

impl SphereData {
    fn v_terms(&self, v: f64) -> VTerms {
        let (cos_v, sin_v) = (v.cos(), v.sin());
        VTerms {
            r_cos: self.radius * cos_v,
            r_sin: self.radius * sin_v,
        }
    }
    // `CX + RcosV * dirUX + RsinV * ZX`.
    fn point(&self, ctx: &UContext, t: &VTerms) -> Point3 {
        let f = &self.frame;
        let at = |k: usize| f.c[k] + t.r_cos * ctx.dir[k] + t.r_sin * f.z[k];
        Point3::new(at(0), at(1), at(2))
    }
    fn first(&self, ctx: &UContext, t: &VTerms) -> GeomGridEvalSurfD1 {
        GeomGridEvalSurfD1 {
            point: self.point(ctx, t),
            d1u: scaled(t.r_cos, ctx.ddir),
            d1v: mix(-t.r_sin, ctx.dir, t.r_cos, self.frame.z),
        }
    }
    fn second(&self, ctx: &UContext, t: &VTerms) -> GeomGridEvalSurfD2 {
        let first: GeomGridEvalSurfD1 = self.first(ctx, t);
        GeomGridEvalSurfD2 {
            point: first.point,
            d1u: first.d1u,
            d1v: first.d1v,
            d2u: scaled(-t.r_cos, ctx.dir),
            d2v: mix(-t.r_cos, ctx.dir, -t.r_sin, self.frame.z),
            d2uv: scaled(-t.r_sin, ctx.ddir),
        }
    }
}

// The V-phase coefficients of the `cos(v)` (XY) and `sin(v)` (Z) terms, period 4.
fn v_phase(nv: i32, cos_v: f64, sin_v: f64) -> (f64, f64) {
    match nv % 4 {
        0 => (cos_v, sin_v),
        1 => (-sin_v, cos_v),
        2 => (-cos_v, -sin_v),
        _ => (sin_v, -cos_v),
    }
}

impl QuadricTerms for SphereData {
    fn frame(&self) -> &QuadricFrame {
        &self.frame
    }
    fn d0(&self, ctx: &UContext, v: f64) -> Point3 {
        self.point(ctx, &self.v_terms(v))
    }
    fn d1(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD1 {
        self.first(ctx, &self.v_terms(v))
    }
    fn d2(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD2 {
        self.second(ctx, &self.v_terms(v))
    }
    // `D3V = RsinV * DirU - RcosV * Z` is written `a * DirU - b * Z` natively; the
    // negated coefficient gives the same bits.
    fn d3(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD3 {
        let t: VTerms = self.v_terms(v);
        let second: GeomGridEvalSurfD2 = self.second(ctx, &t);
        let d3v = |k: usize| t.r_sin * ctx.dir[k] - t.r_cos * self.frame.z[k];
        GeomGridEvalSurfD3 {
            point: second.point,
            d1u: second.d1u,
            d1v: second.d1v,
            d2u: second.d2u,
            d2v: second.d2v,
            d2uv: second.d2uv,
            d3u: scaled(-t.r_cos, ctx.ddir),
            d3v: Vector3::new(d3v(0), d3v(1), d3v(2)),
            d3uuv: scaled(t.r_sin, ctx.dir),
            d3uvv: scaled(-t.r_cos, ctx.ddir),
        }
    }
    // `R * (coeffVXY * dir + zTerm)`, with the Z term only when `nu == 0` (a zero is
    // added otherwise, which turns a -0 into +0 as natively).
    fn dn(&self, ctx: &UContext, v: f64, nu: i32, nv: i32) -> Vector3 {
        let (coeff_xy, coeff_z) = v_phase(nv, v.cos(), v.sin());
        let dir: [f64; 3] = ctx.dir_derivative(nu);
        let z: [f64; 3] = if nu == 0 {
            self.frame.z.map(|c| coeff_z * c)
        } else {
            [0.0; 3]
        };
        let r: f64 = self.radius;
        let at = |k: usize| r * (coeff_xy * dir[k] + z[k]);
        Vector3::new(at(0), at(1), at(2))
    }
}

impl GeomGridEvalSphere {
    /// Evaluator of `sphere` (`GeomGridEval_Sphere(theSphere)`); a null handle is
    /// accepted; `GeomGridEvalSphere::new(sphere)`.
    pub fn new(sphere: Handle<GeomSphericalSurface>) -> Self {
        Self { geom: sphere }
    }
    // `extractData` when there is something to evaluate.
    fn terms(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Option<SphereData>, Failure> {
        let Some(surface) = self.geom.get() else {
            return Ok(None);
        };
        if any_empty(u, v) {
            return Ok(None);
        }
        let sphere: Sphere3 = surface.sphere()?;
        Ok(Some(SphereData {
            frame: QuadricFrame::new(sphere.location(), sphere.position()),
            radius: sphere.radius(),
        }))
    }
    quadric_grid_members!(GeomSphericalSurface);
}
