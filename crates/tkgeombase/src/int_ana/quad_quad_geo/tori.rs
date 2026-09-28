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
// Rust translation of the torus Perform overloads of IntAna_QuadQuadGeo.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of a torus with a plane, a cylinder, a cone, a sphere or a
//! torus: circles when the configuration is coaxial (or the plane contains the
//! axis), else no geometric solution. A torus whose minor radius reaches the
//! major one has no geometric solution.
use super::QuadQuadGeo;
use crate::int_ana::ResultType;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Line3, Plane3, Point3, Sphere3, Torus3, Vector3, Xyz,
};

/// The meridian section of a coaxial intersection: the circle of the torus tube
/// (center `tube`, radius `minor`) meets a circle or a line of the other surface
/// at `ph ± dval`; each point sweeps a circle around the torus axis.
struct MeridianPoints {
    ph: Xyz,
    dval: Xyz,
    axis: Line3,
    x_dir: Direction3,
}

impl QuadQuadGeo {
    /// Perform(gp_Pln, gp_Torus, Tol): a plane normal to the axis cuts one or two
    /// circles (tangent within 1e-13 of the tube), a plane through the axis (within
    /// 1e-14) two tube circles; otherwise no geometric solution.
    /// `inter.perform_plane_torus(plane, torus, 1e-7)?`.
    pub fn perform_plane_torus(
        &mut self,
        plane: Plane3,
        torus: Torus3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let (rmin, rmaj) = (torus.minor_radius(), torus.major_radius());
        if rmin >= rmaj {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let (pln_ax, tor_ax): (Axis1, Axis1) = (plane.axis(), torus.axis());
        let parallel: bool = tor_ax.is_parallel(pln_ax, self.my_epsilon_axes_para);
        let normal: bool = !parallel && tor_ax.is_normal(pln_ax, self.my_epsilon_axes_para);
        if !normal && !parallel {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        if parallel {
            return self.plane_across_torus(plane, torus, tol);
        }
        let tor_loc: Point3 = tor_ax.location();
        if plane.distance(tor_loc) > self.my_epsilon_distance {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        self.typeres = ResultType::Circle;
        self.param1 = rmin;
        self.param2 = rmin;
        self.dir1 = pln_ax.direction();
        self.dir2 = self.dir1;
        self.nbint = 2;
        let dir: Direction3 = tor_ax.direction().crossed(self.dir1)?;
        self.pt1.set_xyz(tor_loc.xyz() + rmaj * dir.xyz());
        self.pt2.set_xyz(tor_loc.xyz() - rmaj * dir.xyz());
        Ok(())
    }

    /// A plane normal to the torus axis: the circles at the height of the plane.
    fn plane_across_torus(
        &mut self,
        plane: Plane3,
        torus: Torus3,
        tol: f64,
    ) -> Result<(), Failure> {
        let (rmin, rmaj) = (torus.minor_radius(), torus.major_radius());
        let tol_num: f64 = self.my_epsilon_cylinder_delta_radius;
        let [a, b, c, d] = plane.coefficients()?;
        let tor_loc: Point3 = torus.axis().location();
        let (x, y, z) = (tor_loc.x(), tor_loc.y(), tor_loc.z());
        let mut dist: f64 = a * x + b * y + c * z + d;
        let dr: f64 = dist.abs() - rmin;
        if dr > tol_num {
            self.typeres = ResultType::Empty;
            return Ok(());
        }
        if dr.abs() < tol_num {
            dist = if dist < 0.0 { -rmin } else { rmin };
        }
        self.typeres = ResultType::Circle;
        self.pt1 = Point3::new(x - dist * a, y - dist * b, z - dist * c);
        let dt: f64 = (rmin * rmin - dist * dist).abs().sqrt();
        self.param1 = rmaj + dt;
        self.dir1 = torus.axis().direction();
        self.nbint = 1;
        if (dr < -tol_num) && (dt > tol) {
            self.pt2 = self.pt1;
            self.param2 = rmaj - dt;
            self.dir2 = self.dir1;
            self.nbint = 2;
        }
        Ok(())
    }

    /// Perform(gp_Cylinder, gp_Torus, Tol): a coaxial cylinder crossing the tube
    /// cuts one or two circles; empty when it misses the tube by `tol`.
    /// `inter.perform_cylinder_torus(cylinder, torus, 1e-7)?`.
    pub fn perform_cylinder_torus(
        &mut self,
        cylinder: Cylinder3,
        torus: Torus3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let (rmin, rmaj) = (torus.minor_radius(), torus.major_radius());
        if rmin >= rmaj || !self.coaxial(torus, cylinder.axis(), cylinder.location()) {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let rcyl: f64 = cylinder.radius();
        if ((rcyl + tol) < (rmaj - rmin)) || ((rcyl - tol) > (rmaj + rmin)) {
            self.typeres = ResultType::Empty;
            return Ok(());
        }
        self.typeres = ResultType::Circle;
        let dist: f64 = (rmin * rmin - (rcyl - rmaj) * (rcyl - rmaj)).abs().sqrt();
        let tor_loc: Xyz = torus.axis().location().xyz();
        self.dir1 = torus.axis().direction();
        self.pt1.set_xyz(tor_loc + dist * self.dir1.xyz());
        self.param1 = rcyl;
        self.nbint = 1;
        if (dist > tol) && (rcyl > (rmaj - rmin)) && (rcyl < (rmaj + rmin)) {
            self.dir2 = self.dir1;
            self.pt2.set_xyz(tor_loc - dist * self.dir2.xyz());
            self.param2 = self.param1;
            self.nbint = 2;
        }
        Ok(())
    }

    /// True when `axis` is parallel to the torus axis and `point` on it (1e-14).
    fn coaxial(&self, torus: Torus3, axis: Axis1, point: Point3) -> bool {
        let tor_ax: Axis1 = torus.axis();
        let distance: f64 = Line3::from_axis(tor_ax).distance(point);
        tor_ax.is_parallel(axis, self.my_epsilon_axes_para)
            && (distance.is_nan() || distance <= self.my_epsilon_distance)
    }

    /// Perform(gp_Sphere, gp_Torus, Tol): a sphere centered on the axis cuts the
    /// tube circle in the meridian plane: one or two circles.
    /// `inter.perform_sphere_torus(sphere, torus, 1e-7)?`.
    pub fn perform_sphere_torus(
        &mut self,
        sphere: Sphere3,
        torus: Torus3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let (rmin, rmaj) = (torus.minor_radius(), torus.major_radius());
        let axis = Line3::from_axis(torus.axis());
        let sph_loc: Point3 = sphere.location();
        if rmin >= rmaj || axis.distance(sph_loc) > self.my_epsilon_distance {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let x_dir: Direction3 = torus.x_axis().direction();
        let tube = Point3::from(torus.axis().location().xyz() + rmaj * x_dir.xyz());
        self.tube_circles(
            torus,
            (tube, rmin),
            (sph_loc, sphere.radius()),
            tol,
            (axis, x_dir),
        )
    }

    /// Perform(gp_Torus, gp_Torus, Tol): coaxial tori: same when equal within
    /// `tol`, else the circles of their tube circles in a meridian plane.
    /// `inter.perform_tori(t1, t2, 1e-7)?`.
    pub fn perform_tori(&mut self, first: Torus3, second: Torus3, tol: f64) -> Result<(), Failure> {
        self.done = true;
        let (rmin1, rmaj1) = (first.minor_radius(), first.major_radius());
        let (rmin2, rmaj2) = (second.minor_radius(), second.major_radius());
        let (ax1, ax2) = (first.axis(), second.axis());
        let (loc1, loc2) = (ax1.location(), ax2.location());
        let axis = Line3::from_axis(ax1);
        if !ax1.is_parallel(ax2, self.my_epsilon_axes_para)
            || (axis.distance(loc2) > self.my_epsilon_distance)
        {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        if loc1.is_equal(&loc2, tol)
            && ((rmin1 - rmin2).abs() <= tol)
            && ((rmaj1 - rmaj2).abs() <= tol)
        {
            self.typeres = ResultType::Same;
            return Ok(());
        }
        if rmin1 >= rmaj1 || rmin2 >= rmaj2 {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let x_dir: Direction3 = first.x_axis().direction();
        let p1 = Point3::from(loc1.xyz() + rmaj1 * x_dir.xyz());
        let p2 = Point3::from(loc2.xyz() + rmaj2 * x_dir.xyz());
        self.tube_circles(first, (p1, rmin1), (p2, rmin2), tol, (axis, x_dir))
    }

    /// The intersection of the tube circle `(tube, rmin)` with the circle
    /// `(other, radius)` in the meridian plane of `torus`: empty when apart (or
    /// nested) by `tol`, else one circle, or two when secant and apart by `tol`.
    fn tube_circles(
        &mut self,
        torus: Torus3,
        (tube, rmin): (Point3, f64),
        (other, radius): (Point3, f64),
        tol: f64,
        (axis, x_dir): (Line3, Direction3),
    ) -> Result<(), Failure> {
        let v12 = Vector3::between(tube, other);
        let dist: f64 = v12.magnitude();
        if ((dist - tol) > (rmin + radius)) || ((dist + tol) < (rmin - radius).abs()) {
            self.typeres = ResultType::Empty;
            return Ok(());
        }
        self.typeres = ResultType::Circle;
        let alpha: f64 = 0.5 * (rmin * rmin - radius * radius + dist * dist) / dist;
        let beta: f64 = (rmin * rmin - alpha * alpha).abs().sqrt();
        let dir12: Direction3 = Direction3::from_vector(v12)?;
        let ph: Xyz = tube.xyz() + alpha * dir12.xyz();
        let dc: Direction3 = torus.y_axis().direction().crossed(dir12)?;
        let points = MeridianPoints {
            ph,
            dval: beta * dc.xyz(),
            axis,
            x_dir,
        };
        let axis_dir: Direction3 = torus.axis().direction();
        (self.pt1, self.param1) = points.swept(1.0);
        self.dir1 = axis_dir;
        self.nbint = 1;
        let secant: bool = (dist < (radius + rmin)) && (dist > (radius - rmin).abs());
        if secant && points.dval.modulus() > tol {
            (self.pt2, self.param2) = points.swept(-1.0);
            self.dir2 = self.dir1;
            self.nbint = 2;
        }
        Ok(())
    }

    /// Perform(gp_Cone, gp_Torus, Tol): a cone of apex on the torus axis: its
    /// generatrix in the meridian plane cuts the tube circle on each side of the
    /// axis in up to two points, each giving a circle (up to four). Native quirk
    /// kept: the circle count starts from the previous NbSolutions() (0 for a new
    /// intersection); the slots below it keep default values where the native
    /// arrays are uninitialized. `inter.perform_cone_torus(cone, torus, 1e-7)?`.
    pub fn perform_cone_torus(
        &mut self,
        cone: Cone3,
        torus: Torus3,
        tol: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        let (rmin, rmaj) = (torus.minor_radius(), torus.major_radius());
        if rmin >= rmaj || !self.coaxial(torus, cone.axis(), cone.apex()) {
            self.typeres = ResultType::NoGeometricSolution;
            return Ok(());
        }
        let axis = Line3::from_axis(torus.axis());
        let tor_loc: Point3 = torus.axis().location();
        let pn = Point3::from(tor_loc.xyz() + rmaj * torus.y_axis().direction().xyz());
        let dn: Direction3 = Direction3::from_vector(Vector3::between(tor_loc, pn))?;
        let generatrix: Line3 = axis.rotated(Axis1::new(cone.apex(), dn), cone.semi_angle())?;
        let mut x_dir: Direction3 = torus.x_axis().direction();
        self.typeres = ResultType::Empty;
        let mut circles: [(Point3, f64); 4] = [(Point3::default(), 0.0); 4];
        for side in 0..2 {
            if side == 1 {
                x_dir.reverse();
            }
            let tube = Point3::from(tor_loc.xyz() + rmaj * x_dir.xyz());
            let dist: f64 = generatrix.distance(tube);
            if dist > rmin + tol {
                continue;
            }
            self.typeres = ResultType::Circle;
            let ph: Xyz = tube.xyz() - dist * generatrix.normal(tube)?.direction().xyz();
            let dt: f64 = (rmin * rmin - dist * dist).abs().sqrt();
            let points = MeridianPoints {
                ph,
                dval: dt * generatrix.direction().xyz(),
                axis,
                x_dir,
            };
            self.push_cone_circle(&mut circles, points.swept(1.0))?;
            if (dist < rmin) && (dt > tol) {
                self.push_cone_circle(&mut circles, points.swept(-1.0))?;
            }
        }
        self.store_cone_circles(&circles, torus.axis().direction());
        Ok(())
    }

    /// Stores one cone-torus circle at the current count; Standard_OutOfRange past
    /// the four native slots (undefined behaviour natively).
    fn push_cone_circle(
        &mut self,
        circles: &mut [(Point3, f64); 4],
        circle: (Point3, f64),
    ) -> Result<(), Failure> {
        let slot: usize = self.nbint as usize;
        if slot >= circles.len() {
            return Err(Failure::out_of_range(&format!(
                "IntAna_QuadQuadGeo::Perform(gp_Cone, gp_Torus) - circle {} exceeds the 4 slots",
                slot + 1
            )));
        }
        circles[slot] = circle;
        self.nbint += 1;
        Ok(())
    }

    /// Copies the cone-torus circles into the solution fields.
    fn store_cone_circles(&mut self, circles: &[(Point3, f64); 4], axis_dir: Direction3) {
        let count: usize = (self.nbint.max(0) as usize).min(circles.len());
        for (i, &(point, radius)) in circles.iter().enumerate().take(count) {
            match i {
                0 => (self.pt1, self.param1, self.dir1) = (point, radius, axis_dir),
                1 => (self.pt2, self.param2, self.dir2) = (point, radius, axis_dir),
                2 => (self.pt3, self.param3, self.dir3) = (point, radius, axis_dir),
                _ => (self.pt4, self.param4, self.dir4) = (point, radius, axis_dir),
            }
        }
    }
}

impl MeridianPoints {
    /// The circle through `ph + sign dval`: its center on the axis (the point
    /// moved back along the X direction by its axis distance) and its radius.
    fn swept(&self, sign: f64) -> (Point3, f64) {
        let point = Point3::from(if sign > 0.0 {
            self.ph + self.dval
        } else {
            self.ph - self.dval
        });
        let radius: f64 = self.axis.distance(point);
        (
            Point3::from(point.xyz() - radius * self.x_dir.xyz()),
            radius,
        )
    }
}
