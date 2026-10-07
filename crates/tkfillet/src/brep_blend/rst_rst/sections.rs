//! The sections of the restriction-restriction fillets: the circle at a parameter and the
//! poles at a point of a blend line, without and with their derivatives.
use super::RstRstCore;
use crate::blend::{BlendPoint, SectionPoles};
use crate::blend_func::SectionShape;
use crate::blend_func::choice::live_law;
use crate::blend_func::linalg::{matrix, vector, vset};
use crate::blend_func::package::live_curve;
use crate::brep_blend::live_curve2d_of;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::geom_fill::{CircleD0, get_circle};
use rustycad_tkgeombase::approx::SweepD1;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3, Vector3};
use std::f64::consts::PI;

fn frame(origin: Point3, np: Vector3, x: Vector3) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(
        origin,
        Direction3::from_vector(np)?,
        Direction3::from_vector(x)?,
    )
}

/// The circle data of the first-order section.
struct RstRstCircle {
    center: Point3,
    med: Vector3,
    normmed: f64,
    n1: Vector3,
    n2: Vector3,
}

impl RstRstCore {
    /// `Section(Param, U, V, Pdeb, Pfin, C)`: the circular section at `param` through the
    /// restriction points (centered at the origin when there is no center, as natively).
    pub(crate) fn section_circle(
        &mut self,
        param: f64,
        (u, v): (f64, f64),
    ) -> Result<(f64, f64, Circle3), Failure> {
        (self.ptgui, self.d1gui) = live_curve(&self.tguide)?.d1(param)?;
        self.law_radius(param)?;
        let mut np = self.d1gui.normalized()?;
        self.ptrst1 = self.r1.cons.value(u)?;
        self.ptrst2 = self.r2.cons.value(v)?;
        let (_, center, _) = self.center_circle_rst1_rst2(self.ptrst1, self.ptrst2, np)?;
        let radius = self.ray.abs();
        let ns = Vector3::between(center, self.ptrst1).normalized()?;
        if self.choix % 2 != 0 {
            np.reverse();
        }
        let mut circle = Circle3::new(frame(center, np, ns)?, radius)?;
        let mut pfin = elclib::circle_parameter(circle.position(), self.ptrst2);
        // Test of angles negative and almost null: special case.
        if pfin > 1.5 * PI {
            np.reverse();
            circle = Circle3::new(frame(center, np, ns)?, radius)?;
            pfin = elclib::circle_parameter(circle.position(), self.ptrst2);
        }
        if pfin < precision::p_confusion() {
            pfin += precision::p_confusion();
        }
        Ok((0., pfin, circle))
    }

    fn circle(&self, n1: Vector3, n2: Vector3, center: Point3) -> CircleD0 {
        CircleD0 {
            ns1: n1,
            ns2: n2,
            nplan: self.nplan,
            pts1: self.ptrst1,
            pts2: self.ptrst2,
            rayon: self.ray.abs(),
            center,
        }
    }

    fn linear_poles(
        &self,
        poles: &mut Array1<'_, Point3>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let (low, upp) = (poles.lower(), poles.upper());
        put(poles, low, self.ptrst1)?;
        put(poles, upp, self.ptrst2)?;
        put(weights, low, 1.0)?;
        put(weights, upp, 1.0)
    }

    /// `Section(P, Poles, Poles2d, Weigths)`: a circular section reverses the member plane
    /// normal for an odd choice, as natively.
    pub(crate) fn section_d0(
        &mut self,
        point: &BlendPoint,
        values: SectionPoles<'_, '_>,
    ) -> Result<(), Failure> {
        let prm = point.parameter();
        (self.ptgui, self.d1gui) = live_curve(&self.tguide)?.d1(prm)?;
        self.law_radius(prm)?;
        self.nplan = self.d1gui.normalized()?;
        let (u, v) = (point.parameter_on_c1()?, point.parameter_on_c2()?);
        let pt2d1 = live_curve2d_of(&self.r1.rst, self.class)?.value(u)?;
        let pt2d2 = live_curve2d_of(&self.r2.rst, self.class)?.value(v)?;
        self.ptrst1 = self.r1.cons.value(u)?;
        self.ptrst2 = self.r2.cons.value(v)?;
        self.distmin = std_min(self.distmin, self.ptrst1.distance(&self.ptrst2));
        let (low2d, upp2d) = (values.poles2d.lower(), values.poles2d.upper());
        put(values.poles2d, low2d, pt2d1)?;
        put(values.poles2d, upp2d, pt2d2)?;
        if self.shape == SectionShape::Linear {
            return self.linear_poles(values.poles, values.weights);
        }
        let (_, center, _) = self.center_circle_rst1_rst2(self.ptrst1, self.ptrst2, self.nplan)?;
        let ns = Vector3::between(center, self.ptrst1).normalized()?;
        let ns2 = Vector3::between(center, self.ptrst2).normalized()?;
        if self.choix % 2 != 0 {
            self.nplan.reverse();
        }
        get_circle(
            self.tconv,
            &self.circle(ns, ns2, center),
            values.poles,
            values.weights,
        )
    }

    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: false on a
    /// tangency point, and without writing anything when the points have no center.
    pub(crate) fn section_d1(
        &mut self,
        point: &BlendPoint,
        out: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        let prm = point.parameter();
        (self.ptgui, self.d1gui, self.d2gui) = live_curve(&self.tguide)?.d2(prm)?;
        let dray = self.law_d1(prm)?;
        self.normtg = self.d1gui.magnitude();
        self.nplan = self.d1gui.normalized()?;
        let mut dnplan = self.dnplan();
        self.prmrst1 = point.parameter_on_c1()?;
        self.prmrst2 = point.parameter_on_c2()?;
        self.pt2drst1 = live_curve2d_of(&self.r1.rst, self.class)?.value(self.prmrst1)?;
        self.pt2drst2 = live_curve2d_of(&self.r2.rst, self.class)?.value(self.prmrst2)?;
        let mut sol = vector(2)?;
        vset(&mut sol, 1, self.prmrst1)?;
        vset(&mut sol, 2, self.prmrst2)?;
        let (mut valsol, mut gradsol) = (vector(2)?, matrix(2, 2)?);
        self.values(&sol, &mut valsol, &mut gradsol)?;
        let (p1, d11) = self.r1.cons.d1(self.prmrst1)?;
        let (p2, d21) = self.r2.cons.d1(self.prmrst2)?;
        (self.ptrst1, self.ptrst2) = (p1, p2);
        let secmember = self.guide_second_member(dnplan)?;
        let solved = self.solve_guide(&gradsol, secmember, (1.0e-9, 2))?;
        let (is_center, center, med) =
            self.center_circle_rst1_rst2(self.ptrst1, self.ptrst2, self.nplan)?;
        if !is_center {
            return Ok(false);
        }
        let circle = self.section_circle_data(center, med)?;
        let derivatives = match solved {
            Some(s) => Some(self.normal_derivatives(&circle, s, (d11, d21), (dnplan, dray))?),
            None => None,
        };
        self.section_d1_output(out, circle, derivatives, (&mut dnplan, dray))
    }

    /// `tevol->D1(prm)` (the radius and its derivative), or 0 without a law.
    fn law_d1(&mut self, prm: f64) -> Result<f64, Failure> {
        let Some(law) = &self.law else {
            return Ok(0.);
        };
        let (ray, dray) = live_law(&law.tevol, self.class)?.d1(prm)?;
        self.ray = ray;
        Ok(dray)
    }

    /// The center, the normalized bisector and the normals from the center (normalized
    /// here for the constant radius, after their derivatives with a law, as natively).
    fn section_circle_data(
        &self,
        center: Point3,
        mut med: Vector3,
    ) -> Result<RstRstCircle, Failure> {
        let normmed = med.magnitude();
        med = med.normalized()?;
        let (mut n1, mut n2) = (
            Vector3::between(center, self.ptrst1),
            Vector3::between(center, self.ptrst2),
        );
        if self.law.is_none() {
            n1 = n1.normalized()?;
            n2 = n2.normalized()?;
        }
        Ok(RstRstCircle {
            center,
            med,
            normmed,
            n1,
            n2,
        })
    }
}

#[path = "sections_d1.rs"]
mod d1;
