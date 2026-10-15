// SPDX-License-Identifier: GPL-2.0-only
//! Elliptic carriers (ADR 0083): `E(t) = c + a cos(t) u + b sin(t) v` with `u` the unit
//! major direction and `v` its counterclockwise normal, so counterclockwise sketch rotation
//! is increasing `t`. Every piece of an ellipse is the affine image of a circle piece, so the
//! certified rational quadratic forms of circles carry over unchanged.
use super::plane_math::length2;

/// A checked ellipse in plane coordinates; `minor <= major`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EllipseCarrier {
    pub center: [f64; 2],
    /// Unit major direction.
    pub axis: [f64; 2],
    pub major: f64,
    pub minor: f64,
}

impl EllipseCarrier {
    /// The carrier of a caller ellipse, `None` for a zero or nonfinite major axis.
    pub(crate) fn new(center: [f64; 2], major_axis: [f64; 2], minor: f64) -> Option<Self> {
        let major = length2(major_axis);
        if !(major.is_finite() && major > 0.0) {
            return None;
        }
        let axis = major_axis.map(|c| c / major);
        Some(Self {
            center,
            axis,
            major,
            minor,
        })
    }

    /// Counterclockwise unit normal of the major direction.
    pub(crate) fn minor_axis(&self) -> [f64; 2] {
        [-self.axis[1], self.axis[0]]
    }

    /// The affine image of the unit-circle point `[x, y]`: `c + a x u + b y v`.
    pub(crate) fn image(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        let v = self.minor_axis();
        std::array::from_fn(|k| {
            self.center[k] + self.major * x * self.axis[k] + self.minor * y * v[k]
        })
    }

    /// Point at parameter `t`.
    pub(crate) fn point(&self, t: f64) -> [f64; 2] {
        self.image([t.cos(), t.sin()])
    }

    /// Parameter of the point of the carrier closest in the affine (circle) sense to `p`;
    /// exact for points on the carrier.
    pub(crate) fn parameter(&self, p: [f64; 2]) -> f64 {
        let d = [p[0] - self.center[0], p[1] - self.center[1]];
        let v = self.minor_axis();
        let along = (d[0] * self.axis[0] + d[1] * self.axis[1]) / self.major;
        let across = (d[0] * v[0] + d[1] * v[1]) / self.minor;
        across.atan2(along)
    }

    /// Distance from `p` to the carrier point of its parameter: zero on the carrier and an
    /// upper bound of the distance to the carrier elsewhere.
    pub(crate) fn gap(&self, p: [f64; 2]) -> f64 {
        let q = self.point(self.parameter(p));
        length2([p[0] - q[0], p[1] - q[1]])
    }

    /// Green's-theorem term `1/2 integral (x dy - y dx)` of the arc from `start` to `end`
    /// sweeping `sweep` radians: `1/2 (c x (end - start) + a b sweep)`.
    pub(crate) fn green_term(&self, start: [f64; 2], end: [f64; 2], sweep: f64) -> f64 {
        let chord = [end[0] - start[0], end[1] - start[1]];
        let cross = self.center[0] * chord[1] - self.center[1] * chord[0];
        0.5 * (cross + self.major * self.minor * sweep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    #[test]
    fn parameter_inverts_the_point_and_the_full_turn_is_pi_a_b() {
        let carrier = EllipseCarrier::new([1., -2.], [1.2, 1.6], 0.5).unwrap();
        assert_eq!(carrier.major, 2.0);
        for t in [-2.5, -0.3, 0.0, 1.0, 3.0] {
            assert!((carrier.parameter(carrier.point(t)) - t).abs() < 1e-14);
            assert!(carrier.gap(carrier.point(t)) < 1e-14);
        }
        let start = carrier.point(0.0);
        let full = carrier.green_term(start, start, 2.0 * PI);
        assert!((full - PI * 2.0 * 0.5).abs() < 1e-14);
        let quarter = carrier.green_term(start, carrier.point(FRAC_PI_2), FRAC_PI_2);
        let half = carrier.green_term(carrier.point(FRAC_PI_2), carrier.point(PI), FRAC_PI_2);
        assert!((quarter + half - carrier.green_term(start, carrier.point(PI), PI)).abs() < 1e-14);
        assert!(EllipseCarrier::new([0., 0.], [0., 0.], 1.).is_none());
    }
}
