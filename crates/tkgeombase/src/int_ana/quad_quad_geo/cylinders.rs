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
// Rust translation of IntAna_QuadQuadGeo::Perform(gp_Cylinder, gp_Cylinder / gp_Sphere /
// gp_Cone) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cylinder-cylinder, cylinder-sphere and cylinder-cone intersections.
use super::QuadQuadGeo;
use super::axe_operator::AxeOperator;
use crate::int_ana::ResultType;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Direction3, Point3, Sphere3, Vector3};
use std::f64::consts::PI;

impl QuadQuadGeo {
    /// Perform(gp_Cylinder, gp_Cylinder, Tol): parallel axes give same, empty,
    /// one tangent line or two lines (one when their chord is below `tol`);
    /// intersecting axes and equal radii give two ellipses; skew axes at the radii
    /// sum give the tangency point; otherwise no geometric solution.
    /// `inter.perform_cylinders(c1, c2, 1e-7)?`.
    pub fn perform_cylinders(
        &mut self,
        first: Cylinder3,
        second: Cylinder3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let axes = AxeOperator::with_tolerances(
            first.axis(),
            second.axis(),
            self.my_epsilon_cylinder_delta_distance,
            self.my_epsilon_axes_para,
        )?;
        let (r1, r2) = (first.radius(), second.radius());
        let rmr: f64 = if r1 > r2 { r1 - r2 } else { r2 - r1 };
        let rmr_relative: f64 = rmr / if r1 > r2 { r1 } else { r2 };
        let dist: f64 = axes.distance();
        if axes.parallel() {
            if dist <= tol {
                self.typeres = if rmr <= tol {
                    ResultType::Same
                } else {
                    ResultType::Empty
                };
                return Ok(());
            }
            return self.parallel_cylinders(first, second, tol, dist);
        }
        if (rmr_relative <= self.my_epsilon_cylinder_delta_radius) && axes.intersect() {
            return self.crossing_cylinders(first, second, &axes);
        }
        if (dist - first.radius() - second.radius()).abs() < tol {
            return self.skew_tangent_cylinders(first, second, &axes);
        }
        self.typeres = ResultType::NoGeometricSolution;
        Ok(())
    }

    /// Parallel axes farther apart than `tol`: the circles of the two bases in the
    /// base plane of the first cylinder.
    fn parallel_cylinders(
        &mut self,
        first: Cylinder3,
        second: Cylinder3,
        tol: f64,
        dist: f64,
    ) -> Result<(), Failure> {
        let (r1, r2) = (first.radius(), second.radius());
        let rmr: f64 = if r1 > r2 { r1 - r2 } else { r2 - r1 };
        let p1: Point3 = first.location();
        let p2t: Point3 = second.location();
        let dir_cyl: Direction3 = first.position().direction();
        // The location of the second cylinder projected on the base of the first.
        let projection: f64 = Vector3::from(dir_cyl).dot(Vector3::between(p1, p2t));
        let p2 = Point3::new(
            p2t.x() - projection * dir_cyl.x(),
            p2t.y() - projection * dir_cyl.y(),
            p2t.z() - projection * dir_cyl.z(),
        );
        let r1p_r2: f64 = r1 + r2;
        if dist > (r1p_r2 + tol) {
            self.typeres = ResultType::Empty;
            self.nbint = 0;
        } else if (r1p_r2 - dist) <= f64::MIN_POSITIVE {
            // One tangent line.
            self.typeres = ResultType::Line;
            self.nbint = 1;
            self.dir1 = dir_cyl;
            self.pt1 = along(p1, p2, r1 / r1p_r2);
        } else if dist > rmr {
            self.secant_cylinders(first, (r2, tol, dist), (p1, p2));
        } else if dist > (rmr - tol) {
            // One tangent line inside.
            self.typeres = ResultType::Line;
            self.nbint = 1;
            self.dir1 = dir_cyl;
            let mut ratio: f64 = r1 / rmr;
            if r1 < r2 {
                ratio = -ratio;
            }
            self.pt1 = along(p1, p2, ratio);
        } else {
            self.nbint = 0;
            self.typeres = ResultType::Empty;
        }
        Ok(())
    }

    /// Two lines through the intersection points A and B of the base circles
    /// (centers P1, P2); one line when `AB = 2 R1 sin(A P1 P2)` is below `tol`.
    fn secant_cylinders(
        &mut self,
        first: Cylinder3,
        (r2, tol, dist): (f64, f64, f64),
        (p1, p2): (Point3, Point3),
    ) {
        let r1: f64 = first.radius();
        self.typeres = ResultType::Line;
        self.nbint = 2;
        self.dir1 = first.position().direction();
        self.dir2 = self.dir1;
        let r1r1: f64 = r1 * r1;
        // Cosine and square sine of the angle A-P1-P2.
        let cos: f64 = 0.5 * (r1r1 - r2 * r2 + dist * dist) / (r1 * dist);
        let sin2: f64 = 1. - cos * cos;
        let direction = Vector3::from((p2.xyz() - p1.xyz()) / dist);
        if (4.0 * r1r1 * sin2) < tol * tol {
            // The segment from P1 along P1P2 of length |P1 O1|.
            self.nbint = 1;
            self.pt1.set_xyz(p1.xyz() + *direction.xyz() * r1 * cos);
            return;
        }
        // Here sin2 >= 0 (a negative one makes the tangent branch).
        let sin: f64 = sin2.sqrt();
        let (x_dir, y_dir) = (
            first.position().x_direction(),
            first.position().y_direction(),
        );
        let (r1_x, r1_y) = (r1 * x_dir.xyz(), r1 * y_dir.xyz());
        let dx: f64 = direction.dot(Vector3::from(x_dir));
        let dy: f64 = direction.dot(Vector3::from(y_dir));
        // P1P2 rotated by -/+ the angle A-P1-P2 around P1, at the distance R1.
        let (new_dx, new_dy) = (dx * cos - dy * sin, dy * cos + dx * sin);
        self.pt1.set_xyz(p1.xyz() + new_dx * r1_x + new_dy * r1_y);
        let (new_dx, new_dy) = (dx * cos + dy * sin, dy * cos - dx * sin);
        self.pt2.set_xyz(p1.xyz() + new_dx * r1_x + new_dy * r1_y);
    }

    /// Intersecting axes and equal radii: two ellipses through the axes point,
    /// along the bisectors of the axes; same when the axes are (anti)parallel.
    fn crossing_cylinders(
        &mut self,
        first: Cylinder3,
        second: Cylinder3,
        axes: &AxeOperator,
    ) -> Result<(), Failure> {
        self.typeres = ResultType::Ellipse;
        self.nbint = 2;
        let (d1, d2) = (first.position().direction(), second.position().direction());
        self.pt1 = axes.pt_intersect();
        self.pt2 = self.pt1;
        let angle: f64 = d1.angle(&d2);
        let b: f64 = (0.5 * (PI - angle)).sin().abs();
        let a: f64 = (0.5 * angle).sin().abs();
        if a == 0.0 || b == 0.0 {
            self.typeres = ResultType::Same;
            return Ok(());
        }
        let (v1, v2) = (Vector3::from(d1), Vector3::from(d2));
        self.dir1 = Direction3::from_vector(v1 + v2)?;
        self.dir2 = Direction3::from_vector(v1 - v2)?;
        let radius: f64 = first.radius();
        self.param2 = radius / a;
        self.param1 = radius / b;
        self.param1bis = radius;
        self.param2bis = radius;
        if self.param1 < self.param1bis {
            std::mem::swap(&mut self.param1, &mut self.param1bis);
        }
        if self.param2 < self.param2bis {
            std::mem::swap(&mut self.param2, &mut self.param2bis);
        }
        Ok(())
    }

    /// Skew axes at the distance of the radii sum: the tangency point on the
    /// common perpendicular, at R1 from the first axis.
    fn skew_tangent_cylinders(
        &mut self,
        first: Cylinder3,
        second: Cylinder3,
        axes: &AxeOperator,
    ) -> Result<(), Failure> {
        self.typeres = ResultType::Point;
        let (d1, d2) = (first.axis().direction(), second.axis().direction());
        let (_, p1, p2) = axes.distance_parameters()?;
        let o1: Point3 = first.axis().location();
        let foot1 = Point3::new(
            o1.x() - p1 * d1.x(),
            o1.y() - p1 * d1.y(),
            o1.z() - p1 * d1.z(),
        );
        let o2: Point3 = second.axis().location();
        let foot2 = Point3::new(
            o2.x() - p2 * d2.x(),
            o2.y() - p2 * d2.y(),
            o2.z() - p2 * d2.z(),
        );
        let towards: Direction3 = Direction3::from_vector(Vector3::between(foot1, foot2))?;
        let r1: f64 = first.radius();
        self.pt1 = Point3::new(
            foot1.x() + r1 * towards.x(),
            foot1.y() + r1 * towards.y(),
            foot1.z() + r1 * towards.z(),
        );
        self.nbint = 1;
        Ok(())
    }

    /// Perform(gp_Cylinder, gp_Sphere, Tol): a sphere centered on the axis gives
    /// one or two circles (empty for a smaller sphere); otherwise no geometric
    /// solution. The tolerance is unused. `inter.perform_cylinder_sphere(c, s, 1e-7)?`.
    pub fn perform_cylinder_sphere(
        &mut self,
        cylinder: Cylinder3,
        sphere: Sphere3,
        _tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let center: Point3 = sphere.location();
        let axes = AxeOperator::new(cylinder.axis(), sphere.position().axis())?;
        let centered: bool = axes.intersect() && center.distance(&axes.pt_intersect()) == 0.0;
        if !(centered || axes.same()) {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        if sphere.radius() < cylinder.radius() {
            self.typeres = ResultType::Empty;
            return Ok(());
        }
        let dist: f64 =
            (sphere.radius() * sphere.radius() - cylinder.radius() * cylinder.radius()).sqrt();
        let dir: Direction3 = cylinder.position().direction();
        self.dir1 = dir;
        self.dir2 = dir;
        self.typeres = ResultType::Circle;
        self.pt1 = offset(center, dist, dir);
        self.nbint = 1;
        self.param1 = cylinder.radius();
        if dist > f64::EPSILON {
            self.pt2 = offset(center, -dist, dir);
            self.param2 = cylinder.radius();
            self.nbint = 2;
        }
        Ok(())
    }

    /// Perform(gp_Cylinder, gp_Cone, Tol): coaxial surfaces give the two circles
    /// at `R / tan(SemiAngle)` on both sides of the apex; otherwise no geometric
    /// solution. The tolerance is unused. `inter.perform_cylinder_cone(c, k, 1e-7)?`.
    pub fn perform_cylinder_cone(
        &mut self,
        cylinder: Cylinder3,
        cone: Cone3,
        _tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let axes = AxeOperator::new(cylinder.axis(), cone.axis())?;
        if !axes.same() {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let apex: Point3 = cone.apex();
        let dist: f64 = cylinder.radius() / cone.semi_angle().tan();
        let dir: Direction3 = cylinder.position().direction();
        self.pt1 = offset(apex, dist, dir);
        self.pt2 = offset(apex, -dist, dir);
        self.dir1 = dir;
        self.dir2 = dir;
        self.param1 = cylinder.radius();
        self.param2 = cylinder.radius();
        self.nbint = 2;
        self.typeres = ResultType::Circle;
        Ok(())
    }
}

/// `P + dist D` coordinate by coordinate (`P - |dist| D` written as a negative
/// `dist`, which rounds identically since `x - d y == x + (-d) y`).
pub(super) fn offset(point: Point3, dist: f64, direction: Direction3) -> Point3 {
    Point3::new(
        point.x() + dist * direction.x(),
        point.y() + dist * direction.y(),
        point.z() + dist * direction.z(),
    )
}

/// `P1 + ratio (P2 - P1)`, coordinate by coordinate.
fn along(p1: Point3, p2: Point3, ratio: f64) -> Point3 {
    Point3::new(
        p1.x() + ratio * (p2.x() - p1.x()),
        p1.y() + ratio * (p2.y() - p1.y()),
        p1.z() + ratio * (p2.z() - p1.z()),
    )
}
