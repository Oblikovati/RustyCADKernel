// SPDX-License-Identifier: GPL-2.0-only
//! Exact implicit equations of lines and conics, composed with homogeneous pieces.
//!
//! A line through `A` with direction `d` is `det(d, p - A) = 0`. A conic is
//! `Q(p - C) - c = 0` with a positive definite quadratic form `Q`: `|q|^2 - r^2`
//! for circles and `rb^2 (a.q)^2 + ra^2 det(a,q)^2 - ra^2 rb^2 |a|^2` for ellipses
//! with any nonzero major-axis vector `a`. Composing with `(x, y) / w` and clearing
//! `w` gives the exact contact polynomial of a piece; the sign of a composed conic
//! is the sign of `F` because the even power of `w` is positive.
use super::super::certificate_arithmetic::ExactRatio;
use super::rational_polynomial::{Ring, ratio};
use super::spans::HomogeneousPoint;

/// Symmetric quadratic form `xx qx^2 + 2 xy qx qy + yy qy^2`.
#[derive(Clone, Debug)]
pub(super) struct QuadraticForm {
    pub xx: ExactRatio,
    pub xy: ExactRatio,
    pub yy: ExactRatio,
}
/// Positive definite conic `Q(p - center) = constant`.
#[derive(Clone, Debug)]
pub(super) struct ConicImplicit {
    pub center: [ExactRatio; 2],
    pub form: QuadraticForm,
    pub constant: ExactRatio,
}
/// Line `det(direction, p - origin) = 0`.
#[derive(Clone, Debug)]
pub(super) struct LineImplicit {
    pub origin: [ExactRatio; 2],
    pub direction: [ExactRatio; 2],
}
impl QuadraticForm {
    /// The form applied to polynomial (or surd) coordinates.
    pub fn compose<T: Ring>(&self, vector: &[T; 2]) -> T {
        let [x, y] = vector;
        let cross = x.multiply(y).scale(&self.xy);
        x.multiply(x)
            .scale(&self.xx)
            .add(&cross)
            .add(&cross)
            .add(&y.multiply(y).scale(&self.yy))
    }
    /// Exact value at a rational vector.
    pub fn value(&self, vector: &[ExactRatio; 2]) -> ExactRatio {
        let applied = self.apply(vector);
        applied[0]
            .multiply(&vector[0])
            .add(&applied[1].multiply(&vector[1]))
    }
    /// Matrix-vector product `M q`, half the gradient of the form.
    pub fn apply(&self, vector: &[ExactRatio; 2]) -> [ExactRatio; 2] {
        let [x, y] = vector;
        [
            self.xx.multiply(x).add(&self.xy.multiply(y)),
            self.xy.multiply(x).add(&self.yy.multiply(y)),
        ]
    }
}
impl ConicImplicit {
    /// Circle of the given center and positive radius.
    pub fn circle(center: [f64; 2], radius: f64) -> Self {
        Self::circle_exact(center, ratio(radius))
    }
    /// Circle with an exact rational radius, e.g. a circle offset `r + d`.
    pub fn circle_exact(center: [f64; 2], radius: ExactRatio) -> Self {
        Self {
            center: center.map(ratio),
            form: QuadraticForm {
                xx: ratio(1.),
                xy: ratio(0.),
                yy: ratio(1.),
            },
            constant: radius.multiply(&radius),
        }
    }
    /// Ellipse with a nonzero major-axis vector and positive radii.
    pub fn ellipse(center: [f64; 2], axis: [f64; 2], radii: [f64; 2]) -> Self {
        let [ax, ay] = axis.map(ratio);
        let major = ratio(radii[0]).multiply(&ratio(radii[0]));
        let minor = ratio(radii[1]).multiply(&ratio(radii[1]));
        let (xx_axis, yy_axis, xy_axis) = (ax.multiply(&ax), ay.multiply(&ay), ax.multiply(&ay));
        let length = xx_axis.add(&yy_axis);
        Self {
            center: center.map(ratio),
            form: QuadraticForm {
                xx: minor.multiply(&xx_axis).add(&major.multiply(&yy_axis)),
                xy: minor.subtract(&major).multiply(&xy_axis),
                yy: minor.multiply(&yy_axis).add(&major.multiply(&xx_axis)),
            },
            constant: major.multiply(&minor).multiply(&length),
        }
    }
    /// `w^2 F((x, y) / w)`, exact in the piece variable.
    pub fn compose<T: Ring>(&self, point: &HomogeneousPoint<T>) -> T {
        let relative = point.relative(&self.center);
        let constant = point.w.multiply(&point.w).scale(&self.constant.negated());
        self.form.compose(&relative).add(&constant)
    }
}
impl LineImplicit {
    /// Line through two distinct binary64 points.
    pub fn through(points: [[f64; 2]; 2]) -> Self {
        let origin = points[0].map(ratio);
        let direction = std::array::from_fn(|axis| ratio(points[1][axis]).subtract(&origin[axis]));
        Self { origin, direction }
    }
    /// `w det(d, (x, y)/w - A)`, exact in the piece variable.
    pub fn compose<T: Ring>(&self, point: &HomogeneousPoint<T>) -> T {
        let [x, y] = point.relative(&self.origin);
        let [dx, dy] = &self.direction;
        y.scale(dx).add(&x.scale(&dy.negated()))
    }
    /// `w d.((x, y)/w - A)`, the unnormalized longitudinal coordinate.
    pub fn along<T: Ring>(&self, point: &HomogeneousPoint<T>) -> T {
        let [x, y] = point.relative(&self.origin);
        let [dx, dy] = &self.direction;
        x.scale(dx).add(&y.scale(dy))
    }
    /// `|d|^2`.
    pub fn squared_length(&self) -> ExactRatio {
        let [dx, dy] = &self.direction;
        dx.multiply(dx).add(&dy.multiply(dy))
    }
    /// Exact parameter `d.(p - A) / |d|^2` of the orthogonal projection of a point.
    pub fn projected_parameter(&self, point: [f64; 2]) -> ExactRatio {
        let [dx, dy] = &self.direction;
        let [ox, oy] = &self.origin;
        let offset = [ratio(point[0]).subtract(ox), ratio(point[1]).subtract(oy)];
        dx.multiply(&offset[0])
            .add(&dy.multiply(&offset[1]))
            .divide(&self.squared_length())
    }
}
