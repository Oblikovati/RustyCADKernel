// Rust translation of ModelingData/TKGeomBase/CPnts/CPnts_MyRootFunction.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `CPnts_MyRootFunction`: `Integral(X0, X, F) - L`, whose root is the parameter at
//! the curvilinear distance `L` from `X0`.
use super::{MyGaussFunction, RealFunction};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Function, FunctionWithDerivative, GaussSingleIntegration};

/// `CPnts_MyRootFunction`: the `math_FunctionWithDerivative` solved by
/// `CPnts_AbscissaPoint`; `MyRootFunction::new()`.
///
/// A failure of the integrand is kept by the inner [`MyGaussFunction`] and read back
/// with [`Self::take_failure`].
#[derive(Default)]
pub struct MyRootFunction<'a> {
    my_function: MyGaussFunction<'a>,
    my_x0: f64,
    my_l: f64,
    my_order: i32,
    my_tol: f64,
}

impl<'a> MyRootFunction<'a> {
    /// The empty function, all fields null (`CPnts_MyRootFunction()`);
    /// `MyRootFunction::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the integrand and the Gauss order (`Init(F, D, Order)`);
    /// `root.init_function(Box::new(|_| Ok(1.0)), 10)`.
    pub fn init_function(&mut self, function: RealFunction<'a>, order: i32) {
        self.my_function.init(function);
        self.my_order = order;
    }
    /// Solve `Integral(X0, X, F) = L` without tolerance (`Init(X0, L)`, which stores
    /// the tolerance -1); `root.init_target(0.0, 2.0)`.
    pub fn init_target(&mut self, x0: f64, l: f64) {
        self.my_x0 = x0;
        self.my_l = l;
        self.my_tol = -1.0;
    }
    /// Solve `Integral(X0, X, F) = L` with the integration tolerance `tol`
    /// (`Init(X0, L, Tol)`); `root.init_target_with_tolerance(0.0, 2.0, 1e-8)`.
    pub fn init_target_with_tolerance(&mut self, x0: f64, l: f64, tol: f64) {
        self.my_x0 = x0;
        self.my_l = l;
        self.my_tol = tol;
    }
    /// `Err` with the first failure raised by the integrand since the last call;
    /// `root.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.my_function.take_failure()
    }
    // Integral(X0, X, F) - L, or None when the integration is not done; a null or
    // negative tolerance selects the single Gauss rule.
    fn length_gap(&mut self, x: f64) -> Option<f64> {
        let length: GaussSingleIntegration = if self.my_tol <= 0.0 {
            GaussSingleIntegration::new(&mut self.my_function, self.my_x0, x, self.my_order)
        } else {
            GaussSingleIntegration::with_tolerance(
                &mut self.my_function,
                self.my_x0,
                x,
                self.my_order,
                self.my_tol,
            )
        };
        length.value().ok().map(|value| value - self.my_l)
    }
}

impl Function for MyRootFunction<'_> {
    /// `Value(X, F)`: `Integral(X0, X, F) - L`.
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        match self.length_gap(x) {
            Some(value) => {
                *result = value;
                true
            }
            None => false,
        }
    }
}

impl FunctionWithDerivative for MyRootFunction<'_> {
    /// `Derivative(X, Df)`: `F(X)`.
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        self.my_function.value(x, derivative)
    }
    /// `Values(X, F, Df)`: both, `Df` evaluated only after a done integration.
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        match self.length_gap(x) {
            Some(gap) => {
                *value = gap;
                self.my_function.value(x, derivative)
            }
            None => false,
        }
    }
}
