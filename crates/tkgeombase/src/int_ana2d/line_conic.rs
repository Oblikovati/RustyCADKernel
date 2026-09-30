// Rust translation of IntAna2d_AnaIntersection_4.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_AnaIntersection::Perform(gp_Lin2d, IntAna2d_Conic).
use super::{AnaIntersection, Conic, MyDirectPolynomialRoots, traitement_points_confondus};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Line2;

impl AnaIntersection {
    /// Perform(gp_Lin2d, IntAna2d_Conic): the roots of the conic equation along
    /// the line `X = X0 - L B, Y = Y0 + L A` (A, B, C the line coefficients),
    /// without the coincident points; identical elements when every coefficient
    /// vanishes. The points are implicit (no parameter on the conic).
    /// `inter.perform_line_conic(line, &conic)?`.
    pub fn perform_line_conic(&mut self, line: Line2, conic: &Conic) -> Result<(), Failure> {
        self.done = false;
        self.nbp = 0;
        self.para = false;
        self.iden = false;
        let [a, b, c, d, e, f] = conic.coefficients();
        let (dr_a, dr_b, _) = line.coefficients();
        let (x0, y0) = (line.location().x(), line.location().y());
        let px0: f64 = f + x0 * (d + d + a * x0 + 2.0 * c * y0) + y0 * (e + e + b * y0);
        let px1: f64 =
            2.0 * (e * dr_a - d * dr_b + x0 * (c * dr_a - a * dr_b) + y0 * (b * dr_a - c * dr_b));
        let px2: f64 = dr_a * (b * dr_a - 2.0 * c * dr_b) + a * (dr_b * dr_b);
        let roots = MyDirectPolynomialRoots::quadratic(px2, px1, px0)?;
        if !roots.is_done() {
            self.done = false;
            return Ok(());
        }
        if roots.infinite_roots() {
            self.iden = true;
            self.done = true;
            return Ok(());
        }
        self.nbp = roots.nb_solutions();
        for i in 1..=self.nbp {
            let s: f64 = roots.value(i)?;
            let (tx, ty) = (x0 - s * dr_b, y0 + s * dr_a);
            self.lpnt[(i - 1) as usize].set_implicit_value(tx, ty, s);
        }
        self.nbp = traitement_points_confondus(self.nbp, &mut self.lpnt);
        self.done = true;
        Ok(())
    }
}
