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
// Rust translation of the Data/UContext code shared by ModelingData/TKG3d/GeomGridEval/GeomGridEval_{Cylinder,Sphere,Cone,Torus}.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cylinder, sphere, cone and torus evaluators share their pre-extracted frame
//! (`Data`), their per-row U terms (`UContext`, `computeUContext`), the cyclic U
//! derivative of `DirU` and the grid loops; `QuadricTerms` holds the per-surface
//! formulas.
use super::surface_grid::map_rows;
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::gp::{Axis3, Point3, Vector3};

/// The center and the X, Y, Z directions (`CX..CZ, XX..XZ, YX..YZ, ZX..ZZ`).
#[derive(Clone, Copy)]
pub(super) struct QuadricFrame {
    pub(super) c: [f64; 3],
    pub(super) x: [f64; 3],
    pub(super) y: [f64; 3],
    pub(super) z: [f64; 3],
}

/// The U-dependent values of one grid row (`UContext`): `DirU = cosU*X + sinU*Y` and
/// `dDirU = -sinU*X + cosU*Y`.
#[derive(Clone, Copy)]
pub(super) struct UContext {
    pub(super) cos_u: f64,
    pub(super) sin_u: f64,
    pub(super) dir: [f64; 3],
    pub(super) ddir: [f64; 3],
}

impl QuadricFrame {
    /// The frame of `Location()` and `Position()`.
    pub(super) fn new(location: Point3, position: Axis3) -> Self {
        let (x, y, z) = (
            position.x_direction(),
            position.y_direction(),
            position.direction(),
        );
        Self {
            c: [location.x(), location.y(), location.z()],
            x: [x.x(), x.y(), x.z()],
            y: [y.x(), y.y(), y.z()],
            z: [z.x(), z.y(), z.z()],
        }
    }
    /// `computeUContext`.
    pub(super) fn u_context(&self, u: f64) -> UContext {
        let (cos_u, sin_u) = (u.cos(), u.sin());
        let (x, y) = (self.x, self.y);
        UContext {
            cos_u,
            sin_u,
            dir: [0, 1, 2].map(|k| cos_u * x[k] + sin_u * y[k]),
            ddir: [0, 1, 2].map(|k| -sin_u * x[k] + cos_u * y[k]),
        }
    }
    /// The Z direction as a vector (`gp_Vec(ZX, ZY, ZZ)`).
    pub(super) fn z_vector(&self) -> Vector3 {
        vector(self.z)
    }
}

impl UContext {
    /// `d^n(DirU)/du^n` for `n >= 0`, cyclic with period 4: `DirU`, `dDirU`, `-DirU`,
    /// `-dDirU` (the `switch (theNU % 4)` of every DN).
    pub(super) fn dir_derivative(&self, n: i32) -> [f64; 3] {
        match n % 4 {
            0 => self.dir,
            1 => self.ddir,
            2 => self.dir.map(|c| -c),
            _ => self.ddir.map(|c| -c),
        }
    }
}

/// `gp_Vec(a[0], a[1], a[2])`.
pub(super) fn vector(a: [f64; 3]) -> Vector3 {
    Vector3::new(a[0], a[1], a[2])
}

/// `gp_Vec(k * a[0], k * a[1], k * a[2])`.
pub(super) fn scaled(k: f64, a: [f64; 3]) -> Vector3 {
    Vector3::new(k * a[0], k * a[1], k * a[2])
}

/// The zero vector.
pub(super) fn zero() -> Vector3 {
    Vector3::new(0.0, 0.0, 0.0)
}

/// The formulas of one quadric evaluator (`computeD0`..`computeDN`).
pub(super) trait QuadricTerms {
    /// The pre-extracted frame.
    fn frame(&self) -> &QuadricFrame;
    /// `computeD0`.
    fn d0(&self, ctx: &UContext, v: f64) -> Point3;
    /// `computeD1`.
    fn d1(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD1;
    /// `computeD2`.
    fn d2(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD2;
    /// `computeD3`.
    fn d3(&self, ctx: &UContext, v: f64) -> GeomGridEvalSurfD3;
    /// `computeDN`, for valid orders.
    fn dn(&self, ctx: &UContext, v: f64, nu: i32, nv: i32) -> Vector3;
}

/// The grid loop of every quadric member: one `UContext` per U value, then `cell` for
/// every V value.
pub(super) fn quadric_grid<K: QuadricTerms, T: Default>(
    k: &K,
    u: &Array1<'_, f64>,
    v: &Array1<'_, f64>,
    cell: impl Fn(&K, &UContext, f64) -> T,
) -> Array2<'static, T> {
    map_rows(u, v, |u| k.frame().u_context(u), |ctx, v| cell(k, ctx, v))
}

/// The public members of a quadric evaluator whose `terms(u, v)` reads the surface
/// (`Result<Option<K>, Failure>`, `None` for a null surface or empty parameters).
macro_rules! quadric_grid_members {
    ($geom:ty) => {
        /// The evaluated surface (`Geometry`); `evaluator.geometry().is_null()`.
        pub fn geometry(&self) -> &Handle<$geom> {
            &self.geom
        }
        /// Points at every `(u, v)` (`EvaluateGrid`); `evaluator.evaluate_grid(&u, &v)?`.
        pub fn evaluate_grid(
            &self,
            u: &Array1<'_, f64>,
            v: &Array1<'_, f64>,
        ) -> Result<Array2<'static, Point3>, Failure> {
            Ok(match self.terms(u, v)? {
                Some(k) => quadric_grid(&k, u, v, |k, c, v| k.d0(c, v)),
                None => empty_grid(),
            })
        }
        /// Points and first derivatives (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&u, &v)?`.
        pub fn evaluate_grid_d1(
            &self,
            u: &Array1<'_, f64>,
            v: &Array1<'_, f64>,
        ) -> Result<Array2<'static, GeomGridEvalSurfD1>, Failure> {
            Ok(match self.terms(u, v)? {
                Some(k) => quadric_grid(&k, u, v, |k, c, v| k.d1(c, v)),
                None => empty_grid(),
            })
        }
        /// Points and derivatives up to order 2 (`EvaluateGridD2`);
        /// `evaluator.evaluate_grid_d2(&u, &v)?`.
        pub fn evaluate_grid_d2(
            &self,
            u: &Array1<'_, f64>,
            v: &Array1<'_, f64>,
        ) -> Result<Array2<'static, GeomGridEvalSurfD2>, Failure> {
            Ok(match self.terms(u, v)? {
                Some(k) => quadric_grid(&k, u, v, |k, c, v| k.d2(c, v)),
                None => empty_grid(),
            })
        }
        /// Points and derivatives up to order 3 (`EvaluateGridD3`);
        /// `evaluator.evaluate_grid_d3(&u, &v)?`.
        pub fn evaluate_grid_d3(
            &self,
            u: &Array1<'_, f64>,
            v: &Array1<'_, f64>,
        ) -> Result<Array2<'static, GeomGridEvalSurfD3>, Failure> {
            Ok(match self.terms(u, v)? {
                Some(k) => quadric_grid(&k, u, v, |k, c, v| k.d3(c, v)),
                None => empty_grid(),
            })
        }
        /// Derivative `(nu, nv)` at every `(u, v)`; empty for negative orders or
        /// `nu + nv < 1`, tested before the surface is read (`EvaluateGridDN`);
        /// `evaluator.evaluate_grid_dn(&u, &v, 1, 1)?`.
        pub fn evaluate_grid_dn(
            &self,
            u: &Array1<'_, f64>,
            v: &Array1<'_, f64>,
            nu: i32,
            nv: i32,
        ) -> Result<Array2<'static, Vector3>, Failure> {
            if invalid_orders(nu, nv) {
                return Ok(empty_grid());
            }
            Ok(match self.terms(u, v)? {
                Some(k) => quadric_grid(&k, u, v, |k, c, v| k.dn(c, v, nu, nv)),
                None => empty_grid(),
            })
        }
    };
}
pub(super) use quadric_grid_members;
