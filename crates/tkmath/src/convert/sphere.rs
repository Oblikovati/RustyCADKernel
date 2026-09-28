// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/Convert/Convert_SphereToBSplineSurface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Spheres: a meridian arc of radius R swept around the axis.
use super::elementary::{ElementarySurfaceToBSplineSurface, SurfaceSizes};
use super::revolution::{arc_profile, periodic_knots, revolve, spans, trimmed_knots};
use crate::gp::{Point3, Sphere3};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

const SPHERE_SIZES: SurfaceSizes = SurfaceSizes {
    u_poles: 7,
    v_poles: 5,
    u_knots: 4,
    v_knots: 3,
    u_degree: 2,
    v_degree: 2,
};

// ComputePoles: the meridian of radius R on [v1, v2] swept over [u1, u2].
fn compute_poles(
    radius: f64,
    (u1, u2): (f64, f64),
    (v1, v2): (f64, f64),
    poles: &mut Array2<'_, Point3>,
) -> Result<(), Failure> {
    let profile = arc_profile(
        v1,
        v2,
        SPHERE_SIZES.v_poles as usize,
        |angle| (radius * angle.cos(), radius * angle.sin()),
        |angle, cosine| (radius * angle.cos() / cosine, radius * angle.sin() / cosine),
    )?;
    let nb_v_poles: i32 = spans(v2 - v1).0.wrapping_mul(2).wrapping_add(1);
    revolve(&profile, nb_v_poles, u1, u2, poles)
}

fn domain(message: String) -> Failure {
    Failure::with_type("Standard_DomainError", &message, "")
}

impl ElementarySurfaceToBSplineSurface {
    /// Sphere patch `[u1, u2] x [v1, v2]` (`Convert_SphereToBSplineSurface(Sph, U1, U2, V1, V2)`);
    /// requires `0 <= u2 - u1 <= 2 PI`, `v1 >= -PI / 2` and `v2 <= PI / 2`;
    /// `ElementarySurfaceToBSplineSurface::from_sphere_patch(&sphere, 0.0, PI, -1.0, 1.0)?`.
    pub fn from_sphere_patch(
        sphere: &Sphere3,
        u1: f64,
        u2: f64,
        v1: f64,
        v2: f64,
    ) -> Result<Self, Failure> {
        let (delta_u, delta_v) = (u2 - u1, v2 - v1);
        if (delta_u > 2.0 * PI) || (delta_u < 0.) || (v1 < -PI / 2.0) || (v2 > PI / 2.0) {
            return Err(domain(format!(
                "Convert_SphereToBSplineSurface: patch [{u1}, {u2}] x [{v1}, {v2}], expected \
                 0 <= U2 - U1 <= 2 PI and -PI/2 <= V1, V2 <= PI/2"
            )));
        }
        let mut result = Self::with_sizes(SPHERE_SIZES)?;
        let (nb_u_spans, alfa_u) = spans(delta_u);
        let (nb_v_spans, alfa_v) = spans(delta_v);
        result.set_counts(nb_u_spans, nb_v_spans);
        compute_poles(sphere.radius(), (u1, u2), (v1, v2), &mut result.poles)?;
        trimmed_knots(
            &mut result.u_knots,
            &mut result.u_mults,
            u1,
            result.nb_u_knots,
            alfa_u,
        )?;
        trimmed_knots(
            &mut result.v_knots,
            &mut result.v_mults,
            v1,
            result.nb_v_knots,
            alfa_v,
        )?;
        result.weigh_and_place(sphere.position(), (alfa_u.cos(), alfa_v.cos()))?;
        result.finalize()?;
        Ok(result)
    }

    /// Sphere band (`Convert_SphereToBSplineSurface(Sph, Param1, Param2, UTrim)`):
    /// trimmed in U on `[param1, param2]` over the whole meridian when `u_trim`,
    /// otherwise periodic in U and trimmed in V;
    /// `ElementarySurfaceToBSplineSurface::from_sphere_band(&sphere, 0.0, PI, true)?`.
    pub fn from_sphere_band(
        sphere: &Sphere3,
        param1: f64,
        param2: f64,
        u_trim: bool,
    ) -> Result<Self, Failure> {
        let delta: f64 = param2 - param1;
        if (delta > 2.0 * PI) || (delta < 0.) {
            return Err(domain(format!(
                "Convert_SphereToBSplineSurface: band [{param1}, {param2}], expected 0 <= Param2 - Param1 <= 2 PI"
            )));
        }
        let mut result = Self::with_sizes(SPHERE_SIZES)?;
        result.u_periodic = !u_trim;
        let cosines: (f64, f64) = if u_trim {
            result.sphere_lune(sphere.radius(), param1, param2)?
        } else {
            result.sphere_zone(sphere.radius(), param1, param2)?
        };
        result.weigh_and_place(sphere.position(), cosines)?;
        result.finalize()?;
        Ok(result)
    }

    /// Whole sphere, periodic in U (`Convert_SphereToBSplineSurface(Sph)`);
    /// `ElementarySurfaceToBSplineSurface::from_sphere(&sphere)?.nb_v_poles() == 5`.
    pub fn from_sphere(sphere: &Sphere3) -> Result<Self, Failure> {
        let mut result = Self::with_sizes(SPHERE_SIZES)?;
        result.u_periodic = true;
        (result.nb_u_poles, result.nb_v_poles) = (6, 5);
        (result.nb_u_knots, result.nb_v_knots) = (4, 3);
        compute_poles(
            sphere.radius(),
            (0., 2. * PI),
            (-PI / 2., PI / 2.),
            &mut result.poles,
        )?;
        let u_knots = [0., 2. * PI / 3., 4. * PI / 3., 2. * PI];
        for (i, knot) in (1..=4).zip(u_knots) {
            (result.u_knots[i], result.u_mults[i]) = (knot, 2);
        }
        result.set_pole_meridian_knots();
        result.weigh_and_place(sphere.position(), (0.5, 2.0_f64.sqrt() / 2.))?;
        result.finalize()?;
        Ok(result)
    }

    // The zone between two latitudes, periodic in U; the V weights take cos(AlfaV).
    fn sphere_zone(
        &mut self,
        radius: f64,
        param1: f64,
        param2: f64,
    ) -> Result<(f64, f64), Failure> {
        compute_poles(radius, (0., 2. * PI), (param1, param2), &mut self.poles)?;
        (self.nb_u_poles, self.nb_u_knots) = (6, 4);
        let (nb_v_spans, alfa_v) = spans(param2 - param1);
        self.nb_v_poles = nb_v_spans.wrapping_mul(2).wrapping_add(1);
        self.nb_v_knots = nb_v_spans.wrapping_add(1);
        periodic_knots(&mut self.u_knots, &mut self.u_mults)?;
        trimmed_knots(
            &mut self.v_knots,
            &mut self.v_mults,
            param1,
            self.nb_v_knots,
            alfa_v,
        )?;
        Ok((0.5, alfa_v.cos()))
    }

    // The lune between two meridians over the whole latitude range; the V weights
    // take 0.5 although the two V spans open by PI / 4 (native quirk).
    fn sphere_lune(
        &mut self,
        radius: f64,
        param1: f64,
        param2: f64,
    ) -> Result<(f64, f64), Failure> {
        compute_poles(
            radius,
            (param1, param2),
            (-PI / 2., PI / 2.),
            &mut self.poles,
        )?;
        (self.nb_v_poles, self.nb_v_knots) = (5, 3);
        let (nb_u_spans, alfa_u) = spans(param2 - param1);
        self.nb_u_poles = nb_u_spans.wrapping_mul(2).wrapping_add(1);
        self.nb_u_knots = nb_u_spans.wrapping_add(1);
        self.set_pole_meridian_knots();
        trimmed_knots(
            &mut self.u_knots,
            &mut self.u_mults,
            param1,
            self.nb_u_knots,
            alfa_u,
        )?;
        Ok((alfa_u.cos(), 0.5))
    }

    // V knots -PI/2, 0, PI/2 with multiplicities 3, 2, 3.
    fn set_pole_meridian_knots(&mut self) {
        (self.v_knots[1], self.v_mults[1]) = (-PI / 2., 3);
        (self.v_knots[2], self.v_mults[2]) = (0., 2);
        (self.v_knots[3], self.v_mults[3]) = (PI / 2., 3);
    }
}
