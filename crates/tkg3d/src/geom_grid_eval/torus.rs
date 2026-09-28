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
// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Torus.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Torus`: `P(u, v) = C + (R + r * cos(v)) * DirU + r * sin(v) * Z` on a grid.
use super::quadric::{
    QuadricFrame, QuadricTerms, UContext, quadric_grid, quadric_grid_members, scaled,
};
use super::surface_grid::{any_empty, empty_grid, invalid_orders};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use crate::geom::GeomToroidalSurface;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Torus3, Vector3};

/// Batch evaluator of a `Geom_ToroidalSurface` (`GeomGridEval_Torus`).
///
/// The torus is read once per call (`Torus()`), the U terms once per row. The D3
/// formulas regroup the terms (`Som1`, `Som3`, `A3`..`A6`) and give bits that can
/// differ from the D1 and D2 formulas of the same point, as natively; a null surface or
/// empty parameters give an empty grid;
/// `GeomGridEvalTorus::new(torus).evaluate_grid(&u, &v)?`.
pub struct GeomGridEvalTorus {
    geom: Handle<GeomToroidalSurface>,
}

// The torus `Data`: the frame and both radii.
struct TorusData {
    frame: QuadricFrame,
    major: f64,
    minor: f64,
}

// `Vector3` of `f(k)` for each coordinate.
fn build(f: impl Fn(usize) -> f64) -> Vector3 {
    Vector3::new(f(0), f(1), f(2))
}

impl TorusData {
    // `CX + K * dirUX + MinorRadius * sinV * ZX` with `K = R + r * cosV`.
    fn point(&self, ctx: &UContext, cos_v: f64, sin_v: f64) -> Point3 {
        let (f, r) = (&self.frame, self.minor);
        let k: f64 = self.major + r * cos_v;
        let at = |i: usize| f.c[i] + k * ctx.dir[i] + r * sin_v * f.z[i];
        Point3::new(at(0), at(1), at(2))
    }
    // D1U = K * dDirU, D1V = -r * sinV * DirU + r * cosV * Z.
    fn first(&self, ctx: &UContext, cos_v: f64, sin_v: f64) -> GeomGridEvalSurfD1 {
        let (r, z) = (self.minor, self.frame.z);
        let k: f64 = self.major + r * cos_v;
        GeomGridEvalSurfD1 {
            point: self.point(ctx, cos_v, sin_v),
            d1u: scaled(k, ctx.ddir),
            d1v: build(|i| -r * sin_v * ctx.dir[i] + r * cos_v * z[i]),
        }
    }
}

// The regrouped terms of `computeD3`.
struct ThirdOrderTerms {
    r1: f64,
    r2: f64,
    a: [f64; 4],
    som1: [f64; 3],
    som3: [f64; 3],
}

impl TorusData {
    fn third_terms(&self, ctx: &UContext, cos_v: f64, sin_v: f64) -> ThirdOrderTerms {
        let (r1, r2) = (self.minor * cos_v, self.minor * sin_v);
        let k: f64 = self.major + r1;
        let a = [
            r2 * ctx.cos_u,
            r2 * ctx.sin_u,
            r1 * ctx.cos_u,
            r1 * ctx.sin_u,
        ];
        let (x, y) = (self.frame.x, self.frame.y);
        ThirdOrderTerms {
            r1,
            r2,
            a,
            som1: ctx.dir.map(|d| k * d),
            som3: [0, 1, 2].map(|i| a[0] * x[i] + a[1] * y[i]),
        }
    }
    // `a * X - b * Y` (the D2UV and D3UVV forms).
    fn x_minus_y(&self, a: f64, b: f64) -> Vector3 {
        let (x, y) = (self.frame.x, self.frame.y);
        build(|i| a * x[i] - b * y[i])
    }
    fn third(&self, ctx: &UContext, cos_v: f64, sin_v: f64) -> GeomGridEvalSurfD3 {
        let t: ThirdOrderTerms = self.third_terms(ctx, cos_v, sin_v);
        let (f, [a3, a4, a5, a6]) = (&self.frame, t.a);
        let k: f64 = self.major + t.r1;
        GeomGridEvalSurfD3 {
            point: Point3::new(
                f.c[0] + t.som1[0] + t.r2 * f.z[0],
                f.c[1] + t.som1[1] + t.r2 * f.z[1],
                f.c[2] + t.som1[2] + t.r2 * f.z[2],
            ),
            d1u: scaled(k, ctx.ddir),
            d1v: build(|i| -t.som3[i] + t.r1 * f.z[i]),
            d2u: build(|i| -t.som1[i]),
            d2v: build(|i| -a5 * f.x[i] - a6 * f.y[i] - t.r2 * f.z[i]),
            d2uv: self.x_minus_y(a4, a3),
            // `d3U = -dU`: the negated product, the same bits as `-K * dDirU`.
            d3u: build(|i| -(k * ctx.ddir[i])),
            d3v: build(|i| t.som3[i] - t.r1 * f.z[i]),
            d3uuv: build(|i| t.som3[i]),
            d3uvv: self.x_minus_y(a6, a5),
        }
    }
}

// The V-phase derivatives of `cos(v)` and `sin(v)`, period 4.
fn v_phase(nv: i32, cos_v: f64, sin_v: f64) -> (f64, f64) {
    match nv % 4 {
        0 => (cos_v, sin_v),
        1 => (-sin_v, cos_v),
        2 => (-cos_v, -sin_v),
        _ => (sin_v, -cos_v),
    }
}

impl QuadricTerms for TorusData {
    fn frame(&self) -> &QuadricFrame {
        &self.frame
    }
    fn d0(&self, ctx: &UContext, v: f64) -> Point3 {
        self.point(ctx, v.cos(), v.sin())
    }
    fn d1(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD1 {
        self.first(ctx, v.cos(), v.sin())
    }
    // D2U = -K * DirU, D2V = -r*cosV*DirU - r*sinV*Z, D2UV = -r*sinV*dDirU.
    fn d2(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD2 {
        let (cos_v, sin_v) = (v.cos(), v.sin());
        let first: GeomGridEvalSurfD1 = self.first(ctx, cos_v, sin_v);
        let (r, z) = (self.minor, self.frame.z);
        let k: f64 = self.major + r * cos_v;
        GeomGridEvalSurfD2 {
            point: first.point,
            d1u: first.d1u,
            d1v: first.d1v,
            d2u: scaled(-k, ctx.dir),
            d2v: build(|i| -r * cos_v * ctx.dir[i] - r * sin_v * z[i]),
            d2uv: build(|i| -r * sin_v * ctx.ddir[i]),
        }
    }
    fn d3(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD3 {
        self.third(ctx, v.cos(), v.sin())
    }
    // nv == 0: K(v) * d^nu(DirU); nu == 0: dK * DirU + r * d^nv(sin) * Z; mixed:
    // r * d^nv(cos) * d^nu(DirU).
    fn dn(&self, ctx: &UContext, v: f64, nu: i32, nv: i32) -> Vector3 {
        let (cos_v, sin_v) = (v.cos(), v.sin());
        let dir: [f64; 3] = ctx.dir_derivative(nu);
        let (d_cos, d_sin) = v_phase(nv, cos_v, sin_v);
        let r: f64 = self.minor;
        if nv == 0 {
            return scaled(self.major + r * cos_v, dir);
        }
        if nu == 0 {
            let (dk, z) = (r * d_cos, self.frame.z);
            return build(|i| dk * dir[i] + r * d_sin * z[i]);
        }
        scaled(r * d_cos, dir)
    }
}

impl GeomGridEvalTorus {
    /// Evaluator of `torus` (`GeomGridEval_Torus(theTorus)`); a null handle is
    /// accepted; `GeomGridEvalTorus::new(torus)`.
    pub fn new(torus: Handle<GeomToroidalSurface>) -> Self {
        Self { geom: torus }
    }
    // `extractData` when there is something to evaluate.
    fn terms(
        &self,
        u: &Array1<'_, f64>,
        v: &Array1<'_, f64>,
    ) -> Result<Option<TorusData>, Failure> {
        let Some(surface) = self.geom.get() else {
            return Ok(None);
        };
        if any_empty(u, v) {
            return Ok(None);
        }
        let torus: Torus3 = surface.torus()?;
        Ok(Some(TorusData {
            frame: QuadricFrame::new(torus.location(), torus.position()),
            major: torus.major_radius(),
            minor: torus.minor_radius(),
        }))
    }
    quadric_grid_members!(GeomToroidalSurface);
}
