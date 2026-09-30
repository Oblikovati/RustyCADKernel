// Rust translation of IntAna2d_Conic.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_Conic: a conic known by its implicit quadratic equation.
use rustycad_tkmath::gp::{Axis2d, Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Xy};

/// IntAna2d_Conic: the conic `A.X**2 + B.Y**2 + 2.C.X*Y + 2.D.X + 2.E.Y + F = 0`.
///
/// `let conic = Conic::from_line(line); let f: f64 = conic.value(0.0, 0.0);`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Conic {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Conic {
    /// The conic of gp coefficients `[A, B, C, D, E, F]`.
    fn from_coefficients([a, b, c, d, e, f]: [f64; 6]) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// IntAna2d_Conic(gp_Circ2d). `Conic::from_circle(circle)`.
    pub fn from_circle(circle: Circle2) -> Self {
        Self::from_coefficients(circle.coefficients())
    }

    /// IntAna2d_Conic(gp_Lin2d): the line `A x + B y + C = 0` written as
    /// `2 A x + 2 B y + 2 C = 0`, so D = A, E = B and F = 2 C. `Conic::from_line(line)`.
    pub fn from_line(line: Line2) -> Self {
        let (d, e, f) = line.coefficients();
        Self {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d,
            e,
            f: 2.0 * f,
        }
    }

    /// IntAna2d_Conic(gp_Parab2d). `Conic::from_parabola(parabola)`.
    pub fn from_parabola(parabola: Parabola2) -> Self {
        Self::from_coefficients(parabola.coefficients())
    }

    /// IntAna2d_Conic(gp_Hypr2d). `Conic::from_hyperbola(hyperbola)`.
    pub fn from_hyperbola(hyperbola: Hyperbola2) -> Self {
        Self::from_coefficients(hyperbola.coefficients())
    }

    /// IntAna2d_Conic(gp_Elips2d). `Conic::from_ellipse(ellipse)`.
    pub fn from_ellipse(ellipse: Ellipse2) -> Self {
        Self::from_coefficients(ellipse.coefficients())
    }

    /// Value(X, Y): the value of the implicit function at `(x, y)`.
    /// `conic.value(1.0, 0.0)`.
    pub fn value(&self, x: f64, y: f64) -> f64 {
        let [a, b, c, d, e, f] = self.coefficients();
        a * x * x + b * y * y + 2. * c * x * y + 2. * d * x + 2. * e * y + f
    }

    /// Grad(X, Y): the gradient of the implicit function at `(x, y)`.
    /// `conic.grad(1.0, 0.0).x()`.
    pub fn grad(&self, x: f64, y: f64) -> Xy {
        let [a, b, c, d, e, _] = self.coefficients();
        Xy::new(
            2. * a * x + 2. * c * y + 2. * d,
            2. * b * y + 2. * c * x + 2. * e,
        )
    }

    /// ValAndGrad(X, Y, Val, Grd): the value and the gradient at `(x, y)`, with the
    /// same operation order as [`Self::value`] and [`Self::grad`].
    /// `let (value, gradient) = conic.val_and_grad(1.0, 0.0);`
    pub fn val_and_grad(&self, x: f64, y: f64) -> (f64, Xy) {
        (self.value(x, y), self.grad(x, y))
    }

    /// Coefficients(A, B, C, D, E, F): `[A, B, C, D, E, F]`. `conic.coefficients()[5]`.
    pub fn coefficients(&self) -> [f64; 6] {
        [self.a, self.b, self.c, self.d, self.e, self.f]
    }

    /// NewCoefficients(A, B, C, D, E, F, Axis): the equation `coefficients`
    /// (natural frame) rewritten in the frame of `axis`. As natively, the
    /// coefficients of `self` are not read: the input array is transformed.
    /// `conic.new_coefficients(conic.coefficients(), axis)`.
    pub fn new_coefficients(&self, coefficients: [f64; 6], axis: Axis2d) -> [f64; 6] {
        let [a, b, c, d, e, f] = coefficients;
        let (t11, t21) = (axis.direction().x(), axis.direction().y());
        let (t13, t23) = (axis.location().x(), axis.location().y());
        let (t22, t12) = (t11, -t21);
        [
            t11 * (a * t11 + 2. * c * t21) + b * t21 * t21,
            t12 * (a * t12 + 2. * c * t22) + b * t22 * t22,
            t12 * (a * t11 + c * t21) + t22 * (c * t11 + b * t21),
            t11 * (d + a * t13) + t21 * (e + c * t13) + t23 * (c * t11 + b * t21),
            t12 * (d + a * t13) + t22 * (e + c * t13) + t23 * (c * t12 + b * t22),
            f + t13 * (2.0 * d + a * t13) + t23 * (2.0 * e + 2.0 * c * t13 + b * t23),
        ]
    }
}
