// Contap_ContAna.cxx cylinder overloads at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Closed-form cylinder generatrices.
use super::cont_ana::{ContapContAna, trigonometric_roots};
use super::geometry::point_of;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Cylinder3, Direction3, Line3, Point3, Xyz};
impl ContapContAna {
    /// Perform(gp_Cylinder, gp_Dir): the two generatrices whose normal is normal to `d`;
    /// none when `d` is parallel to the axis (`|axis x d| <= 1e-15`);
    /// `contour.perform_cylinder(&c, d)?`.
    pub fn perform_cylinder(&mut self, c: &Cylinder3, d: Direction3) -> Result<(), Failure> {
        self.done = false;
        let mut normale = c.position().direction().xyz();
        normale.cross(d.xyz());
        if normale.modulus() <= 1e-15 {
            self.nb_sol = 0;
        } else {
            normale.normalize()?;
            self.typ_l = CurveType::Line;
            self.dir[0] = c.position().direction();
            self.dir[1] = self.dir[0];
            self.pt[0] = point_of(c.location().xyz() + c.radius() * normale);
            self.pt[1] = point_of(c.location().xyz() - c.radius() * normale);
            self.nb_sol = 2;
        }
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Cylinder, gp_Dir, Ang): the two generatrices of draft angle `ang`, from
    /// `Coefcos cos(t) + Coefsin sin(t) = cos(PI/2 + ang)`; none when it has no root;
    /// `contour.perform_cylinder_draft(&c, d, 0.1)?`.
    pub fn perform_cylinder_draft(
        &mut self,
        c: &Cylinder3,
        d: Direction3,
        ang: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        let equation = CylinderDraftEquation::new(c, d, ang);
        // The native test is `|Coefcst| < norm2`, so a NaN coefficient has no root.
        let has_roots = equation.coefcst.abs() < equation.norm1.sqrt();
        if has_roots {
            self.store_cylinder_draft(c, equation)?;
        } else {
            self.nb_sol = 0;
        }
        self.done = true;
        Ok(())
    }

    /// Perform(gp_Cylinder, gp_Pnt Eye): the two generatrices tangent to the planes
    /// through `eye`; none when the eye is inside the cylinder;
    /// `contour.perform_cylinder_eye(&c, eye)?`.
    pub fn perform_cylinder_eye(&mut self, c: &Cylinder3, eye: Point3) -> Result<(), Failure> {
        self.done = false;
        let radius = c.radius();
        let theaxis = Line3::from_axis(c.axis());
        let dist = theaxis.distance(eye);
        if dist <= radius {
            self.nb_sol = 0;
            self.done = true;
            return Ok(());
        }
        self.store_cylinder_eye(c, eye, theaxis, dist)?;
        self.done = true;
        Ok(())
    }
    fn store_cylinder_eye(
        &mut self,
        c: &Cylinder3,
        eye: Point3,
        theaxis: Line3,
        dist: f64,
    ) -> Result<(), Failure> {
        let radius = c.radius();
        self.typ_l = CurveType::Line;
        self.prm = radius * (1. - radius * radius / (dist * dist)).sqrt();
        self.dir[0] = c.axis().direction();
        self.dir[1] = self.dir[0];
        // The axis is oriented to the outside, towards the eye.
        let axeye = theaxis.normal(eye)?.direction().xyz();
        let normale = theaxis
            .direction()
            .crossed(Direction3::from_xyz(axeye)?)?
            .xyz();
        let foot = c.location().xyz() + (radius * radius / dist) * axeye;
        self.pt[1] = point_of(foot - self.prm * normale);
        self.pt[0] = point_of(foot + self.prm * normale);
        self.nb_sol = 2;
        Ok(())
    }
    fn store_cylinder_draft(
        &mut self,
        c: &Cylinder3,
        equation: CylinderDraftEquation,
    ) -> Result<(), Failure> {
        let CylinderDraftEquation {
            mut coefcos,
            mut coefsin,
            coefcst,
            norm1,
        } = equation;
        self.typ_l = CurveType::Line;
        self.nb_sol = 2;
        self.dir[0] = c.position().direction();
        self.dir[1] = self.dir[0];
        if !c.direct()? {
            // The normal is inverted.
            coefcos = -coefcos;
            coefsin = -coefsin;
        }
        self.prm = (norm1 - coefcst * coefcst).sqrt();
        let roots = trigonometric_roots(coefcos, coefsin, coefcst, self.prm, norm1);
        self.store_cylinder_roots(c, roots);
        Ok(())
    }
    fn store_cylinder_roots(&mut self, c: &Cylinder3, roots: [(f64, f64); 2]) {
        let position = c.position();
        let (xdir, ydir) = (position.x_direction().xyz(), position.y_direction().xyz());
        for (k, (cost, sint)) in roots.into_iter().enumerate() {
            let mut dirxyz = Xyz::default();
            dirxyz.set_linear_form(cost, xdir, sint, ydir);
            dirxyz *= c.radius();
            self.pt[k] = point_of(c.location().xyz() + dirxyz);
        }
    }
}
struct CylinderDraftEquation {
    coefcos: f64,
    coefsin: f64,
    coefcst: f64,
    norm1: f64,
}
impl CylinderDraftEquation {
    fn new(c: &Cylinder3, d: Direction3, angle: f64) -> Self {
        let position = c.position();
        let coefcos = d.dot(position.x_direction());
        let coefsin = d.dot(position.y_direction());
        Self {
            coefcos,
            coefsin,
            coefcst: (std::f64::consts::PI * 0.5 + angle).cos(),
            norm1: coefcos * coefcos + coefsin * coefsin,
        }
    }
}
