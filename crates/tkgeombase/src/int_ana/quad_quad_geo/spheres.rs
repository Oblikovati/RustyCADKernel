// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of IntAna_QuadQuadGeo::Perform(gp_Sphere, gp_Sphere / gp_Cone)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sphere-sphere and sphere-cone intersections.
use super::QuadQuadGeo;
use super::axe_operator::AxeOperator;
use super::cylinders::offset;
use crate::int_ana::ResultType;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Cone3, Direction3, Point3, Sphere3, Vector3};
use rustycad_tkmath::math::DirectPolynomialRoots;

impl QuadQuadGeo {
    /// Perform(gp_Sphere, gp_Sphere, Tol): same spheres (centers and radii within
    /// `tol`), empty for concentric different spheres, a point when internally
    /// tangent within `tol` or when the circle radius is below 0.01 Confusion, a
    /// circle when secant, else empty. `inter.perform_spheres(s1, s2, 1e-7)?`.
    pub fn perform_spheres(
        &mut self,
        first: Sphere3,
        second: Sphere3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let (o1, o2) = (first.location(), second.location());
        let d: f64 = o1.distance(&o2);
        let (r1, r2) = (first.radius(), second.radius());
        self.typeres = ResultType::Empty;
        self.param2bis = 0.0;
        let (rmin, rmax) = if r1 > r2 { (r2, r1) } else { (r1, r2) };
        if d <= tol && (r1 - r2).abs() <= tol {
            self.typeres = ResultType::Same;
            return Ok(());
        }
        if d <= tol {
            return Ok(());
        }
        let dir: Direction3 = Direction3::from_vector(Vector3::between(o1, o2))?;
        let t: f64 = rmax - d - rmin;
        if t >= 0.0 && t <= tol {
            // Internal tangency: one sphere touches the inside of the other.
            self.typeres = ResultType::Point;
            self.nbint = 1;
            let t2: f64 = if r1 == rmax {
                (r1 + (r2 + d)) * 0.5
            } else {
                (-r1 + (d - r2)) * 0.5
            };
            self.pt1 = offset(o1, t2, dir);
        } else if (d > (r1 + r2 + tol)) || (rmax > (d + rmin + tol)) {
            self.typeres = ResultType::Empty;
        } else {
            let mut alpha: f64 = 0.5 * (r1 * r1 - r2 * r2 + d * d) / d;
            let beta: f64 = r1 * r1 - alpha * alpha;
            let beta: f64 = if beta > 0.0 { beta.sqrt() } else { 0.0 };
            if beta <= self.my_epsilon_mini_circle_radius {
                self.typeres = ResultType::Point;
                alpha = (r1 + (d - r2)) * 0.5;
            } else {
                self.typeres = ResultType::Circle;
                self.dir1 = dir;
                self.param1 = beta;
            }
            self.pt1 = offset(o1, alpha, dir);
            self.nbint = 1;
        }
        Ok(())
    }

    /// Perform(gp_Sphere, gp_Cone, Tol): a sphere centered on the cone axis gives
    /// the circles of the roots of `x² + y² = R²`, `tan(a) = y / (x + d)` (d the
    /// apex distance); a circle of radius at most 0.01 Confusion makes the result a
    /// point and a circle. Otherwise no geometric solution; not done when the
    /// quadratic fails. The tolerance is unused. `inter.perform_sphere_cone(s, c, 1e-7)?`.
    pub fn perform_sphere_cone(
        &mut self,
        sphere: Sphere3,
        cone: Cone3,
        _tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let axes = AxeOperator::new(cone.axis(), sphere.position().axis())?;
        let center: Point3 = sphere.location();
        let centered: bool = axes.intersect() && (center.distance(&axes.pt_intersect()) == 0.0);
        if !(centered || axes.same()) {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let apex: Point3 = cone.apex();
        let d_apex: f64 = center.distance(&apex);
        let con_dir: Direction3 = if d_apex > f64::EPSILON {
            Direction3::from_vector(Vector3::between(apex, center))?
        } else {
            cone.position().direction()
        };
        let rad: f64 = sphere.radius();
        let tga: f64 = cone.semi_angle().tan();
        let tgatga: f64 = tga * tga;
        let eq = DirectPolynomialRoots::quadratic(
            1.0 + tgatga,
            2.0 * tgatga * d_apex,
            -rad * rad + d_apex * d_apex * tgatga,
        );
        if !eq.is_done() {
            self.done = false;
            return Ok(());
        }
        let nbsol: i32 = eq.solutions()?;
        if nbsol == 0 {
            self.typeres = ResultType::Empty;
            return Ok(());
        }
        self.typeres = ResultType::Circle;
        let first: f64 = d_apex + eq.value(1)?;
        self.nbint = 1;
        self.pt1 = offset(apex, first, con_dir);
        self.param1 = (tga * first).abs();
        self.dir1 = con_dir;
        if self.param1 <= self.my_epsilon_mini_circle_radius {
            self.typeres = ResultType::PointAndCircle;
            self.param1 = 0.0;
        }
        if nbsol >= 2 {
            let second: f64 = d_apex + eq.value(2)?;
            self.nbint = 2;
            self.pt2 = offset(apex, second, con_dir);
            self.param2 = (tga * second).abs();
            self.dir2 = con_dir;
            if self.param2 <= self.my_epsilon_mini_circle_radius {
                self.typeres = ResultType::PointAndCircle;
                self.param2 = 0.0;
            }
        }
        Ok(())
    }
}
