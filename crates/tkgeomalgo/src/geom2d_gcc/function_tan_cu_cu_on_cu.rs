// Rust translation of Geom2dGcc_FunctionTanCuCuOnCu.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_FunctionTanCuCuOnCu`: the system of a circle tangent to two
//! arguments with its centre on a third.
use super::function_tan_cu_cu_cu::{TanCuCuCuDerivatives, curve_d2, line_d2};
use super::{CurveTool, EvalFailure, Type2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Line2, Point2, Vector2};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// `Geom2dGcc_FunctionTanCuCuOnCu`: with `P1`, `P2` the tangency points of
/// parameters `X(1)`, `X(2)`, `P3` the centre of parameter `X(3)` on the locus and
/// `X(4)` the radius, the four equations `|P3P1|^2 = X(4)^2`, `|P2P3|^2 = X(4)^2`,
/// `P3P1 . T1 = 0` and `P2P3 . T2 = 0`, scaled by the first radius `Rad`; a
/// `math_FunctionSetWithDerivatives` of 4 variables and 4 equations.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::FunctionTanCuCuOnCu;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// use rustycad_tkmath::math::FunctionSet;
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, 4.0), Direction2::new(1.0, 0.0)?)).cast();
/// let adaptor = Geom2dAdaptorCurve::with_curve(&line)?;
/// let on = Line2::new(Point2::new(0.0, 0.0), Direction2::new(0.0, 1.0)?);
/// let f = FunctionTanCuCuOnCu::curve_pnt_on_line(&adaptor, Point2::new(0.0, 0.0), on, 2.0);
/// assert_eq!(f.equations(), 4);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FunctionTanCuCuOnCu<'a> {
    curv1: Option<&'a Geom2dAdaptorCurve>,
    curv2: Option<&'a Geom2dAdaptorCurve>,
    circ1: Circle2,
    lin1: Line2,
    pnt2: Point2,
    circon: Circle2,
    linon: Line2,
    curvon: Option<&'a Geom2dAdaptorCurve>,
    first_rad: f64,
    the_type: Type2,
    failure: EvalFailure,
}

// One named constructor per native (curve, curve) overload.
macro_rules! curve_curve_constructor {
    ($(#[$doc:meta])* $name:ident, $on:ident, $on_ty:ty, $kind:ident) => {
        $(#[$doc])*
        pub fn $name(
            c1: &'a Geom2dAdaptorCurve,
            c2: &'a Geom2dAdaptorCurve,
            on: $on_ty,
            rad: f64,
        ) -> Self {
            let mut f: Self = Self::with_type(Type2::$kind, rad);
            (f.curv1, f.curv2, f.$on) = (Some(c1), Some(c2), on.into());
            f
        }
    };
}

// One named constructor per native (circle, curve) overload.
macro_rules! circ_curve_constructor {
    ($(#[$doc:meta])* $name:ident, $on:ident, $on_ty:ty, $kind:ident) => {
        $(#[$doc])*
        pub fn $name(c1: Circle2, c2: &'a Geom2dAdaptorCurve, on: $on_ty, rad: f64) -> Self {
            let mut f: Self = Self::with_type(Type2::$kind, rad);
            (f.circ1, f.curv2, f.$on) = (c1, Some(c2), on.into());
            f
        }
    };
}

// One named constructor per native (line, curve) overload.
macro_rules! lin_curve_constructor {
    ($(#[$doc:meta])* $name:ident, $on:ident, $on_ty:ty, $kind:ident) => {
        $(#[$doc])*
        pub fn $name(l1: Line2, c2: &'a Geom2dAdaptorCurve, on: $on_ty, rad: f64) -> Self {
            let mut f: Self = Self::with_type(Type2::$kind, rad);
            (f.lin1, f.curv2, f.$on) = (l1, Some(c2), on.into());
            f
        }
    };
}

// One named constructor per native (curve, point) overload.
macro_rules! curve_pnt_constructor {
    ($(#[$doc:meta])* $name:ident, $on:ident, $on_ty:ty, $kind:ident) => {
        $(#[$doc])*
        pub fn $name(c1: &'a Geom2dAdaptorCurve, p2: Point2, on: $on_ty, rad: f64) -> Self {
            let mut f: Self = Self::with_type(Type2::$kind, rad);
            (f.curv1, f.pnt2, f.$on) = (Some(c1), p2, on.into());
            f
        }
    };
}

impl<'a> FunctionTanCuCuOnCu<'a> {
    fn with_type(the_type: Type2, rad: f64) -> Self {
        Self {
            curv1: None,
            curv2: None,
            circ1: Circle2::default(),
            lin1: Line2::default(),
            pnt2: Point2::default(),
            circon: Circle2::default(),
            linon: Line2::default(),
            curvon: None,
            first_rad: rad,
            the_type,
            failure: EvalFailure::default(),
        }
    }

    curve_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(C1, C2, gp_Circ2d OnCi, Rad)`.
        curve_curve_on_circle, circon, Circle2, CuCuOnCi
    );
    circ_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(gp_Circ2d C1, C2, gp_Circ2d OnCi, Rad)`.
        /// Natively it selects the type CuCuOnCi (not CiCuOnCi), so it evaluates the
        /// empty first curve: a null handle natively, Standard_NullObject here.
        circ_curve_on_circle, circon, Circle2, CuCuOnCi
    );
    lin_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(gp_Lin2d L1, C2, gp_Circ2d OnCi, Rad)`.
        lin_curve_on_circle, circon, Circle2, LiCuOnCi
    );
    curve_pnt_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(C1, gp_Pnt2d P2, gp_Circ2d OnCi, Rad)`.
        curve_pnt_on_circle, circon, Circle2, CuPtOnCi
    );
    curve_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(C1, C2, gp_Lin2d OnLi, Rad)`.
        curve_curve_on_line, linon, Line2, CuCuOnLi
    );
    circ_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(gp_Circ2d C1, C2, gp_Lin2d OnLi, Rad)`.
        circ_curve_on_line, linon, Line2, CiCuOnLi
    );
    lin_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(gp_Lin2d L1, C2, gp_Lin2d OnLi, Rad)`.
        lin_curve_on_line, linon, Line2, LiCuOnLi
    );
    curve_pnt_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(C1, gp_Pnt2d P2, gp_Lin2d OnLi, Rad)`.
        curve_pnt_on_line, linon, Line2, CuPtOnLi
    );
    curve_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(C1, C2, OnCu, Rad)`.
        curve_curve_on_curve, curvon, &'a Geom2dAdaptorCurve, CuCuOnCu
    );
    circ_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(gp_Circ2d C1, C2, OnCu, Rad)`.
        circ_curve_on_curve, curvon, &'a Geom2dAdaptorCurve, CiCuOnCu
    );
    lin_curve_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(gp_Lin2d L1, C2, OnCu, Rad)`.
        lin_curve_on_curve, curvon, &'a Geom2dAdaptorCurve, LiCuOnCu
    );
    curve_pnt_constructor!(
        /// `Geom2dGcc_FunctionTanCuCuOnCu(C1, gp_Pnt2d P1, OnCu, Rad)`.
        curve_pnt_on_curve, curvon, &'a Geom2dAdaptorCurve, CuPtOnCu
    );

    /// The exception a curve evaluation raised natively, after a member answered
    /// `false`; `f.raised()?`.
    pub fn raised(&self) -> Result<(), Failure> {
        self.failure.check()
    }

    /// `InitDerivative(X, Point1, Point2, Point3, Tan1, Tan2, Tan3, D21, D22, D23)`:
    /// the first argument at `X(1)`, the second at `X(2)` (the point has null
    /// derivatives) and the centre locus at `X(3)`; a line has a null second
    /// derivative. `let [a1, a2, on] = f.init_derivative(&x)?;`.
    pub fn init_derivative(&self, x: &Vector<'_>) -> Result<TanCuCuCuDerivatives, Failure> {
        use Type2::*;
        let (u1, u2, u3) = (*x.value(1)?, *x.value(2)?, *x.value(3)?);
        let first = match self.the_type {
            CiCuOnCu | CiCuOnLi | CiCuOnCi => elclib::d2(u1, &self.circ1),
            LiCuOnCu | LiCuOnLi | LiCuOnCi => line_d2(u1, &self.lin1),
            _ => curve_d2(self.curv1, u1)?,
        };
        let zero = Vector2::new(0.0, 0.0);
        let second = match self.the_type {
            CuPtOnCu | CuPtOnLi | CuPtOnCi => (self.pnt2, zero, zero),
            _ => curve_d2(self.curv2, u2)?,
        };
        let third = match self.the_type {
            CuCuOnCi | CiCuOnCi | LiCuOnCi | CuPtOnCi => elclib::d2(u3, &self.circon),
            CuCuOnLi | CiCuOnLi | LiCuOnLi | CuPtOnLi => line_d2(u3, &self.linon),
            _ => CurveTool::d2(self.curvon_or_null()?, u3)?,
        };
        Ok([first, second, third])
    }

    fn curvon_or_null(&self) -> Result<&'a Geom2dAdaptorCurve, Failure> {
        self.curvon
            .ok_or_else(|| super::null_object("Geom2dGcc_FunctionTanCuCuOnCu::InitDerivative"))
    }

    // Fval(1..4).
    fn write_values(&self, x: &Vector<'_>, fval: &mut Vector<'_>) -> Result<(), Failure> {
        let [(point1, tan1, _), (point2, tan2, _), (point3, _, _)] = self.init_derivative(x)?;
        let p2p3 = Vector2::between(point2, point3);
        let p3p1 = Vector2::between(point3, point1);
        let nnor1: f64 = tan1.magnitude();
        let nnor2: f64 = tan2.magnitude();
        let (r, x4): (f64, f64) = (self.first_rad, *x.value(4)?);
        *fval.change_value(1)? = (p3p1.dot(p3p1) - x4 * x4) / (r * r);
        *fval.change_value(2)? = (p2p3.dot(p2p3) - x4 * x4) / (r * r);
        *fval.change_value(3)? = p3p1.dot(tan1) / (nnor1 * r);
        *fval.change_value(4)? = p2p3.dot(tan2) / (nnor2 * r);
        Ok(())
    }

    // Deriv(1..4, 1..4); the (4, 3) entry divides by |Tan1| as natively.
    fn write_derivatives(&self, x: &Vector<'_>, deriv: &mut Matrix<'_>) -> Result<(), Failure> {
        let [(point1, tan1, d21), (point2, tan2, d22), (point3, tan3, _)] =
            self.init_derivative(x)?;
        let p2p3 = Vector2::between(point2, point3);
        let p3p1 = Vector2::between(point3, point1);
        let (n1, n2): (f64, f64) = (tan1.magnitude(), tan2.magnitude());
        let (r, x4): (f64, f64) = (self.first_rad, *x.value(4)?);
        let rows: [[f64; 4]; 4] = [
            [
                2.0 * tan1.dot(p3p1) / (r * r),
                0.0,
                -2.0 * tan3.dot(p3p1) / (r * r),
                -2.0 * x4 / (r * r),
            ],
            [
                0.0,
                -2.0 * tan2.dot(p2p3) / (r * r),
                2.0 * tan3.dot(p2p3) / (r * r),
                -2.0 * x4 / (r * r),
            ],
            [
                (p3p1.dot(d21) + tan1.dot(tan1)) / (r * n1)
                    - (p3p1.dot(tan1) * d21.dot(tan1)) / (r * n1 * n1 * n1),
                0.0,
                -(tan3.dot(tan1)) / (r * n1),
                0.0,
            ],
            [
                0.0,
                (p2p3.dot(d22) - tan2.dot(tan2)) / (r * n2)
                    - p2p3.dot(tan2) * tan2.dot(d22) / (r * n2 * n2 * n2),
                tan3.dot(tan2) / (r * n1),
                0.0,
            ],
        ];
        for (row, values) in (1..=4).zip(rows) {
            for (col, value) in (1..=4).zip(values) {
                *deriv.change_value(row, col)? = value;
            }
        }
        Ok(())
    }
}

impl FunctionSet for FunctionTanCuCuOnCu<'_> {
    fn variables(&self) -> i32 {
        4
    }
    fn equations(&self) -> i32 {
        4
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        let result = self.write_values(x, f);
        self.failure.keep(result).is_some()
    }
}

impl FunctionSetWithDerivatives for FunctionTanCuCuOnCu<'_> {
    fn derivatives(&mut self, x: &Vector<'_>, deriv: &mut Matrix<'_>) -> bool {
        let result = self.write_derivatives(x, deriv);
        self.failure.keep(result).is_some()
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, deriv: &mut Matrix<'_>) -> bool {
        let result = self
            .write_values(x, f)
            .and_then(|()| self.write_derivatives(x, deriv));
        self.failure.keep(result).is_some()
    }
}
