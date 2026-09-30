// Rust translation of IntAna2d_IntPoint.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna2d_IntPoint: one point of a 2D analytic intersection.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

/// IntAna2d_IntPoint: the point, its parameter on the first element and, unless
/// the second element is an implicit curve, its parameter on the second one.
///
/// `let point = IntPoint::new(1.0, 2.0, 0.5, 0.25);`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntPoint {
    myu1: f64,
    myu2: f64,
    myp: Point2,
    myimplicit: bool,
}

impl Default for IntPoint {
    /// IntAna2d_IntPoint(): every coordinate and parameter at RealLast(), not
    /// implicit. `IntPoint::default().param_on_first() == f64::MAX`.
    fn default() -> Self {
        Self {
            myu1: f64::MAX,
            myu2: f64::MAX,
            myp: Point2::new(f64::MAX, f64::MAX),
            myimplicit: false,
        }
    }
}

impl IntPoint {
    /// IntAna2d_IntPoint(X, Y, U1, U2): a point between two parametric curves.
    /// `IntPoint::new(1.0, 2.0, 0.5, 0.25)`.
    pub fn new(x: f64, y: f64, u1: f64, u2: f64) -> Self {
        Self {
            myu1: u1,
            myu2: u2,
            myp: Point2::new(x, y),
            myimplicit: false,
        }
    }

    /// IntAna2d_IntPoint(X, Y, U1): a point between a parametric curve and an
    /// implicit one; the second parameter is RealLast(). `IntPoint::implicit(1.0, 2.0, 0.5)`.
    pub fn implicit(x: f64, y: f64, u1: f64) -> Self {
        Self {
            myu1: u1,
            myu2: f64::MAX,
            myp: Point2::new(x, y),
            myimplicit: true,
        }
    }

    /// SetValue(X, Y, U1, U2): the values of a "non-implicit" point.
    /// `point.set_value(1.0, 2.0, 0.5, 0.25)`.
    pub fn set_value(&mut self, x: f64, y: f64, u1: f64, u2: f64) {
        self.myimplicit = false;
        self.myp.set_coordinates(x, y);
        self.myu1 = u1;
        self.myu2 = u2;
    }

    /// SetValue(X, Y, U1): the values of an "implicit" point; the second parameter
    /// becomes RealLast(). `point.set_implicit_value(1.0, 2.0, 0.5)`.
    pub fn set_implicit_value(&mut self, x: f64, y: f64, u1: f64) {
        self.myimplicit = true;
        self.myp.set_coordinates(x, y);
        self.myu1 = u1;
        self.myu2 = f64::MAX;
    }

    /// Value(): the geometric point. `point.value().x()`.
    pub fn value(&self) -> Point2 {
        self.myp
    }

    /// SecondIsImplicit(): true when the second curve is implicit.
    /// `point.second_is_implicit()`.
    pub fn second_is_implicit(&self) -> bool {
        self.myimplicit
    }

    /// ParamOnFirst(): the parameter on the first element. `point.param_on_first()`.
    pub fn param_on_first(&self) -> f64 {
        self.myu1
    }

    /// ParamOnSecond(): the parameter on the second element; Standard_DomainError
    /// when the second element is an implicit curve. `point.param_on_second()?`.
    pub fn param_on_second(&self) -> Result<f64, Failure> {
        if self.myimplicit {
            return Err(Failure::with_type(
                "Standard_DomainError",
                "IntAna2d_IntPoint::ParamOnSecond() - the second element is implicit; expected a parametric second element",
                "",
            ));
        }
        Ok(self.myu2)
    }
}
