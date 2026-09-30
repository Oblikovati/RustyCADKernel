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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_PrmPrmSvSurfaces.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ApproxInt_PrmPrmSvSurfaces.gxx`: the SvSurfaces of two parametric surfaces, which
//! refine a point of the line with the IntImp `Int2S` solver (docs/geomalgo-design.md
//! section 20).
use super::sv_cache::{SvCache, sv_stored_members};
use super::sv_surfaces::{ApproxIntSvComputed, ApproxIntSvSurfaces, ApproxIntSvSurfacesBase};
use crate::int_imp::{IntImpInt2S, ThePSurfaceTool};
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::GeomSurfaceD1;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::FunctionSetRoot;

/// `TOLTANGENCY`: the tangency tolerance of the `Int2S` solver.
const TOLTANGENCY: f64 = 0.0000000001;

/// `ApproxInt_PrmPrmSvSurfaces` over the surface tool `T`
/// (GeomInt_ThePrmPrmSvSurfacesOfWLApprox over Adaptor3d_HSurfaceTool): the point of the
/// intersection line of two parametric surfaces near given parameters, and its
/// tangents, with the two last results kept.
///
/// ```ignore
/// let mut sv = GeomIntThePrmPrmSvSurfacesOfWLApprox::new(&s1, &s2)?;
/// let (flag, tangent) = sv.tangency([0.5, 0.5, 0.5, 0.5])?;
/// ```
pub struct ApproxIntPrmPrmSvSurfaces<T: ThePSurfaceTool> {
    base: ApproxIntSvSurfacesBase,
    cache: SvCache,
    my_intersection_on2s: IntImpInt2S<T>,
}

impl<T: ThePSurfaceTool> ApproxIntPrmPrmSvSurfaces<T> {
    /// `ApproxInt_PrmPrmSvSurfaces(Surf1, Surf2)`: the solver on the two surfaces with
    /// the tangency tolerance 1e-10; `ApproxIntPrmPrmSvSurfaces::<T>::new(&s1, &s2)?`.
    pub fn new(surf1: &T::Surface, surf2: &T::Surface) -> Result<Self, Failure> {
        Ok(Self {
            base: ApproxIntSvSurfacesBase::default(),
            cache: SvCache::default(),
            my_intersection_on2s: IntImpInt2S::new(surf1, surf2, TOLTANGENCY)?,
        })
    }

    /// The solver run of `Compute` from `uv`: the tangent flag of the solver when a
    /// point is found (stored in the solver), else the failure exit (a solver not done
    /// keeps `MyIsTangent`, an empty result clears it).
    fn solve(&mut self, uv: [f64; 4]) -> Result<Result<bool, bool>, Failure> {
        let mut param = one_based::<f64>(4)?;
        for (k, value) in uv.iter().enumerate() {
            param[k as i32 + 1] = *value;
        }
        let inter = &mut self.my_intersection_on2s;
        let mut rsnld = FunctionSetRoot::without_tolerance(inter.function(), 100)?;
        inter.perform(&param, &mut rsnld)?;
        if !inter.is_done() {
            self.cache.computed = false;
            self.cache.computed_bis = false;
            return Ok(Err(false));
        }
        if inter.is_empty()? {
            return Ok(Err(self.cache.fail()));
        }
        Ok(Ok(inter.is_tangent()?))
    }

    /// The tangent in the parameter plane of a surface of first derivatives `d`:
    /// `Tg = DeltaU * TU + DeltaV * TV` solved by the normal equations; None when their
    /// determinant is below `Precision::Angular()`.
    fn plane_tangent(tg: Vector3, d: &GeomSurfaceD1) -> Option<Vector2> {
        let (tu, tv) = (d.d1u, d.d1v);
        let tutu: f64 = tu.dot(tu);
        let tvtv: f64 = tv.dot(tv);
        let tutv: f64 = tu.dot(tv);
        let tgtu: f64 = tg.dot(tu);
        let tgtv: f64 = tg.dot(tv);
        let dis: f64 = tutu * tvtv - tutv * tutv;
        if dis.abs() < precision::angular() {
            return None;
        }
        let delta_u: f64 = (tgtu * tvtv - tgtv * tutv) / dis;
        let delta_v: f64 = (tgtv * tutu - tgtu * tutv) / dis;
        Some(Vector2::new(delta_u, delta_v))
    }

    /// The tangents of a point found by the solver, after the point and the
    /// parameters are stored.
    fn tangents(&mut self, uv: &[f64; 4], out: &mut ApproxIntSvComputed) -> Result<bool, Failure> {
        let inter = &mut self.my_intersection_on2s;
        let surf1 = inter.function().auxillar_surface1().clone();
        let surf2 = inter.function().auxillar_surface2().clone();
        let direction: Vector3 = Vector3::from(inter.direction()?);
        out.tangent = direction;
        self.cache.cur.tg = direction;
        out.tangent_uv1 = Vector2::from(inter.direction_on_s1()?);
        self.cache.cur.tguv1 = out.tangent_uv1;
        out.tangent_uv2 = Vector2::from(inter.direction_on_s2()?);
        self.cache.cur.tguv2 = out.tangent_uv2;
        out.tangent.normalize()?;
        self.cache.cur.tg = out.tangent;
        let d1: GeomSurfaceD1 = T::d1(&surf1, uv[0], uv[1])?;
        let Some(tguv1) = Self::plane_tangent(out.tangent, &d1) else {
            return Ok(self.cache.fail());
        };
        out.tangent_uv1 = tguv1;
        self.cache.cur.tguv1 = tguv1;
        let d2: GeomSurfaceD1 = T::d1(&surf2, uv[2], uv[3])?;
        let Some(tguv2) = Self::plane_tangent(out.tangent, &d2) else {
            return Ok(self.cache.fail());
        };
        out.tangent_uv2 = tguv2;
        self.cache.cur.tguv2 = tguv2;
        Ok(true)
    }
}

impl<T: ThePSurfaceTool> ApproxIntSvSurfaces for ApproxIntPrmPrmSvSurfaces<T> {
    fn sv_base(&self) -> &ApproxIntSvSurfacesBase {
        &self.base
    }
    fn sv_base_mut(&mut self) -> &mut ApproxIntSvSurfacesBase {
        &mut self.base
    }

    /// `Compute`: the stored result at these parameters, else the solver from them;
    /// `uv` gets the parameters of the point found and `out` the point and tangents.
    fn compute(
        &mut self,
        uv: &mut [f64; 4],
        out: &mut ApproxIntSvComputed,
    ) -> Result<bool, Failure> {
        let t: [f64; 4] = *uv;
        if let Some(flag) = self.cache.lookup(t) {
            return Ok(flag);
        }
        self.cache.cur.is_tangent = true;
        let tangent: bool = match self.solve(t)? {
            Ok(tangent) => tangent,
            Err(flag) => return Ok(flag),
        };
        self.cache.computed = true;
        let point = self.my_intersection_on2s.point()?;
        out.point = point.value();
        self.cache.cur.pnt = out.point;
        let (u1, v1, u2, v2) = point.parameters();
        *uv = [u1, v1, u2, v2];
        self.cache.cur.par_on_s1 = Point2::new(t[0], t[1]);
        self.cache.cur.par_on_s2 = Point2::new(t[2], t[3]);
        if tangent {
            return Ok(self.cache.fail());
        }
        self.tangents(uv, out)
    }

    sv_stored_members!();

    /// `SeekPoint`: the point of `Compute` with the parameters it found.
    fn seek_point(&mut self, uv: [f64; 4]) -> Result<Option<IntSurfPntOn2S>, Failure> {
        let mut t: [f64; 4] = uv;
        let mut out = ApproxIntSvComputed::default();
        if !self.compute(&mut t, &mut out)? {
            return Ok(None);
        }
        let mut point = IntSurfPntOn2S::new();
        point.set_value_all(out.point, t[0], t[1], t[2], t[3]);
        Ok(Some(point))
    }
}
