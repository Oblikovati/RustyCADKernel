// Rust translation of IntAna2d_AnaIntersection_2.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_AnaIntersection::Perform(gp_Circ2d, gp_Circ2d).
use super::AnaIntersection;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::gp::{Circle2, Vector2};
use std::f64::consts::PI;

impl AnaIntersection {
    /// Perform(gp_Circ2d, gp_Circ2d): concentric (parallel or identical),
    /// exterior, externally tangent, secant, internally tangent or nested circles,
    /// decided against Epsilon() of the radii sum. The points carry the angular
    /// parameters on both circles in `[0, 2 PI)`. `inter.perform_circles(c1, c2)?`.
    pub fn perform_circles(&mut self, first: Circle2, second: Circle2) -> Result<(), Failure> {
        self.done = false;
        let d: f64 = first.location().distance(second.location());
        let (r1, r2) = (first.radius(), second.radius());
        let sum: f64 = r1 + r2;
        let dif: f64 = (r1 - r2).abs();
        if d <= f64::EPSILON {
            self.set_counts(true, dif <= f64::EPSILON, 0);
        } else if (d - sum) > epsilon(sum) {
            self.set_counts(false, false, 0);
        } else if (d - sum).abs() <= epsilon(sum) {
            self.set_counts(false, false, 1);
            self.external_tangency(first, second)?;
        } else if ((sum - d) > epsilon(sum)) && ((d - dif) > epsilon(d + dif)) {
            self.set_counts(false, false, 2);
            self.two_points(first, second, d)?;
        } else if (d - dif).abs() <= epsilon(sum) {
            self.set_counts(false, false, 1);
            self.internal_tangency(first, second)?;
        } else {
            // d < dif - Resol and d != 0: one circle inside the other.
            self.set_counts(false, false, 0);
        }
        self.done = true;
        Ok(())
    }

    /// The flags of a circle-circle result: empty when there is no point and the
    /// circles are not identical.
    fn set_counts(&mut self, parallel: bool, identical: bool, count: i32) {
        self.para = parallel;
        self.iden = identical;
        self.nbp = count;
        self.empt = count == 0 && !identical;
    }

    /// The tangency point of externally tangent circles, weighted by the radii.
    fn external_tangency(&mut self, first: Circle2, second: Circle2) -> Result<(), Failure> {
        let (r1, r2) = (first.radius(), second.radius());
        let sum: f64 = r1 + r2;
        let (o1, o2) = (first.location(), second.location());
        let ax: Vector2 = Vector2::between(o1, o2);
        let xs: f64 = (o1.x() * r2 + o2.x() * r1) / sum;
        let ys: f64 = (o1.y() * r2 + o2.y() * r1) / sum;
        let ang1: f64 = positive_angle(x_vector(first).angle(ax)?);
        let ang2: f64 = x_vector(second).angle(ax)? + PI;
        self.lpnt[0].set_value(xs, ys, ang1, ang2);
        Ok(())
    }

    /// The two points of secant circles, at distance `l1` along the axis of the
    /// centers and `h` across it; the angles take arccos between PI/4 and 3PI/4,
    /// arcsin otherwise.
    fn two_points(&mut self, first: Circle2, second: Circle2, d: f64) -> Result<(), Failure> {
        let (r1, r2) = (first.radius(), second.radius());
        let o1 = first.location();
        let ax: Vector2 = Vector2::between(o1, second.location());
        let ref1: f64 = x_vector(first).angle(ax)?;
        let ref2: f64 = x_vector(second).angle(ax)?;
        let mut l1: f64 = (d * d + r1 * r1 - r2 * r2) / (2.0 * d);
        if r1 * r1 - l1 * l1 < 0. {
            l1 = if l1 > 0.0 { r1 } else { -r1 };
        }
        let h: f64 = (r1 * r1 - l1 * l1).sqrt();
        let xs1: f64 = o1.x() + l1 * ax.x() / d - h * ax.y() / d;
        let ys1: f64 = o1.y() + l1 * ax.y() / d + h * ax.x() / d;
        let xs2: f64 = o1.x() + l1 * ax.x() / d + h * ax.y() / d;
        let ys2: f64 = o1.y() + l1 * ax.y() / d - h * ax.x() / d;
        let ang1: f64 = positive_sine_angle(l1 / r1, h / r1);
        let ang2: f64 = positive_sine_angle((l1 - d) / r2, h / r2);
        self.lpnt[0].set_value(xs1, ys1, wrap_angle(ref1 + ang1), wrap_angle(ref2 + ang2));
        self.lpnt[1].set_value(xs2, ys2, wrap_angle(ref1 - ang1), wrap_angle(ref2 - ang2));
        Ok(())
    }

    /// The tangency point of internally tangent circles; the axis of the centers
    /// points from the larger circle to the smaller one.
    fn internal_tangency(&mut self, first: Circle2, second: Circle2) -> Result<(), Failure> {
        let (r1, r2) = (first.radius(), second.radius());
        let (o1, o2) = (first.location(), second.location());
        let mut ax: Vector2 = Vector2::between(o1, o2);
        if first.radius() < second.radius() {
            ax.reverse();
        }
        let ang1: f64 = positive_angle(x_vector(first).angle(ax)?);
        let ang2: f64 = positive_angle(x_vector(second).angle(ax)?);
        let xs: f64 = (o1.x() * r2 - o2.x() * r1) / (r2 - r1);
        let ys: f64 = (o1.y() * r2 - o2.y() * r1) / (r2 - r1);
        self.lpnt[0].set_value(xs, ys, ang1, ang2);
        Ok(())
    }
}

/// `gp_Vec2d(C.XAxis().Direction())`.
fn x_vector(circle: Circle2) -> Vector2 {
    Vector2::from(circle.x_axis().direction())
}

/// An angle of `(-PI, PI]` brought into `[0, 2 PI)` by adding 2 PI when negative.
fn positive_angle(angle: f64) -> f64 {
    if angle < 0.0 { 2.0 * PI + angle } else { angle }
}

/// An angle brought into `[0, 2 PI)` by one period in either direction.
fn wrap_angle(angle: f64) -> f64 {
    if angle < 0. {
        2.0 * PI + angle
    } else if angle >= 2.0 * PI {
        angle - 2.0 * PI
    } else {
        angle
    }
}

/// The angle of positive sine of cosine `cost` and sine `sint`: arccos when
/// `|cost| <= 0.707`, else arcsin, mirrored to `PI - angle` for a negative cosine.
fn positive_sine_angle(cost: f64, sint: f64) -> f64 {
    if cost.abs() <= 0.707 {
        return cost.acos();
    }
    let angle: f64 = sint.asin();
    if cost < 0.0 { PI - angle } else { angle }
}
