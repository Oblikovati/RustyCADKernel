// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ApproxInt_ImpPrmSvSurfaces::Compute of
// ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_ImpPrmSvSurfaces.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Compute`: the point of the line from the solver, the middle of its images on both
//! surfaces, and the 3d and 2d tangents from the normals (or from the derivatives at a
//! singular point).
use super::ApproxIntImpPrmSvSurfaces;
use super::initial::PatchBounds;
use super::singular::{Singular, is_singular, non_singular};
use crate::approx_int::surface_tool::ApproxIntSurfaceTool;
use crate::approx_int::sv_surfaces::{ApproxIntSvComputed, ApproxIntSvSurfaces};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomSurfaceD1;
use rustycad_tkmath::gp::{Point2, Vector2, Vector3};
use rustycad_tkmath::math::Vector;

/// The derivatives of both surfaces at the point (`aD1uPrm`, `aD1vPrm`, `aD1uQuad`,
/// `aD1vQuad`) and the tolerances (`aNullValue`, the square of
/// `Precision::Approximation()`, and `anAngTol`).
struct Derivatives {
    prm: (Vector3, Vector3),
    quad: (Vector3, Vector3),
    null_value: f64,
    ang_tol: f64,
}

impl Derivatives {
    fn singular(&self, (du, dv): (Vector3, Vector3), compute_3d: bool) -> Singular {
        Singular {
            du,
            dv,
            is_to_3d_tg_compute: compute_3d,
            lin_tol: self.null_value,
            ang_tol: self.ang_tol,
        }
    }
}

impl<T: ApproxIntSurfaceTool> ApproxIntImpPrmSvSurfaces<T> {
    /// The body of `Compute` after the cache lookup.
    pub(super) fn compute_point(
        &mut self,
        uv: &mut [f64; 4],
        out: &mut ApproxIntSvComputed,
    ) -> Result<bool, Failure> {
        let t: [f64; 4] = *uv;
        if let Some(flag) = self.cache.lookup(t) {
            return Ok(flag);
        }
        let b: PatchBounds = self.bounds()?;
        let mut x = Vector::filled(1, 2, 0.0)?;
        let Some((tu, tv)) = self.fill_initial_vector_of_solution(t, b, &mut x)? else {
            self.cache.bis.is_tangent = false;
            return Ok(self.cache.fail());
        };
        let (pour_tester_u, pour_tester_v) = (x[1], x[2]);
        let use_solver: bool = self.get_use_solver();
        let rsnld_is_done: bool = use_solver && self.solve(&mut x, b)?;
        if !(rsnld_is_done || !use_solver) {
            return Ok(self.cache.fail());
        }
        self.cache.computed = true;
        let dist_u: f64 = (pour_tester_u - x[1]).abs();
        let dist_v: f64 = (pour_tester_v - x[2]).abs();
        out.point = T::value(self.p_surface()?, x[1], x[2])?;
        self.cache.cur.pnt = out.point;
        if !(dist_v <= 0.001 && dist_u <= 0.001) {
            return Ok(self.cache.fail());
        }
        let x12: (f64, f64) = (x[1], x[2]);
        let d = self.point_and_derivatives(uv, t, (x12, (x[1] - tu, x[2] - tv)), out)?;
        self.tangents(&d, out)
    }

    /// The parameters of the point on the parametric surface, the stored parameters
    /// (the second one keeps `(tu2, tu2)` when the quadric is the second surface), the
    /// derivatives of both surfaces at the solver point `X` and at the input quadric
    /// parameters, and the middle of the two images of the point.
    fn point_and_derivatives(
        &mut self,
        uv: &mut [f64; 4],
        t: [f64; 4],
        ((x1, x2), prm): ((f64, f64), (f64, f64)),
        out: &mut ApproxIntSvComputed,
    ) -> Result<Derivatives, Failure> {
        let q: IntSurfQuadric = self.i_surface()?;
        // The native loops that bring the quadric U within Pi of its input compare it
        // with itself and do nothing.
        let (quad_u, quad_v) = if self.my_implicit_first {
            (uv[2], uv[3]) = prm;
            self.cache.cur.par_on_s1 = Point2::new(t[0], t[1]);
            self.cache.cur.par_on_s2 = Point2::new(t[2], t[3]);
            (uv[0], uv[1])
        } else {
            (uv[0], uv[1]) = prm;
            self.cache.cur.par_on_s1 = Point2::new(t[0], t[1]);
            self.cache.cur.par_on_s2 = Point2::new(t[2], t[2]);
            (uv[2], uv[3])
        };
        let d1: GeomSurfaceD1 = T::d1(self.p_surface()?, x1, x2)?;
        out.point = d1.point;
        // An undefined quadric leaves the outputs unchanged natively; every instance
        // sets its quadric.
        let (p2, du_quad, dv_quad) = q.d1(quad_u, quad_v).unwrap_or_default();
        // Middle point of the P-P2 segment.
        out.point.barycenter(1.0, p2, 1.0);
        self.cache.cur.pnt = out.point;
        Ok(Derivatives {
            prm: (d1.d1u, d1.d1v),
            quad: (du_quad, dv_quad),
            null_value: precision::approximation() * precision::approximation(),
            ang_tol: precision::angular(),
        })
    }

    /// The 3d tangent and the 2d tangents of `out` (the quadric's is `Tguv1` when the
    /// quadric is the first surface): from the normals at a regular point, from the
    /// derivatives at a singular one.
    fn tangents(
        &mut self,
        d: &Derivatives,
        out: &mut ApproxIntSvComputed,
    ) -> Result<bool, Failure> {
        let q: IntSurfQuadric = self.i_surface()?;
        let mut normal_prm: Vector3 = d.prm.0.crossed(d.prm.1);
        let mut normal_imp: Vector3 = q.normale_at(self.cache.cur.pnt)?;
        let sq_magn_prm: f64 = normal_prm.square_magnitude();
        let sq_magn_imp: f64 = normal_imp.square_magnitude();
        let ApproxIntSvComputed {
            tangent,
            tangent_uv1,
            tangent_uv2,
            ..
        } = out;
        let (quad_tg, prm_tg) = if self.my_implicit_first {
            (tangent_uv1, tangent_uv2)
        } else {
            (tangent_uv2, tangent_uv1)
        };
        let is_prm_singular: bool = is_singular(d.prm.0, d.prm.1, d.null_value, d.ang_tol);
        if is_prm_singular {
            if !d.singular(d.prm, true).process(tangent, prm_tg) {
                return Ok(self.cache.fail());
            }
            self.cache.cur.tg = *tangent;
        } else {
            normal_prm /= sq_magn_prm.sqrt();
        }
        // Analogically for the implicit surface.
        let is_imp_singular: bool = sq_magn_imp < d.null_value;
        if is_imp_singular {
            if !d
                .singular(d.quad, !is_prm_singular)
                .process(tangent, quad_tg)
            {
                return Ok(self.cache.fail());
            }
            self.cache.cur.tg = *tangent;
        } else {
            normal_imp /= sq_magn_imp.sqrt();
        }
        if !(is_imp_singular && is_prm_singular) {
            if !(is_imp_singular || is_prm_singular) {
                // Pure non-singular case: the normals are normalized, so Tg is null
                // only when they are parallel.
                *tangent = normal_imp.crossed(normal_prm);
            }
            let singular = (is_prm_singular, is_imp_singular);
            if !self.regular_tangents(d, singular, tangent, (quad_tg, prm_tg)) {
                return Ok(self.cache.fail());
            }
        }
        self.cache.cur.tguv1 = out.tangent_uv1;
        self.cache.cur.tguv2 = out.tangent_uv2;
        self.cache.cur.is_tangent = true;
        Ok(true)
    }

    /// The normalization of the 3d tangent and the 2d tangents of the regular
    /// surfaces; false for a null tangent or an abnormal singular case.
    fn regular_tangents(
        &mut self,
        d: &Derivatives,
        (is_prm_singular, is_imp_singular): (bool, bool),
        tangent: &mut Vector3,
        (quad_tg, prm_tg): (&mut Vector2, &mut Vector2),
    ) -> bool {
        let sq_magn_tg: f64 = tangent.square_magnitude();
        if sq_magn_tg < d.null_value {
            return false;
        }
        *tangent /= sq_magn_tg.sqrt();
        self.cache.cur.tg = *tangent;
        let tols: (f64, f64) = (d.null_value, d.ang_tol);
        // A singular surface has its 2d tangent from the singular processing.
        if !is_prm_singular && !non_singular(d.prm, *tangent, tols, prm_tg) {
            return false;
        }
        is_imp_singular || non_singular(d.quad, *tangent, tols, quad_tg)
    }
}
