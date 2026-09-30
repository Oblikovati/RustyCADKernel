// Rust translation of IntAna2d_AnaIntersection_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_AnaIntersection::Perform(gp_Lin2d, gp_Lin2d).
use super::AnaIntersection;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::gp::Line2;

/// One row `(alpha, beta, gamma)` of the pivoted linear system.
type Row = (f64, f64, f64);

impl AnaIntersection {
    /// Perform(gp_Lin2d, gp_Lin2d): one point, parallel lines (empty) or identical
    /// lines (no point, not empty), decided against RealEpsilon(); always done.
    /// The point carries the parameters on both lines. `inter.perform_lines(l1, l2)`.
    pub fn perform_lines(&mut self, first: Line2, second: Line2) {
        self.done = false;
        let (a1, b1, c1) = first.coefficients();
        let (a2, b2, c2) = second.coefficients();
        // std::max, not f64::max: the pivot choice follows the native comparisons.
        let det: f64 = std_max(a1.abs(), std_max(a2.abs(), std_max(b1.abs(), b2.abs())));
        let ((al1, be1, ga1), (al2, be2, ga2)) = pivot_rows(det, (a1, b1, c1), (a2, b2, c2));
        let rap: f64 = al2 / al1;
        let denom: f64 = be2 - rap * be1;
        if denom.abs() <= f64::EPSILON {
            self.para = true;
            self.nbp = 0;
            self.iden = (ga2 - rap * ga1).abs() <= f64::EPSILON;
            self.empt = !self.iden;
        } else {
            self.para = false;
            self.iden = false;
            self.empt = false;
            self.nbp = 1;
            let xs: f64 = (be1 * ga2 / al1 - be2 * ga1 / al1) / denom;
            let ys: f64 = (rap * ga1 - ga2) / denom;
            // The solution is in the pivoted variables: undo the exchange of X and Y.
            let exchanged: bool = (a1.abs() != det && b1.abs() == det)
                || (a1.abs() != det && b1.abs() != det && a2.abs() != det);
            let (xs, ys) = if exchanged { (ys, xs) } else { (xs, ys) };
            let la: f64 = line_parameter(first, (a1, b1), (xs, ys));
            let mu: f64 = line_parameter(second, (a2, b2), (xs, ys));
            self.lpnt[0].set_value(xs, ys, la, mu);
        }
        self.done = true;
    }
}

/// The row and column exchange: the coefficient of largest magnitude `det`
/// becomes the pivot `al1`.
fn pivot_rows(det: f64, first: Row, second: Row) -> (Row, Row) {
    let ((a1, b1, c1), (a2, b2, c2)) = (first, second);
    if a1.abs() == det {
        ((a1, b1, c1), (a2, b2, c2))
    } else if b1.abs() == det {
        ((b1, a1, c1), (b2, a2, c2))
    } else if a2.abs() == det {
        ((a2, b2, c2), (a1, b1, c1))
    } else {
        ((b2, a2, c2), (b1, a1, c1))
    }
}

/// The parameter of the point `(xs, ys)` on `line` of coefficients `(a, b)`,
/// divided by the coefficient of larger magnitude.
fn line_parameter(line: Line2, (a, b): (f64, f64), (xs, ys): (f64, f64)) -> f64 {
    if a.abs() >= b.abs() {
        (ys - line.location().y()) / a
    } else {
        (line.location().x() - xs) / b
    }
}
