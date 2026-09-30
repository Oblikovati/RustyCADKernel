// Rust translation of the line-cylinder and line-sphere members of Extrema_ExtElCS.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The line-cylinder and line-sphere extrema of Extrema_ExtElCS.
use super::{ExtElCS, minimal_square_distance, parallel_square_distance, surface_point};
use crate::extrema::pon::{POnCurv, POnSurf};
use crate::extrema::{ExtElC, ExtPElC, ExtPElS};
use crate::int_ana::{IntConicQuad, Quadric};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Cylinder3, Line3, Point3, Sphere3};

impl ExtElCS {
    /// `Extrema_ExtElCS(gp_Lin, gp_Cylinder)`; `ExtElCS::line_cylinder(line, cylinder)?`.
    pub fn line_cylinder(line: Line3, cylinder: Cylinder3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_line_cylinder(&line, &cylinder)?;
        Ok(extrema)
    }
    /// `Perform(gp_Lin, gp_Cylinder)`: a line crossing the cylinder gives its
    /// intersection points (square distance 0, arrays sized for two more values);
    /// a tangent or outside line the extrema from its point nearest to the axis. A
    /// line parallel to the axis (for Extrema_ExtElC, or lying in the quadric for
    /// IntAna) is parallel with `(d - R)²`, `d` the smallest axis-line distance;
    /// `extrema.perform_line_cylinder(&line, &cylinder)?`.
    pub fn perform_line_cylinder(
        &mut self,
        line: &Line3,
        cylinder: &Cylinder3,
    ) -> Result<(), Failure> {
        self.done = false;
        self.nb_ext = 0;
        self.is_par = false;
        let radius: f64 = cylinder.radius();
        let extrem = ExtElC::lines(
            Line3::from_axis(cylinder.position().axis()),
            *line,
            precision::angular(),
        );
        let mut is_parallel: bool = extrem.is_parallel()?;
        if !is_parallel {
            let (on_axis, on_line) = extrem.points(1)?;
            if radius - on_axis.value().distance(on_line.value()) > precision::p_confusion() {
                is_parallel = self.line_cylinder_intersections(line, cylinder)?;
            } else {
                self.line_cylinder_projection(on_line, cylinder)?;
            }
            self.done = true;
        }
        if is_parallel {
            let sq_dist: f64 =
                minimal_square_distance(extrem.nb_ext()?, |i| extrem.square_distance(i))?;
            self.sq_dist = Some(vec![parallel_square_distance(sq_dist, radius)]);
            self.done = true;
            self.is_par = true;
            self.nb_ext = 1;
        }
        Ok(())
    }

    // The intersection points; true when IntAna finds the line in the quadric.
    fn line_cylinder_intersections(
        &mut self,
        line: &Line3,
        cylinder: &Cylinder3,
    ) -> Result<bool, Failure> {
        let inters = IntConicQuad::line_quadric(*line, &Quadric::from_cylinder(*cylinder))?;
        if !inters.is_done() {
            return Ok(false);
        }
        if inters.is_in_quadric()? {
            return Ok(true);
        }
        self.nb_ext = inters.nb_points()?;
        if self.nb_ext <= 0 {
            return Ok(false);
        }
        // Not more than 2 additional points from perpendiculars.
        self.allocate(self.nb_ext as usize + 2);
        let position: Axis3 = cylinder.position();
        for i in 1..=self.nb_ext {
            let p_int: Point3 = inters.point(i)?;
            let on_curve = POnCurv::new(inters.param_on_conic(i)?, p_int);
            let (u, v) = elslib::cylinder_parameters(position, cylinder.radius(), p_int);
            self.set_extremum(i as usize, 0., on_curve, POnSurf::new(u, v, p_int));
        }
        Ok(false)
    }

    // The extrema from the line point nearest to the axis, with that point on the curve.
    fn line_cylinder_projection(
        &mut self,
        on_line: POnCurv,
        cylinder: &Cylinder3,
    ) -> Result<(), Failure> {
        let projection =
            ExtPElS::with_cylinder(*on_line.value(), *cylinder, precision::confusion())?;
        if !projection.is_done() {
            return Ok(());
        }
        self.nb_ext = projection.nb_ext()?;
        self.allocate(self.nb_ext as usize);
        for i in 1..=self.nb_ext {
            let on_surface: POnSurf = projection.point(i)?;
            let sq_dist: f64 = on_line.value().square_distance(on_surface.value());
            self.set_extremum(i as usize, sq_dist, on_line, on_surface);
        }
        Ok(())
    }
}

impl ExtElCS {
    /// `Extrema_ExtElCS(gp_Lin, gp_Sphere)`; `ExtElCS::line_sphere(line, sphere)?`.
    pub fn line_sphere(line: Line3, sphere: Sphere3) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_line_sphere(&line, &sphere)?;
        Ok(extrema)
    }
    /// `Perform(gp_Lin, gp_Sphere)`: always done; a line through the sphere gives its
    /// two intersection points, then (for every line) the minimum and maximum from
    /// the line point nearest to the centre;
    /// `extrema.perform_line_sphere(&line, &sphere)?`.
    pub fn perform_line_sphere(&mut self, line: &Line3, sphere: &Sphere3) -> Result<(), Failure> {
        self.done = false;
        self.nb_ext = 0;
        self.is_par = false;
        let center: Point3 = sphere.location();
        let extrem = ExtPElC::with_line(center, *line, precision::angular(), f64::MIN, f64::MAX);
        if extrem.is_done() && extrem.nb_ext()? > 0 {
            let nearest: POnCurv = extrem.point(1)?;
            let mut start: i32 = 0;
            if nearest.value().distance(&center) <= sphere.radius() {
                start = self.line_sphere_intersections(line, sphere)?;
            }
            self.line_sphere_projection(nearest, sphere, start)?;
        }
        self.done = true;
        Ok(())
    }

    // The intersection points; returns their count (aStartIdx).
    fn line_sphere_intersections(
        &mut self,
        line: &Line3,
        sphere: &Sphere3,
    ) -> Result<i32, Failure> {
        let inters = IntConicQuad::line_quadric(*line, &Quadric::from_sphere(*sphere))?;
        if !inters.is_done() {
            return Ok(0);
        }
        self.nb_ext = inters.nb_points()?;
        // Not more than 2 additional points from perpendiculars.
        self.allocate(self.nb_ext as usize + 2);
        for i in 1..=self.nb_ext {
            let on_curve = POnCurv::new(inters.param_on_conic(i)?, inters.point(i)?);
            let on_surface: POnSurf = surface_point(sphere, inters.point(i)?);
            let sq_dist: f64 = on_curve.value().square_distance(on_surface.value());
            self.set_extremum(i as usize, sq_dist, on_curve, on_surface);
        }
        Ok(self.nb_ext)
    }

    // The sphere extrema from the nearest line point, after the `start` intersections.
    fn line_sphere_projection(
        &mut self,
        nearest: POnCurv,
        sphere: &Sphere3,
        start: i32,
    ) -> Result<(), Failure> {
        let projection = ExtPElS::with_sphere(*nearest.value(), *sphere, precision::confusion())?;
        if !projection.is_done() {
            return Ok(());
        }
        if start == 0 {
            self.nb_ext = projection.nb_ext()?;
            self.allocate(self.nb_ext as usize);
        } else {
            self.nb_ext += projection.nb_ext()?;
        }
        for i in start + 1..=self.nb_ext {
            let on_surface: POnSurf = projection.point(i - start)?;
            let sq_dist: f64 = nearest.value().square_distance(on_surface.value());
            self.set_extremum(i as usize, sq_dist, nearest, on_surface);
        }
        Ok(())
    }
}
