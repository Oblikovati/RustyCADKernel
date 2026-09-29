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
// Rust translation of Extrema_ExtPElS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtPElS`: the extrema of the distance between a point and an elementary
//! surface (plane, cylinder, cone, sphere, torus), in closed form.
use super::failures::{extremum_index, not_done};
use super::pon::POnSurf;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real;
use rustycad_tkmath::elslib::{self, ElementarySurface};
use rustycad_tkmath::gp::{
    Axis3, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};
use std::f64::consts::PI;

/// `Extrema_ExtPElS`: at most four extrema;
/// `ExtPElS::with_sphere(point, sphere, 1e-7)?.nb_ext()? == 2`.
#[derive(Clone, Debug)]
pub struct ExtPElS {
    done: bool,
    nb_ext: i32,
    sq_dist: [f64; 4],
    points: [POnSurf; 4],
}

impl Default for ExtPElS {
    fn default() -> Self {
        Self {
            done: false,
            nb_ext: 0,
            sq_dist: [f64::MAX; 4],
            points: [POnSurf::default(); 4],
        }
    }
}

// ExtPElS_MyEps: an angle within Epsilon(2 PI) of zero is zero.
fn snap_zero(angle: f64) -> f64 {
    let eps: f64 = real::epsilon(2. * PI);
    if angle > -eps && angle < eps {
        0.
    } else {
        angle
    }
}

// The angle of `opp` from the X direction of `position`, about `X ^ Y`, in (-PI, PI].
fn frame_angle(position: &Axis3, opp: Vector3) -> Result<f64, Failure> {
    let my_z: Direction3 = position.x_direction().crossed(position.y_direction())?;
    let x_dir: Vector3 = Vector3::from(position.x_direction());
    Ok(snap_zero(x_dir.angle_with_ref(opp, Vector3::from(my_z))?))
}

impl ExtPElS {
    /// `Extrema_ExtPElS()`: not done; `ExtPElS::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    // Starts a Perform: not done, no extremum.
    fn reset(&mut self) {
        self.done = false;
        self.nb_ext = 0;
    }
    // Appends the surface point at `(u, v)` and its square distance to `point`.
    fn push<S: ElementarySurface>(&mut self, point: Point3, surface: &S, (u, v): (f64, f64)) {
        let ps: Point3 = elslib::value(u, v, surface);
        let index: usize = self.nb_ext as usize;
        self.sq_dist[index] = ps.square_distance(&point);
        self.points[index] = POnSurf::new(u, v, ps);
        self.nb_ext += 1;
    }
    /// `Extrema_ExtPElS(P, gp_Cylinder, Tol)`;
    /// `ExtPElS::with_cylinder(point, cylinder, 1e-7)?`.
    pub fn with_cylinder(point: Point3, cylinder: Cylinder3, tol: f64) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_cylinder(point, &cylinder, tol)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Cylinder, Tol)`: the nearest and farthest points at the height
    /// of P; not done when P is within Tol of the axis;
    /// `extrema.perform_cylinder(point, &cylinder, 1e-7)?`.
    pub fn perform_cylinder(
        &mut self,
        point: Point3,
        cylinder: &Cylinder3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.reset();
        let position: Axis3 = cylinder.position();
        let origin: Point3 = position.location();
        let oz: Vector3 = Vector3::from(position.direction());
        let v: f64 = Vector3::between(origin, point).dot(oz);
        let pp: Point3 = point.translated(oz * -v);
        let opp: Vector3 = Vector3::between(origin, pp);
        if opp.magnitude() < tol {
            return Ok(());
        }
        let mut u1: f64 = frame_angle(&position, opp)?;
        let u2: f64 = u1 + PI;
        if u1 < 0. {
            u1 += 2. * PI;
        }
        self.push(point, cylinder, (u1, v));
        self.push(point, cylinder, (u2, v));
        self.done = true;
        Ok(())
    }
}

impl ExtPElS {
    /// `Extrema_ExtPElS(P, gp_Cone, Tol)`; `ExtPElS::with_cone(point, cone, 1e-7)?`.
    pub fn with_cone(point: Point3, cone: Cone3, tol: f64) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_cone(point, &cone, tol)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Cone, Tol)`: the apex when P is within Tol of it; otherwise the
    /// two points of the generatrices in the plane of P and the axis; not done when P
    /// is within Tol of the axis; `extrema.perform_cone(point, &cone, 1e-7)?`.
    pub fn perform_cone(&mut self, point: Point3, cone: &Cone3, tol: f64) -> Result<(), Failure> {
        self.reset();
        let apex: Point3 = cone.apex();
        let position: Axis3 = cone.position();
        let origin: Point3 = position.location();
        let semi_angle: f64 = cone.semi_angle();
        let oz: Vector3 = Vector3::from(position.direction());
        let mp: Vector3 = Vector3::between(apex, point);
        let l2: f64 = mp.square_magnitude();
        let vm: f64 = -(cone.ref_radius() / semi_angle.sin());
        // Case when P is mixed with the apex.
        if l2 < tol * tol {
            self.sq_dist[0] = l2;
            self.points[0] = POnSurf::new(0., vm, apex);
            self.nb_ext = 1;
            self.done = true;
            return Ok(());
        }
        let dir_z: Vector3 = if apex.square_distance(&origin) < tol * tol {
            if semi_angle < 0. { -oz } else { oz }
        } else {
            Vector3::between(apex, origin)
        };
        let zp: f64 = Vector3::between(origin, point).dot(oz);
        let opp: Vector3 = Vector3::between(origin, point.translated(oz * -zp));
        if opp.square_magnitude() < tol * tol {
            return Ok(());
        }
        let same: bool = dir_z.dot(mp) >= 0.0;
        let (u1, u2) = cone_angles(frame_angle(&position, opp)?, same);
        let (v1, v2) = cone_heights(mp, dir_z, semi_angle.abs(), same)?;
        let sense: f64 = oz.dot(Vector3::from(Direction3::from_vector(dir_z)?));
        self.push(point, cone, (u1, v1 * sense + vm));
        self.push(point, cone, (u2, v2 * sense + vm));
        self.done = true;
        Ok(())
    }
}

// U1 (shifted by PI when P is below the apex) and U2 = U1 + PI, both in [0, 2 PI].
fn cone_angles(angle: f64, same: bool) -> (f64, f64) {
    let mut u1: f64 = angle;
    if !same {
        u1 += PI;
    }
    let mut u2: f64 = u1 + PI;
    if u1 < 0. {
        u1 += 2. * PI;
    }
    if u2 > 2. * PI {
        u2 -= 2. * PI;
    }
    (u1, u2)
}

// `V1 = L cos(B - A)`, `V2 = L cos(B + A)` with `B = angle(MP, DirZ)`, reflected when
// P is below the apex.
fn cone_heights(mp: Vector3, dir_z: Vector3, a: f64, same: bool) -> Result<(f64, f64), Failure> {
    let b: f64 = mp.angle(dir_z)?;
    let l: f64 = mp.square_magnitude().sqrt();
    if !same {
        let b: f64 = PI - b;
        return Ok((-l * (b - a).cos(), -l * (b + a).cos()));
    }
    Ok((l * (b - a).cos(), l * (b + a).cos()))
}

impl ExtPElS {
    /// `Extrema_ExtPElS(P, gp_Sphere, Tol)`; `ExtPElS::with_sphere(point, sphere, 1e-7)?`.
    pub fn with_sphere(point: Point3, sphere: Sphere3, tol: f64) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_sphere(point, &sphere, tol)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Sphere, Tol)`: the nearest `(U1, V)` and farthest `(U1 + PI, -V)`
    /// points; a point on the axis gives the poles; not done at the centre;
    /// `extrema.perform_sphere(point, &sphere, 1e-7)?`.
    pub fn perform_sphere(
        &mut self,
        point: Point3,
        sphere: &Sphere3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.reset();
        let position: Axis3 = sphere.position();
        let op: Vector3 = Vector3::between(position.location(), point);
        // Case when P is mixed with O.
        if op.square_magnitude() < tol * tol {
            return Ok(());
        }
        let origin: Point3 = position.location();
        let oz: Vector3 = Vector3::from(position.direction());
        let zp: f64 = op.dot(oz);
        let opp: Vector3 = Vector3::between(origin, point.translated(oz * -zp));
        let (u1, u2, v) = if opp.square_magnitude() < tol * tol {
            (0., 0., if zp < 0. { -PI / 2. } else { PI / 2. })
        } else {
            let mut u1: f64 = frame_angle(&position, opp)?;
            let u2: f64 = u1 + PI;
            if u1 < 0. {
                u1 += 2. * PI;
            }
            let v: f64 = op.angle(opp)?;
            (u1, u2, if zp < 0. { -v } else { v })
        };
        self.push(point, sphere, (u1, v));
        self.push(point, sphere, (u2, -v));
        self.done = true;
        Ok(())
    }
}

impl ExtPElS {
    /// `Extrema_ExtPElS(P, gp_Torus, Tol)`; `ExtPElS::with_torus(point, torus, 1e-7)?`.
    pub fn with_torus(point: Point3, torus: Torus3, tol: f64) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_torus(point, &torus, tol)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Torus, Tol)`: four points in the plane of P and the axis (two on
    /// each meridian circle); not done when P is within Tol of the axis or of a
    /// meridian centre; `extrema.perform_torus(point, &torus, 1e-7)?`.
    pub fn perform_torus(
        &mut self,
        point: Point3,
        torus: &Torus3,
        tol: f64,
    ) -> Result<(), Failure> {
        let tol2: f64 = tol * tol;
        self.reset();
        let position: Axis3 = torus.position();
        let origin: Point3 = position.location();
        let oz: Vector3 = Vector3::from(position.direction());
        let height: f64 = Vector3::between(origin, point).dot(Vector3::from(position.direction()));
        let mut opp: Vector3 = Vector3::between(origin, point.translated(oz * -height));
        let r2: f64 = opp.square_magnitude();
        if r2 < tol2 {
            return Ok(());
        }
        let mut u1: f64 = frame_angle(&position, opp)?;
        let u2: f64 = u1 + PI;
        if u1 < 0. {
            u1 += 2. * PI;
        }
        let oo1: Vector3 = (opp / r2.sqrt()) * torus.major_radius();
        let o1: Point3 = origin.translated(oo1);
        let o2: Point3 = origin.translated(oo1 * -1.);
        if o1.square_distance(&point) < tol2 || o2.square_distance(&point) < tol2 {
            return Ok(());
        }
        let v1: f64 = snap_zero(opp.angle_with_ref(Vector3::between(o1, point), opp.crossed(oz))?);
        opp.reverse();
        let v2: f64 = snap_zero(opp.angle_with_ref(Vector3::between(point, o2), opp.crossed(oz))?);
        let (v1, v2) = (positive_angle(v1), positive_angle(v2));
        self.push(point, torus, (u1, v1));
        self.push(point, torus, (u1, v1 + PI));
        self.push(point, torus, (u2, v2));
        self.push(point, torus, (u2, v2 + PI));
        self.done = true;
        Ok(())
    }
}

// A negative angle moved by one turn.
fn positive_angle(angle: f64) -> f64 {
    if angle < 0. { angle + 2. * PI } else { angle }
}

impl ExtPElS {
    /// `Extrema_ExtPElS(P, gp_Pln, Tol)`; the tolerance is unused;
    /// `ExtPElS::with_plane(point, plane, 1e-7)`.
    pub fn with_plane(point: Point3, plane: Plane3, tol: f64) -> Self {
        let mut extrema: Self = Self::new();
        extrema.perform_plane(point, &plane, tol);
        extrema
    }
    /// `Perform(P, gp_Pln, Tol)`: the projection of P, with its plane parameters;
    /// `extrema.perform_plane(point, &plane, 1e-7)`.
    pub fn perform_plane(&mut self, point: Point3, plane: &Plane3, _tol: f64) {
        self.reset();
        let origin: Point3 = plane.location();
        let oz: Vector3 = Vector3::from(plane.axis().direction());
        let height: f64 = Vector3::between(origin, point).dot(oz);
        let pp: Point3 = point.translated(oz * -height);
        let (u, v) = elslib::parameters(plane, point);
        self.sq_dist[0] = pp.square_distance(&point);
        self.points[0] = POnSurf::new(u, v, pp);
        self.nb_ext = 1;
        self.done = true;
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtPElS::NbExt()"));
        }
        Ok(self.nb_ext)
    }
    /// `SquareDistance(N)`; Standard_OutOfRange outside `[1, NbExt()]`;
    /// `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let index: usize =
            extremum_index("Extrema_ExtPElS::SquareDistance()", n, || self.nb_ext())?;
        Ok(self.sq_dist[index])
    }
    /// `Point(N)`; `extrema.point(1)?.value()`.
    pub fn point(&self, n: i32) -> Result<POnSurf, Failure> {
        let index: usize = extremum_index("Extrema_ExtPElS::Point()", n, || self.nb_ext())?;
        Ok(self.points[index])
    }
}
