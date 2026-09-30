// Rust translation of Extrema_GlobOptFuncCC.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GlobOptFuncCCC0`, `CCC1` and `CCC2`: the distance `|C2(v) - C1(u)|` and
//! the gradient and Hessian of its square, for math_GlobOptMin. The native classes
//! hold either two 3D or two 2D adaptors (`myType`); here they are generic over the
//! [`ExtremaCurveTool`] of the pair. Outside the curve ranges they return false.
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::gp_space::{GpPoint, GpVector};
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{
    Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, MultipleVarFunctionWithHessian,
    Vector,
};

/// The two curves of a function, with the failure kept by its callbacks.
struct CurvePair<'a, T: ExtremaCurveTool> {
    c1: &'a T::Curve,
    c2: &'a T::Curve,
    failure: FailureCapture,
}

impl<'a, T: ExtremaCurveTool> CurvePair<'a, T> {
    // `u` and `v` inside the curve ranges.
    fn contains(&self, u: f64, v: f64) -> Result<bool, Failure> {
        let outside = u < T::first_parameter(self.c1)?
            || u > T::last_parameter(self.c1)?
            || v < T::first_parameter(self.c2)?
            || v > T::last_parameter(self.c2)?;
        Ok(!outside)
    }
    // `_Value`: F = |C2(v) - C1(u)|.
    fn value(&self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        let (u, v) = (x[1], x[2]);
        if !self.contains(u, v)? {
            return Ok(false);
        }
        *f = T::value(self.c2, v)?.distance_to(&T::value(self.c1, u)?);
        Ok(true)
    }
    // `_Gradient`: the gradient of |C2(v) - C1(u)|².
    fn gradient(&self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<bool, Failure> {
        if !self.contains(x[1], x[2])? {
            return Ok(false);
        }
        let (c1d0, c1d1) = T::d1(self.c1, x[1])?;
        let (c2d0, c2d1) = T::d1(self.c2, x[2])?;
        let d = |i: usize| c2d0.coordinate(i) - c1d0.coordinate(i);
        let mut g1: f64 = -d(0) * c1d1.coordinate(0);
        let mut g2: f64 = d(0) * c2d1.coordinate(0);
        for i in 1..T::Point::DIMENSION {
            g1 -= d(i) * c1d1.coordinate(i);
            g2 += d(i) * c2d1.coordinate(i);
        }
        g[1] = g1 * 2.;
        g[2] = g2 * 2.;
        Ok(true)
    }
    // `_Hessian`: the Hessian of |C2(v) - C1(u)|².
    fn hessian(&self, x: &Vector<'_>, h: &mut Matrix<'_>) -> Result<bool, Failure> {
        if !self.contains(x[1], x[2])? {
            return Ok(false);
        }
        let (c1d0, c1d1, c1d2) = T::d2(self.c1, x[1])?;
        let (c2d0, c2d1, c2d2) = T::d2(self.c2, x[2])?;
        let n: usize = T::Point::DIMENSION;
        let d = |i: usize| c2d0.coordinate(i) - c1d0.coordinate(i);
        let sum_squares = |w: &T::Vector| {
            (1..n).fold(w.coordinate(0) * w.coordinate(0), |s, i| {
                s + w.coordinate(i) * w.coordinate(i)
            })
        };
        let h11 = (0..n).fold(sum_squares(&c1d1), |s, i| s - d(i) * c1d2.coordinate(i));
        let h12 = (1..n).fold(-c2d1.coordinate(0) * c1d1.coordinate(0), |s, i| {
            s - c2d1.coordinate(i) * c1d1.coordinate(i)
        });
        let h22 = (0..n).fold(sum_squares(&c2d1), |s, i| s + d(i) * c2d2.coordinate(i));
        h[(1, 1)] = h11 * 2.;
        h[(1, 2)] = h12 * 2.;
        h[(2, 1)] = h12 * 2.;
        h[(2, 2)] = h22 * 2.;
        Ok(true)
    }
    fn keep(&mut self, result: Result<bool, Failure>) -> bool {
        self.failure.keep(result).unwrap_or(false)
    }
}

macro_rules! glob_opt_func_cc {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        pub struct $name<'a, T: ExtremaCurveTool> {
            pair: CurvePair<'a, T>,
        }
        impl<'a, T: ExtremaCurveTool> $name<'a, T> {
            /// The constructor of the native 3D and 2D overloads; `Function::<CurveTool>::new(&c1, &c2)`.
            pub fn new(c1: &'a T::Curve, c2: &'a T::Curve) -> Self {
                Self { pair: CurvePair { c1, c2, failure: FailureCapture::default() } }
            }
            /// `NbVariables()`: 2; `function.nb_variables()`.
            pub fn nb_variables(&self) -> i32 {
                2
            }
            /// `Value(X, F)`: `|C2(v) - C1(u)|`, false outside the ranges;
            /// `function.value(&x, &mut f)?`.
            pub fn value(&self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
                self.pair.value(x, f)
            }
            /// The first failure a math callback kept since the last call;
            /// `function.take_failure()?`.
            pub fn take_failure(&mut self) -> Result<(), Failure> {
                self.pair.failure.take()
            }
        }
        impl<T: ExtremaCurveTool> MultipleVarFunction for $name<'_, T> {
            fn variables(&self) -> i32 {
                2
            }
            fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
                let result = self.pair.value(point, value);
                self.pair.keep(result)
            }
        }
    };
}

// The gradient members of CCC1 and CCC2.
macro_rules! glob_opt_gradient {
    ($name:ident) => {
        impl<T: ExtremaCurveTool> $name<'_, T> {
            /// `Gradient(X, G)`: the gradient of the square distance;
            /// `function.gradient(&x, &mut g)?`.
            pub fn gradient(&self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<bool, Failure> {
                self.pair.gradient(x, g)
            }
            /// `Values(X, F, G)`: `Value && Gradient`; `function.values(&x, &mut f, &mut g)?`.
            pub fn values(
                &self,
                x: &Vector<'_>,
                f: &mut f64,
                g: &mut Vector<'_>,
            ) -> Result<bool, Failure> {
                Ok(self.pair.value(x, f)? && self.pair.gradient(x, g)?)
            }
        }
        impl<T: ExtremaCurveTool> MultipleVarFunctionWithGradient for $name<'_, T> {
            fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
                let result = self.pair.gradient(point, gradient);
                self.pair.keep(result)
            }
            fn values(
                &mut self,
                point: &Vector<'_>,
                value: &mut f64,
                gradient: &mut Vector<'_>,
            ) -> bool {
                let result = $name::values(self, point, value, gradient);
                self.pair.keep(result)
            }
        }
    };
}

glob_opt_func_cc!(
    /// `Extrema_GlobOptFuncCCC0`: the distance of two C0 curves;
    /// `GlobOptFuncCCC0::<CurveTool>::new(&c1, &c2)`.
    GlobOptFuncCCC0
);
glob_opt_func_cc!(
    /// `Extrema_GlobOptFuncCCC1`: the distance of two C1 curves with its gradient;
    /// `GlobOptFuncCCC1::<CurveTool>::new(&c1, &c2)`.
    GlobOptFuncCCC1
);
glob_opt_gradient!(GlobOptFuncCCC1);
glob_opt_func_cc!(
    /// `Extrema_GlobOptFuncCCC2`: the distance of two C2 curves with its gradient and
    /// Hessian; `GlobOptFuncCCC2::<CurveTool>::new(&c1, &c2)`.
    GlobOptFuncCCC2
);
glob_opt_gradient!(GlobOptFuncCCC2);

impl<T: ExtremaCurveTool> GlobOptFuncCCC2<'_, T> {
    /// `Values(X, F, G, H)`: the Hessian is computed first, then `Value && Gradient`
    /// (the Hessian flag counts last); `function.values_hessian(&x, &mut f, &mut g, &mut h)?`.
    pub fn values_hessian(
        &self,
        x: &Vector<'_>,
        f: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let hessian_computed: bool = self.pair.hessian(x, h)?;
        Ok(self.pair.value(x, f)? && self.pair.gradient(x, g)? && hessian_computed)
    }
}

impl<T: ExtremaCurveTool> MultipleVarFunctionWithHessian for GlobOptFuncCCC2<'_, T> {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        let result = GlobOptFuncCCC2::values_hessian(self, point, value, gradient, hessian);
        self.pair.keep(result)
    }
}
