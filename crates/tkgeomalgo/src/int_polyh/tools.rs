// Created on: 2017-01-11
// Copyright (c) 2017 OPEN CASCADE SAS
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_Tools.hxx and
// IntPolyh_Tools.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPolyh_Tools`: the sampling of a surface for its triangulation.
use super::array::{IntPolyhArrayOfPointNormal, IntPolyhPointNormal};
use crate::int_curve_surface::ThePolyhedronOfHInter;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{self, Vector3};

/// `IntPolyh_Tools`: static members.
///
/// ```ignore
/// let (u_enlarge, v_enlarge) = IntPolyhTools::is_enlarge_possible(&surface)?;
/// ```
pub struct IntPolyhTools;

impl IntPolyhTools {
    /// `IsEnlargePossible(theSurf, theUEnlarge, theVEnlarge)`: in the context of
    /// IntPolyh_Intersection only B-spline and Bezier surfaces that are neither closed,
    /// periodic nor infinite in a direction may be enlarged in it.
    /// `IntPolyhTools::is_enlarge_possible(&s)?`.
    pub fn is_enlarge_possible(surf: &Adaptor3dSurfaceHandle) -> Result<(bool, bool), Failure> {
        let kind: SurfaceType = surf.get_type()?;
        if kind != SurfaceType::BSplineSurface && kind != SurfaceType::BezierSurface {
            return Ok((false, false));
        }
        let mut u_enlarge: bool = false;
        if !surf.is_u_closed()? && !surf.is_u_periodic()? {
            let (u0, u1) = (surf.first_u_parameter()?, surf.last_u_parameter()?);
            u_enlarge = !precision::is_infinite(u0) && !precision::is_infinite(u1);
        }
        let mut v_enlarge: bool = false;
        if !surf.is_v_closed()? && !surf.is_v_periodic()? {
            let (v0, v1) = (surf.first_v_parameter()?, surf.last_v_parameter()?);
            v_enlarge = !precision::is_infinite(v0) && !precision::is_infinite(v1);
        }
        Ok((u_enlarge, v_enlarge))
    }

    /// `MakeSampling(theSurf, theNbSU, theNbSV, theEnlargeZone, theUPars, theVPars)`: the
    /// `NbSU` x `NbSV` uniform parameters of the surface domain (enlarged by 1% on each
    /// side where possible when `enlarge_zone`), the last one exactly the bound.
    /// `IntPolyhTools::make_sampling(&s, 10, 10, false, &mut u, &mut v)?`.
    pub fn make_sampling(
        surf: &Adaptor3dSurfaceHandle,
        (nb_su, nb_sv): (i32, i32),
        enlarge_zone: bool,
        u_pars: &mut Array1<'static, f64>,
        v_pars: &mut Array1<'static, f64>,
    ) -> Result<(), Failure> {
        u_pars
            .resize_bounds(1, nb_su, false)
            .map_err(|e| e.into_failure("NCollection_Array1::Resize"))?;
        v_pars
            .resize_bounds(1, nb_sv, false)
            .map_err(|e| e.into_failure("NCollection_Array1::Resize"))?;
        let mut u = [surf.first_u_parameter()?, surf.last_u_parameter()?];
        let mut v = [surf.first_v_parameter()?, surf.last_v_parameter()?];
        if enlarge_zone {
            enlarge_zone_of(surf, &mut u, &mut v)?;
        }
        fill_uniform(u_pars, nb_su, u);
        fill_uniform(v_pars, nb_sv, v);
        Ok(())
    }

    /// `ComputeDeflection(theSurf, theUPars, theVPars)`: the deflection over-estimation of
    /// the polyhedron of the sampling. `IntPolyhTools::compute_deflection(&s, &u, &v)?`.
    pub fn compute_deflection(
        surf: &Adaptor3dSurfaceHandle,
        u_pars: &Array1<'_, f64>,
        v_pars: &Array1<'_, f64>,
    ) -> Result<f64, Failure> {
        let polyhedron = ThePolyhedronOfHInter::with_parameters(surf, u_pars, v_pars)?;
        Ok(polyhedron.deflection_over_estimation())
    }

    /// `FillArrayOfPointNormal(theSurf, theUPars, theVPars, thePoints)`: the surface
    /// points of the sampling (V inner) with their unit normals, null where the
    /// derivatives are parallel. `IntPolyhTools::fill_array_of_point_normal(&s, &u, &v, &mut a)?`.
    pub fn fill_array_of_point_normal(
        surf: &Adaptor3dSurfaceHandle,
        u_pars: &Array1<'_, f64>,
        v_pars: &Array1<'_, f64>,
        points: &mut IntPolyhArrayOfPointNormal,
    ) -> Result<(), Failure> {
        let (nb_u, nb_v) = (u_pars.length(), v_pars.length());
        let mut i_cnt: i32 = 0;
        points.init(nb_u * nb_v);
        for i in 1..=nb_u {
            let a_u: f64 = array_value(u_pars, i)?;
            for j in 1..=nb_v {
                let a_v: f64 = array_value(v_pars, j)?;
                points[i_cnt] = point_normal(surf, a_u, a_v)?;
                i_cnt += 1;
            }
        }
        points.set_nb_items(i_cnt);
        Ok(())
    }
}

/// `theUPars(i)`: the item of index `i` (not `i` from the lower bound): Standard_OutOfRange
/// outside the bounds. `array_value(&upars, 1)?`.
pub(crate) fn array_value(pars: &Array1<'_, f64>, i: i32) -> Result<f64, Failure> {
    pars.value(i)
        .copied()
        .map_err(|e| e.into_failure("NCollection_Array1::Value"))
}

/// The point and unit normal of the surface at `(u, v)`.
fn point_normal(
    surf: &Adaptor3dSurfaceHandle,
    u: f64,
    v: f64,
) -> Result<IntPolyhPointNormal, Failure> {
    let d1 = surf.d1(u, v)?;
    let mut v_norm: Vector3 = d1.d1u.crossed(d1.d1v);
    let length: f64 = v_norm.magnitude();
    if length > gp::resolution() {
        v_norm /= length;
    } else {
        v_norm = Vector3::new(0.0, 0.0, 0.0);
    }
    Ok(IntPolyhPointNormal {
        point: d1.point,
        normal: v_norm,
    })
}

/// `EnlargeZone`: widens the directions that may be enlarged by 1% on each side.
fn enlarge_zone_of(
    surf: &Adaptor3dSurfaceHandle,
    u: &mut [f64; 2],
    v: &mut [f64; 2],
) -> Result<(), Failure> {
    let (to_enlarge_u, to_enlarge_v) = IntPolyhTools::is_enlarge_possible(surf)?;
    if to_enlarge_u {
        let delta_u: f64 = 0.01 * (u[1] - u[0]).abs();
        u[0] -= delta_u;
        u[1] += delta_u;
    }
    if to_enlarge_v {
        let delta_v: f64 = 0.01 * (v[1] - v[0]).abs();
        v[0] -= delta_v;
        v[1] += delta_v;
    }
    Ok(())
}

/// The `nb` uniform parameters of `[x0, x1]` from index 1, the last one exactly `x1`.
fn fill_uniform(pars: &mut Array1<'static, f64>, nb: i32, [x0, x1]: [f64; 2]) {
    let nb_samples1: i32 = nb - 1;
    let dx: f64 = (x1 - x0) / f64::from(nb_samples1);
    for i in 0..nb {
        let mut a_x: f64 = x0 + f64::from(i) * dx;
        if i == nb_samples1 {
            a_x = x1;
        }
        pars[i + 1] = a_x;
    }
}
