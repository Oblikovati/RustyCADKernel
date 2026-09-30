// Contap_ContAna.cxx sphere overloads at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Closed-form sphere silhouettes and draft circles.
use super::cont_ana::{ContapContAna, TOLPETIT, circle_x_direction};
use super::geometry::point_of;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction3, Point3, Sphere3};
impl ContapContAna {
    /// Perform(gp_Sphere, gp_Dir): the great circle normal to `d`; its X direction is
    /// `d` crossed with the sphere X axis (with the Y axis when they are parallel);
    /// `contour.perform_sphere(&s, d)?`.
    pub fn perform_sphere(&mut self, s: &Sphere3, d: Direction3) -> Result<(), Failure> {
        self.done = false;
        self.typ_l = CurveType::Circle;
        self.pt[0] = s.location();
        self.dir[0] = d;
        self.dir[1] = circle_x_direction(s, d)?;
        self.prm = s.radius();
        self.nb_sol = 1;
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Sphere, gp_Dir, Ang): the circle of the points whose normal makes the
    /// draft angle `ang` with `d` (the angle is reversed on an indirect sphere);
    /// `contour.perform_sphere_draft(&s, d, 0.1)?`.
    pub fn perform_sphere_draft(
        &mut self,
        s: &Sphere3,
        d: Direction3,
        ang: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.typ_l = CurveType::Circle;
        self.dir[0] = d;
        self.dir[1] = circle_x_direction(s, d)?;
        let alpha = if s.direct()? { ang } else { -ang };
        self.pt[0] = point_of(s.location().xyz() - s.radius() * alpha.sin() * d.xyz());
        self.prm = s.radius() * alpha.cos();
        self.nb_sol = 1;
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Sphere, gp_Pnt Eye): the circle of tangency of the cone from `eye`;
    /// none when the eye is inside the sphere or the circle is smaller than 1e-8 (the
    /// contour type then keeps its previous value); `contour.perform_sphere_eye(&s, eye)?`.
    pub fn perform_sphere_eye(&mut self, s: &Sphere3, eye: Point3) -> Result<(), Failure> {
        self.done = false;
        let radius = s.radius();
        let dist = eye.distance(&s.location());
        if dist <= radius {
            self.nb_sol = 0;
            self.done = true;
            return Ok(());
        }
        self.prm = radius * (1. - radius * radius / (dist * dist)).sqrt();
        if self.prm < TOLPETIT {
            self.nb_sol = 0;
        } else {
            self.store_sphere_eye(s, eye, dist)?;
        }
        self.done = true;
        Ok(())
    }
    fn store_sphere_eye(&mut self, s: &Sphere3, eye: Point3, dist: f64) -> Result<(), Failure> {
        let radius = s.radius();
        let locxyz = s.location().xyz();
        self.dir[0] = Direction3::from_xyz(eye.xyz() - locxyz)?;
        self.pt[0] = point_of(locxyz + (radius * radius / dist) * self.dir[0].xyz());
        self.dir[1] = circle_x_direction(s, self.dir[0])?;
        self.nb_sol = 1;
        self.typ_l = CurveType::Circle;
        Ok(())
    }
}
