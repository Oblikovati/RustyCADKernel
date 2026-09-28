// Created on: 1995-07-18
// Created by: Modelistation
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
// Rust translation of the analytical part of Extrema_ExtCS::Perform at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The results of Extrema_ExtElCS: the parallel distance, or the extrema within
//! the bounds; when all of them are outside, the projections of the curve ends on
//! the elementary surface (the nearer one, or both when their distances are within
//! Confusion).
use super::ExtCS;
use crate::extrema::pon::{POnCurv, POnSurf};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::elslib::{self, ElementarySurface};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;

/// The projection of a curve end on the surface: `(U, V, point on surface)`.
type EndProjection = (f64, f64, Point3);

// `ElSLib::Parameters` then `ElSLib::Value` on one elementary surface.
fn project<S: ElementarySurface>(surface: &S, point: Point3) -> EndProjection {
    let (u, v) = elslib::parameters(surface, point);
    (u, v, elslib::value(u, v, surface))
}

impl ExtCS<'_> {
    /// The analytical results, when Extrema_ExtElCS is done.
    pub(super) fn treat_analytic(&mut self, curve: &dyn Adaptor3dCurve) -> Result<(), Failure> {
        self.done = true;
        self.is_par = self.ext_el_cs.is_parallel()?;
        if self.is_par {
            self.sq_dist.push(self.ext_el_cs.square_distance(1)?);
            return Ok(());
        }
        let nb_ext: i32 = self.ext_el_cs.nb_ext()?;
        for i in 1..=nb_ext {
            let (pc, ps): (POnCurv, POnSurf) = self.ext_el_cs.points(i)?;
            let (u, v) = ps.parameter();
            let dist: f64 = self.ext_el_cs.square_distance(i)?;
            self.add_solution(curve, (pc.parameter(), u, v), (*pc.value(), *ps.value()), dist)?;
        }
        if self.sq_dist.is_empty() && nb_ext > 0 {
            // The analytical extrema seem to be out of the curve or surface bounds:
            // try the ends of the curve.
            self.add_curve_ends(curve)?;
        }
        Ok(())
    }

    // The projection of the curve end at `t` on the elementary surface, None for
    // another surface.
    fn end_projection(&self, point: Point3) -> Result<Option<EndProjection>, Failure> {
        let surface = self.surface()?;
        Ok(match self.stype {
            SurfaceType::Plane => Some(project(&surface.plane()?, point)),
            SurfaceType::Sphere => Some(project(&surface.sphere()?, point)),
            SurfaceType::Cylinder => Some(project(&surface.cylinder()?, point)),
            SurfaceType::Torus => Some(project(&surface.torus()?, point)),
            SurfaceType::Cone => Some(project(&surface.cone()?, point)),
            _ => None,
        })
    }

    // The ends of the curve (finite ones) projected on the surface; the nearer is
    // added, or both when their square distances are within Confusion.
    fn add_curve_ends(&mut self, curve: &dyn Adaptor3dCurve) -> Result<(), Failure> {
        let at: [f64; 2] = [self.ucinf, self.ucsup];
        let mut ends: [Option<(Point3, EndProjection, f64)>; 2] = [None, None];
        for i in 0..2 {
            if precision::is_infinite(at[i]) {
                continue;
            }
            let p_on_c: Point3 = curve.value(at[i])?;
            let Some(projection) = self.end_projection(p_on_c)? else {
                continue;
            };
            ends[i] = Some((p_on_c, projection, p_on_c.square_distance(&projection.2)));
        }
        let dist = |end: &Option<(Point3, EndProjection, f64)>| end.map_or(-1., |e| e.2);
        let (d0, d1) = (dist(&ends[0]), dist(&ends[1]));
        let add: [bool; 2] = if d0 >= 0. && d1 >= 0. {
            let diff: f64 = d0 - d1;
            if diff.abs() < precision::confusion() {
                [true, true]
            } else if diff < 0. {
                [true, false]
            } else {
                [false, true]
            }
        } else {
            [d0 >= 0., !(d0 >= 0.) && d1 >= 0.]
        };
        for i in 0..2 {
            if let (true, Some((p_on_c, (u, v, p_on_s), d))) = (add[i], ends[i]) {
                self.add_solution(curve, (at[i], u, v), (p_on_c, p_on_s), d)?;
            }
        }
        Ok(())
    }
}
