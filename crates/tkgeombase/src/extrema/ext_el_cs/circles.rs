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
// Rust translation of the circle members of Extrema_ExtElCS.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circle-plane, circle-cylinder and circle-sphere extrema of Extrema_ExtElCS.
use super::{ExtElCS, minimal_square_distance, parallel_square_distance, surface_point};
use crate::extrema::ExtElC;
use crate::extrema::pon::{POnCurv, POnSurf};
use crate::int_ana::{IntConicQuad, QuadQuadGeo, Quadric, ResultType};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis2, Circle3, Cylinder3, Direction3, Line3, Plane3, Point3, Sphere3, Vector3,
};
use rustycad_tkmath::{elclib, elslib};
use std::f64::consts::PI;

impl ExtElCS {
    /// `Extrema_ExtElCS(gp_Circ, gp_Pln)`; `ExtElCS::circle_plane(circle, plane)?`.
    pub fn circle_plane(circle: Circle3, plane: Plane3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_circle_plane(&circle, &plane)?;
        Ok(extrema)
    }
    /// `Perform(gp_Circ, gp_Pln)`: always done; the two circle points of extreme
    /// height, plus the intersection points when IntAna finds two; a circle parallel
    /// to (or lying in) the plane is parallel with the square distance of its centre;
    /// `extrema.perform_circle_plane(&circle, &plane)?`.
    pub fn perform_circle_plane(
        &mut self,
        circle: &Circle3,
        plane: &Plane3,
    ) -> Result<(), Failure> {
        self.done = true;
        self.is_par = false;
        self.nb_ext = 0;
        let position: Axis2 = circle.position();
        let (n_circ, n_pln) = (position.direction(), plane.axis().direction());
        let mut is_parallel: bool = n_circ.is_parallel(n_pln, precision::angular());
        if !is_parallel {
            let ext_line: Direction3 = n_circ.crossed(n_pln)?.crossed(n_circ)?;
            let mut t0: f64 = position.x_direction().angle_with_ref(ext_line, n_circ);
            if t0 < 0. {
                // Put in period
                t0 += PI;
            }
            self.nb_ext = 2;
            let tol: (f64, f64) = (precision::angular(), precision::confusion());
            let inter = IntConicQuad::circle_plane(*circle, *plane, tol.0, tol.1)?;
            is_parallel = inter.is_done() && inter.is_in_quadric()?;
            if !is_parallel && inter.is_done() && inter.nb_points()? > 1 {
                self.nb_ext += inter.nb_points()?;
            }
            if !is_parallel {
                self.circle_plane_points(circle, plane, [t0, t0 + PI], &inter)?;
            }
        }
        if is_parallel {
            self.set_parallel(plane.square_distance(circle.location()));
        }
        Ok(())
    }

    // The two extreme points, then the intersection points when there are two more.
    fn circle_plane_points(
        &mut self,
        circle: &Circle3,
        plane: &Plane3,
        extreme: [f64; 2],
        inter: &IntConicQuad,
    ) -> Result<(), Failure> {
        self.allocate(self.nb_ext as usize);
        let mut parameters: Vec<f64> = extreme.to_vec();
        if self.nb_ext > 2 {
            for i in 1..=inter.nb_points()? {
                parameters.push(inter.param_on_conic(i)?);
            }
        }
        for (i, t) in parameters.into_iter().enumerate() {
            let pc: Point3 = elclib::circle_value(t, circle.position(), circle.radius());
            let (u, v) = elslib::plane_parameters(plane.position(), pc);
            let pp: Point3 = elslib::plane_value(u, v, plane.position());
            self.set_extremum(
                i + 1,
                pc.square_distance(&pp),
                POnCurv::new(t, pc),
                POnSurf::new(u, v, pp),
            );
        }
        Ok(())
    }
}

impl ExtElCS {
    /// `Extrema_ExtElCS(gp_Circ, gp_Cylinder)`; `ExtElCS::circle_cylinder(circle, cylinder)?`.
    pub fn circle_cylinder(circle: Circle3, cylinder: Cylinder3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_circle_cylinder(&circle, &cylinder)?;
        Ok(extrema)
    }
    /// `Perform(gp_Circ, gp_Cylinder)` (OCC9134): for each axis-circle extremum not on
    /// the axis, the two cylinder points along it (square distances `(d ± R)²`), then
    /// the intersection points; a circle whose axis-circle extrema are parallel, or
    /// that lies in the quadric, is parallel with `(d - R)²`;
    /// `extrema.perform_circle_cylinder(&circle, &cylinder)?`.
    pub fn perform_circle_cylinder(
        &mut self,
        circle: &Circle3,
        cylinder: &Cylinder3,
    ) -> Result<(), Failure> {
        self.done = false;
        self.is_par = false;
        self.nb_ext = 0;
        let ext_c = ExtElC::line_circle(Line3::from_axis(cylinder.axis()), *circle, 0.)?;
        if !ext_c.is_done() {
            return Ok(());
        }
        let mut is_parallel: bool = ext_c.is_parallel()?;
        if !is_parallel {
            let inter = IntConicQuad::circle_quadric(*circle, &Quadric::from_cylinder(*cylinder))?;
            is_parallel = inter.is_done() && inter.is_in_quadric()?;
            let nb_inter: i32 = if !is_parallel && inter.is_done() {
                inter.nb_points()?
            } else {
                0
            };
            if !is_parallel {
                self.circle_cylinder_points(circle, cylinder, &ext_c, (&inter, nb_inter))?;
            }
        }
        self.done = true;
        if is_parallel {
            self.is_par = true;
            self.nb_ext = 1;
            let sq_dist: f64 =
                minimal_square_distance(ext_c.nb_ext()?, |i| ext_c.square_distance(i))?;
            self.sq_dist = Some(vec![parallel_square_distance(sq_dist, cylinder.radius())]);
        }
        Ok(())
    }

    // The cylinder points along each axis-circle extremum, then the intersections.
    fn circle_cylinder_points(
        &mut self,
        circle: &Circle3,
        cylinder: &Cylinder3,
        ext_c: &ExtElC,
        (inter, nb_inter): (&IntConicQuad, i32),
    ) -> Result<(), Failure> {
        let nb_ext: i32 = ext_c.nb_ext()?;
        self.nb_ext = 2 * nb_ext + nb_inter;
        self.allocate(self.nb_ext as usize);
        let tol_conf: f64 = precision::confusion();
        let mut current: usize = 1;
        for i in 1..=nb_ext {
            let sq_dist: f64 = ext_c.square_distance(i)?;
            let distance: f64 = sq_dist.sqrt();
            let (on_axis, on_circle) = ext_c.points(i)?;
            if sq_dist <= (tol_conf * tol_conf) {
                self.nb_ext -= 2;
                continue;
            }
            let dir = Direction3::from_xyz(on_axis.value().xyz() - on_circle.value().xyz())?;
            for shift in [distance + cylinder.radius(), distance - cylinder.radius()] {
                let on_cylinder: Point3 = on_circle.value().translated(Vector3::from(dir) * shift);
                self.set_extremum(
                    current,
                    shift * shift,
                    on_circle,
                    surface_point(cylinder, on_cylinder),
                );
                current += 1;
            }
        }
        // Adding intersection points to the list of extremas
        for i in 1..=nb_inter {
            let inter_pnt: Point3 = inter.point(i)?;
            let on_circle = POnCurv::new(elclib::parameter(circle, inter_pnt)?, inter_pnt);
            self.set_extremum(current, 0.0, on_circle, surface_point(cylinder, inter_pnt));
            current += 1;
        }
        Ok(())
    }
}

impl ExtElCS {
    /// `Extrema_ExtElCS(gp_Circ, gp_Sphere)`; `ExtElCS::circle_sphere(circle, sphere)?`.
    pub fn circle_sphere(circle: Circle3, sphere: Sphere3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_circle_sphere(&circle, &sphere)?;
        Ok(extrema)
    }
    /// `Perform(gp_Circ, gp_Sphere)`: a sphere centred on the circle axis is parallel;
    /// a circle plane missing (or touching) the sphere gives the circle point nearest
    /// to the centre; otherwise the minima of the extrema between the circle and the
    /// section circle (Extrema_ExtElC); not done when IntAna or that extrema fail;
    /// `extrema.perform_circle_sphere(&circle, &sphere)?`.
    pub fn perform_circle_sphere(
        &mut self,
        circle: &Circle3,
        sphere: &Sphere3,
    ) -> Result<(), Failure> {
        self.done = false;
        self.is_par = false;
        self.nb_ext = 0;
        let center: Point3 = sphere.location();
        if Line3::from_axis(circle.axis()).square_distance(center) < precision::square_confusion() {
            // Circle and sphere are parallel
            self.is_par = true;
            self.done = true;
            self.nb_ext = 1;
            let sq_dist: f64 =
                circle.location().square_distance(&center) + circle.radius() * circle.radius();
            self.sq_dist = Some(vec![parallel_square_distance(sq_dist, sphere.radius())]);
            return Ok(());
        }
        // Intersect sphere with circle's plane
        let plane = Plane3::new(circle.location(), circle.axis().direction())?;
        let inter = QuadQuadGeo::plane_sphere(plane, *sphere)?;
        if !inter.is_done() {
            return Ok(());
        }
        if inter.type_inter()? != ResultType::Circle {
            return self.nearest_circle_point(circle, sphere);
        }
        let section: Circle3 = inter.circle(1)?;
        let ext_c = ExtElC::circles(*circle, section)?;
        if !(ext_c.is_done() && !ext_c.is_parallel()? && ext_c.nb_ext()? > 0) {
            return Ok(());
        }
        self.done = true;
        self.minimal_circle_solutions(&ext_c, sphere)
    }

    // The intersection is empty or a point: one minimal solution.
    fn nearest_circle_point(&mut self, circle: &Circle3, sphere: &Sphere3) -> Result<(), Failure> {
        self.nb_ext = 1;
        self.done = true;
        self.allocate(1);
        let t: f64 = elclib::parameter(circle, sphere.location())?;
        let on_circle: Point3 = elclib::value(t, circle);
        let (u, v) = elslib::parameters(sphere, on_circle);
        let on_sphere: Point3 = elslib::value(u, v, sphere);
        let sq_dist: f64 = on_circle.square_distance(&on_sphere);
        self.set_extremum(
            1,
            sq_dist,
            POnCurv::new(t, on_circle),
            POnSurf::new(u, v, on_sphere),
        );
        Ok(())
    }

    // The circle-section extrema within Precision::SquareConfusion() of the smallest.
    fn minimal_circle_solutions(
        &mut self,
        ext_c: &ExtElC,
        sphere: &Sphere3,
    ) -> Result<(), Failure> {
        let count: i32 = ext_c.nb_ext()?;
        let mut min_sq_dist: f64 = f64::MAX;
        for i in 1..=count {
            let sq_dist: f64 = ext_c.square_distance(i)?;
            if sq_dist < min_sq_dist {
                min_sq_dist = sq_dist;
            }
        }
        let mut solutions: Vec<i32> = Vec::new();
        for i in 1..=count {
            if ext_c.square_distance(i)? - min_sq_dist < precision::square_confusion() {
                solutions.push(i);
            }
        }
        self.nb_ext = solutions.len() as i32;
        self.allocate(solutions.len());
        for (index, solution) in solutions.into_iter().enumerate() {
            let (on_circle, _) = ext_c.points(solution)?;
            let (u, v) = elslib::parameters(sphere, *on_circle.value());
            let on_sphere: Point3 = elslib::value(u, v, sphere);
            let sq_dist: f64 = on_circle.value().square_distance(&on_sphere);
            self.set_extremum(index + 1, sq_dist, on_circle, POnSurf::new(u, v, on_sphere));
        }
        Ok(())
    }
}
