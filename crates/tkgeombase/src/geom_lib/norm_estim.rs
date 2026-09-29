// Created on: 1993-07-07
// Created by: Jean Claude VAUTHIER
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (NormEstim)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::NormEstim`: the surface normal at a point, recovered near singular
//! points from the second derivatives.
use super::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomSurface, GeomSurfaceD1};
use rustycad_tkmath::cslib::{NormalStatus, normal_from_second_derivatives};
use rustycad_tkmath::gp::{Direction3, Point2, Vector3};

// Normal recovery near a singular surface point.
const THE_NORMAL_PARAMETER_STEP: f64 = 1.0e-5;
const THE_NORMAL_SQUARED_MAGNITUDE_TOL: f64 = 1.0e-16;
const THE_CONE_NORMAL_ANGULAR_TOLERANCE: f64 = 1.0e-4;

/// `GeomLib::NormEstim(theSurf, theUV, theTol, theNorm)`: 0 when the normal comes
/// from the first derivatives, 1 when it is estimated from the second ones
/// (quasi-singular point), 2 or more when it is undefined (2: a null D1N or a cone
/// apex or infinitely many solutions, 3: otherwise).
///
/// `norm` is written only for the results 0 and 1, as natively. Near a singular
/// point the CSLib normal is oriented like the normal a step of 1e-5 away along the
/// non-degenerate parameter (or diagonally when that one is null too);
/// `let code = geom_lib::norm_estim(&surface, Point2::new(0.4, 1.0), 1e-9, &mut normal)?;`.
pub fn norm_estim(
    surf: &Handle<dyn GeomSurface>,
    uv: Point2,
    tol: f64,
    norm: &mut Direction3,
) -> Result<i32, Failure> {
    let s: &dyn GeomSurface = live(surf, "GeomLib::NormEstim")?;
    let tol2: f64 = tol * tol;
    let first: GeomSurfaceD1 = s.eval_d1(uv.x(), uv.y())?;
    let (mdu, mdv): (f64, f64) = (first.d1u.square_magnitude(), first.d1v.square_magnitude());
    if mdu >= tol2 && mdv >= tol2 {
        let a_norm: Vector3 = first.d1u.crossed(first.d1v);
        if a_norm.square_magnitude() < tol2 {
            return Ok(3);
        }
        norm.set_xyz(*a_norm.xyz())?;
        return Ok(0);
    }
    let second = s.eval_d2(uv.x(), uv.y())?;
    let result = normal_from_second_derivatives(
        second.d1u,
        second.d1v,
        second.d2u,
        second.d2v,
        second.d2uv,
        tol,
    );
    let Some(mut a_normal) = result.normal else {
        // Computation is impossible.
        return Ok(if result.status == NormalStatus::D1NIsNull {
            2
        } else {
            3
        });
    };
    let probe = SingularProbe {
        s,
        uv,
        bounds: s.bounds(),
    };
    if probe.is_cone_apex()? {
        return Ok(2);
    }
    probe.orient(
        &mut a_normal,
        mdu < tol2 && mdv >= tol2,
        mdv < tol2 && mdu >= tol2,
    )?;
    // Quasi-singular.
    let quasi: bool = matches!(
        result.status,
        NormalStatus::D1NuIsNull | NormalStatus::D1NvIsNull | NormalStatus::D1NuIsParallelD1Nv
    );
    if quasi {
        norm.set_xyz(a_normal.xyz())?;
        return Ok(1);
    }
    Ok(if result.status == NormalStatus::InfinityOfSolutions {
        2
    } else {
        3
    })
}

// The neighbourhood of a singular parameter: the surface, the point and its bounds.
struct SingularProbe<'s> {
    s: &'s dyn GeomSurface,
    uv: Point2,
    bounds: (f64, f64, f64, f64),
}

impl SingularProbe<'_> {
    // Check for a cone apex singularity point: the normals a step before and after
    // along V are opposite.
    fn is_cone_apex(&self) -> Result<bool, Failure> {
        let (_, _, vmin, vmax) = self.bounds;
        let (u, v): (f64, f64) = (self.uv.x(), self.uv.y());
        if !(v > vmin + THE_NORMAL_PARAMETER_STEP && v < vmax - THE_NORMAL_PARAMETER_STEP) {
            return Ok(false);
        }
        let sign: f64 = -1.0;
        let before: GeomSurfaceD1 = self.s.eval_d1(u, v - sign * THE_NORMAL_PARAMETER_STEP)?;
        if !regular(&before) {
            return Ok(false);
        }
        let normal1 = Direction3::from_vector(before.d1u.crossed(before.d1v))?;
        let after: GeomSurfaceD1 = self.s.eval_d1(u, v + sign * THE_NORMAL_PARAMETER_STEP)?;
        if !regular(&after) {
            return Ok(false);
        }
        let normal2 = Direction3::from_vector(after.d1u.crossed(after.d1v))?;
        Ok(normal1.is_opposite(normal2, THE_CONE_NORMAL_ANGULAR_TOLERANCE))
    }

    // Along V (only D1U null) then along U (only D1V null): the normal is reversed
    // when the normal a step away, towards the farther bound, points the other way.
    fn orient(&self, normal: &mut Direction3, along_v: bool, along_u: bool) -> Result<(), Failure> {
        let (umin, umax, vmin, vmax) = self.bounds;
        let (u, v): (f64, f64) = (self.uv.x(), self.uv.y());
        let toward = |value: f64, low: f64, high: f64| {
            if (high - value) > (value - low) {
                1.0
            } else {
                -1.0
            }
        };
        let mut sign: f64 = -1.0;
        if along_v {
            sign = toward(v, vmin, vmax);
            let norm = self.shifted_normal((u, v + sign * THE_NORMAL_PARAMETER_STEP), || {
                (
                    u + toward(u, umin, umax) * THE_NORMAL_PARAMETER_STEP,
                    v + sign * THE_NORMAL_PARAMETER_STEP,
                )
            })?;
            reverse_if_opposite(normal, norm);
        }
        if along_u {
            // The sign keeps the value the V case gave it, as natively.
            if (umax - u) > (u - umin) {
                sign = 1.0;
            }
            let norm = self.shifted_normal((u + sign * THE_NORMAL_PARAMETER_STEP, v), || {
                (
                    u + sign * THE_NORMAL_PARAMETER_STEP,
                    v + toward(v, vmin, vmax) * THE_NORMAL_PARAMETER_STEP,
                )
            })?;
            reverse_if_opposite(normal, norm);
        }
        Ok(())
    }

    // D1U ^ D1V at `shifted`, or at `diagonal()` when that one is null.
    fn shifted_normal(
        &self,
        shifted: (f64, f64),
        diagonal: impl Fn() -> (f64, f64),
    ) -> Result<Vector3, Failure> {
        let at = |(pu, pv): (f64, f64)| -> Result<Vector3, Failure> {
            let d1: GeomSurfaceD1 = self.s.eval_d1(pu, pv)?;
            Ok(d1.d1u.crossed(d1.d1v))
        };
        let norm: Vector3 = at(shifted)?;
        if norm.square_magnitude() < THE_NORMAL_SQUARED_MAGNITUDE_TOL {
            return at(diagonal());
        }
        Ok(norm)
    }
}

fn regular(d1: &GeomSurfaceD1) -> bool {
    d1.d1u.square_magnitude() > THE_NORMAL_SQUARED_MAGNITUDE_TOL
        && d1.d1v.square_magnitude() > THE_NORMAL_SQUARED_MAGNITUDE_TOL
}

fn reverse_if_opposite(normal: &mut Direction3, norm: Vector3) {
    if norm.square_magnitude() >= THE_NORMAL_SQUARED_MAGNITUDE_TOL
        && norm.dot(Vector3::from(*normal)) < 0.0
    {
        normal.reverse();
    }
}
