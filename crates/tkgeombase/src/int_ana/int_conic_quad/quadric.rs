// Rust translation of the conic-quadric Perform overloads of IntAna_IntConicQuad.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_IntConicQuad::Perform(conic, IntAna_Quadric): the quadric equation
//! along the conic parametrization, a polynomial in t, in (cos t, sin t) or in
//! Exp(t).
use super::IntConicQuad;
use crate::int_ana::Quadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis3, Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3};
use rustycad_tkmath::math::{DirectPolynomialRoots, TrigonometricRoots};
use std::f64::consts::PI;

impl IntConicQuad {
    /// Perform(gp_Lin, IntAna_Quadric): the roots of the quadratic in t of the
    /// quadric along `Location + t Direction`; in the quadric for an identity.
    /// `inter.perform_line_quadric(line, &quadric)?`.
    pub fn perform_line_quadric(&mut self, line: Line3, quadric: &Quadric) -> Result<(), Failure> {
        self.done = false;
        self.inquadric = false;
        self.parallel = false;
        self.nbpts = 0;
        let [lx, ly, lz] = *line.direction().coordinates();
        let [lx0, ly0, lz0] = *line.location().coordinates();
        let [qxx, qyy, qzz, qxy, qxz, qyz, qx, qy, qz, qcte] = quadric.coefficients();
        let a0: f64 = qcte
            + qxx * lx0 * lx0
            + qyy * ly0 * ly0
            + qzz * lz0 * lz0
            + 2.0 * (lx0 * (qx + qxy * ly0 + qxz * lz0) + ly0 * (qy + qyz * lz0) + qz * lz0);
        let a1: f64 = 2.0
            * (lx * (qx + qxx * lx0 + qxy * ly0 + qxz * lz0)
                + ly * (qy + qxy * lx0 + qyy * ly0 + qyz * lz0)
                + lz * (qz + qxz * lx0 + qyz * ly0 + qzz * lz0));
        let a2: f64 = qxx * lx * lx
            + qyy * ly * ly
            + qzz * lz * lz
            + 2.0 * (lx * (qxy * ly + qxz * lz) + qyz * ly * lz);
        let roots = DirectPolynomialRoots::quadratic(a2, a1, a0);
        self.store_polynomial_roots(&roots, |t| {
            Some((t, Point3::new(lx0 + lx * t, ly0 + ly * t, lz0 + lz * t)))
        })
    }

    /// Perform(gp_Circ, IntAna_Quadric): the roots in `[0, 2 PI]` of the quadric
    /// along `R cos t, R sin t` in the circle frame.
    /// `inter.perform_circle_quadric(circle, &quadric)?`.
    pub fn perform_circle_quadric(
        &mut self,
        circle: Circle3,
        quadric: &Quadric,
    ) -> Result<(), Failure> {
        self.reset_flags();
        let frame: Axis3 = Axis3::from_axis2(circle.position());
        let [qxx, qyy, _, qxy, _, _, qx, qy, _, qcte] =
            quadric.new_coefficients(quadric.coefficients(), frame)?;
        let r: f64 = circle.radius();
        let rr: f64 = r * r;
        let (p_cos_cos, p_sin_sin, p_sin, p_cos, p_cos_sin) =
            (rr * qxx, rr * qyy, r * qy, r * qx, rr * qxy);
        let roots = TrigonometricRoots::new(
            p_cos_cos - p_sin_sin,
            p_cos_sin,
            p_cos + p_cos,
            p_sin + p_sin,
            qcte + p_sin_sin,
            0.0,
            PI + PI,
        );
        let position = circle.position();
        self.store_trigonometric_roots(&roots, |t| elclib::circle_value(t, position, r))
    }

    /// Perform(gp_Elips, IntAna_Quadric): as the circle along
    /// `MajorRadius cos t, MinorRadius sin t`.
    /// `inter.perform_ellipse_quadric(ellipse, &quadric)?`.
    pub fn perform_ellipse_quadric(
        &mut self,
        ellipse: Ellipse3,
        quadric: &Quadric,
    ) -> Result<(), Failure> {
        self.reset_flags();
        let frame: Axis3 = Axis3::from_axis2(ellipse.position());
        let [qxx, qyy, _, qxy, _, _, qx, qy, _, qcte] =
            quadric.new_coefficients(quadric.coefficients(), frame)?;
        let (major, minor) = (ellipse.major_radius(), ellipse.minor_radius());
        let (p_cos_cos, p_sin_sin) = (major * major * qxx, minor * minor * qyy);
        let (p_sin, p_cos, p_cos_sin) = (minor * qy, major * qx, major * minor * qxy);
        let roots = TrigonometricRoots::new(
            p_cos_cos - p_sin_sin,
            p_cos_sin,
            p_cos + p_cos,
            p_sin + p_sin,
            qcte + p_sin_sin,
            0.0,
            PI + PI,
        );
        let position = ellipse.position();
        self.store_trigonometric_roots(&roots, |t| elclib::ellipse_value(t, position, major, minor))
    }

    /// Perform(gp_Parab, IntAna_Quadric): the roots of the quartic in y of the
    /// quadric along `x = y² / (4 Focal)`, `y` in the parabola frame.
    /// `inter.perform_parabola_quadric(parabola, &quadric)?`.
    pub fn perform_parabola_quadric(
        &mut self,
        parabola: Parabola3,
        quadric: &Quadric,
    ) -> Result<(), Failure> {
        self.reset_flags();
        let frame: Axis3 = Axis3::from_axis2(parabola.position());
        let [qxx, qyy, _, qxy, _, _, qx, qy, _, qcte] =
            quadric.new_coefficients(quadric.coefficients(), frame)?;
        let f: f64 = parabola.focal();
        let un_sur_2p: f64 = 0.25 / f;
        let roots = DirectPolynomialRoots::quartic(
            qxx * un_sur_2p * un_sur_2p,
            (qxy + qxy) * un_sur_2p,
            qyy + (qx + qx) * un_sur_2p,
            qy + qy,
            qcte,
        );
        let position = parabola.position();
        self.store_polynomial_roots(&roots, |t| {
            Some((t, elclib::parabola_value(t, position, f)))
        })
    }

    /// Perform(gp_Hypr, IntAna_Quadric): the roots of the quartic in `Exp(t)` of
    /// the quadric along `R Ch t, r Sh t`; only the roots above RealEpsilon() give
    /// points, of parameter `Log(root)`.
    /// `inter.perform_hyperbola_quadric(hyperbola, &quadric)?`.
    pub fn perform_hyperbola_quadric(
        &mut self,
        hyperbola: Hyperbola3,
        quadric: &Quadric,
    ) -> Result<(), Failure> {
        self.reset_flags();
        let frame: Axis3 = Axis3::from_axis2(hyperbola.position());
        let [qxx, qyy, _, qxy, _, _, qx, qy, _, qcte] =
            quadric.new_coefficients(quadric.coefficients(), frame)?;
        let (big, small) = (hyperbola.major_radius(), hyperbola.minor_radius());
        let (rr_big, rr_small, rr_mixed) = (big * big, small * small, big * small);
        let roots = DirectPolynomialRoots::quartic(
            rr_big * qxx + rr_mixed * (qxy + qxy) + rr_small * qyy,
            4.0 * (big * qx + small * qy),
            2.0 * ((qcte + qcte) + qxx * rr_big - qyy * rr_small),
            4.0 * (big * qx - small * qy),
            qxx * rr_big - rr_mixed * (qxy + qxy) + qyy * rr_small,
        );
        let position = hyperbola.position();
        self.store_polynomial_roots(&roots, |t| {
            if t < f64::EPSILON || t.is_nan() {
                return None;
            }
            let parameter: f64 = t.ln();
            Some((
                parameter,
                elclib::hyperbola_value(parameter, position, big, small),
            ))
        })
    }

    /// The flags every conic Perform clears (the point count is kept, as natively).
    fn reset_flags(&mut self) {
        self.done = false;
        self.inquadric = false;
        self.parallel = false;
    }

    /// Stores the points of the polynomial roots that `point_of` accepts; in the
    /// quadric for an identity, not done when the solver fails.
    fn store_polynomial_roots(
        &mut self,
        roots: &DirectPolynomialRoots,
        point_of: impl Fn(f64) -> Option<(f64, Point3)>,
    ) -> Result<(), Failure> {
        if !roots.is_done() {
            return Ok(());
        }
        self.done = true;
        if roots.infinite_roots() {
            self.inquadric = true;
            return Ok(());
        }
        let count: i32 = roots.solutions()?;
        self.nbpts = count;
        let mut kept: usize = 0;
        for index in 1..=count {
            if let Some((parameter, point)) = point_of(roots.value(index)?) {
                self.paramonc[kept] = parameter;
                self.pnts[kept] = point;
                kept += 1;
            }
        }
        self.nbpts = kept as i32;
        Ok(())
    }

    /// Stores the points of the trigonometric roots; in the quadric for an
    /// identity, not done when the solver fails.
    fn store_trigonometric_roots(
        &mut self,
        roots: &TrigonometricRoots,
        point_of: impl Fn(f64) -> Point3,
    ) -> Result<(), Failure> {
        if !roots.is_done() {
            return Ok(());
        }
        self.done = true;
        if roots.infinite_roots() {
            self.inquadric = true;
            return Ok(());
        }
        self.nbpts = roots.solutions()?;
        for index in 1..=self.nbpts {
            let t: f64 = roots.value(index)?;
            self.paramonc[(index - 1) as usize] = t;
            self.pnts[(index - 1) as usize] = point_of(t);
        }
        Ok(())
    }
}
