// Rust translation of the template roles of Contap_SurfFunction (TheFunction of Contap_TheSearchInside, TheIWFunction of Contap_TheIWalking) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solver and template sides of `ContapSurfFunction`: `math_FunctionSetWithDerivatives`
//! with the failure capture, `TheInsideFunction` and `TheIWFunction`.
use super::surf_function::ContapSurfFunction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::{self as hst, Adaptor3dSurfaceHandle};
use rustycad_tkgeomalgo::int_start::TheInsideFunction;
use rustycad_tkgeomalgo::int_walk::TheIWFunction;
use rustycad_tkmath::gp::{Direction2, Point3, Vector3};
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

/// The solver side: a failure of a callback is kept, the solver gets false, and the
/// template raises the failure when the solver returns (`take_failure`).
impl FunctionSet for ContapSurfFunction {
    fn variables(&self) -> i32 {
        self.nb_variables()
    }
    fn equations(&self) -> i32 {
        self.nb_equations()
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        let result = ContapSurfFunction::value(self, x, f);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl FunctionSetWithDerivatives for ContapSurfFunction {
    fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
        let result = ContapSurfFunction::derivatives(self, x, d);
        self.capture.keep(result).unwrap_or(false)
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
        let result = ContapSurfFunction::values(self, x, f, d);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl ContapSurfFunction {
    /// `Set(S)` from a template, whose `Set` cannot fail: a failure of the sampling is
    /// kept and raised by the next `take_failure` (docs/hlr-design.md section 6).
    fn set_surface_captured(&mut self, surface: &Adaptor3dSurfaceHandle) {
        let result = self.set_surface(surface);
        self.capture.keep(result);
    }

    /// The first failure kept since the last call; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }
}

/// `TheFunction` of `Contap_TheSearchInside`.
impl TheInsideFunction for ContapSurfFunction {
    type Surface = Adaptor3dSurfaceHandle;
    fn set_surface(&mut self, surface: &Adaptor3dSurfaceHandle) {
        self.set_surface_captured(surface);
    }
    fn tolerance(&self) -> f64 {
        ContapSurfFunction::tolerance(self)
    }
    fn checked_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        ContapSurfFunction::value(self, x, f)
    }
    fn root(&self) -> f64 {
        ContapSurfFunction::root(self)
    }
    fn is_tangent(&mut self) -> Result<bool, Failure> {
        ContapSurfFunction::is_tangent(self)
    }
    fn point(&self) -> Point3 {
        ContapSurfFunction::point(self)
    }
    fn direction3d(&mut self) -> Result<Vector3, Failure> {
        ContapSurfFunction::direction3d(self)
    }
    fn direction2d(&mut self) -> Result<Direction2, Failure> {
        ContapSurfFunction::direction2d(self)
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        ContapSurfFunction::take_failure(self)
    }
}

/// `TheIWFunction` of `Contap_TheIWalking`.
impl TheIWFunction for ContapSurfFunction {
    type Surface = Adaptor3dSurfaceHandle;
    fn set_surface(&mut self, surface: &Adaptor3dSurfaceHandle) {
        self.set_surface_captured(surface);
    }
    fn checked_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        ContapSurfFunction::value(self, x, f)
    }
    fn checked_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        ContapSurfFunction::values(self, x, f, d)
    }
    fn checked_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        ContapSurfFunction::derivatives(self, x, d)
    }
    fn root(&self) -> f64 {
        ContapSurfFunction::root(self)
    }
    fn tolerance(&self) -> f64 {
        ContapSurfFunction::tolerance(self)
    }
    fn is_tangent(&mut self) -> Result<bool, Failure> {
        ContapSurfFunction::is_tangent(self)
    }
    fn point(&self) -> Point3 {
        ContapSurfFunction::point(self)
    }
    fn direction3d(&mut self) -> Result<Vector3, Failure> {
        ContapSurfFunction::direction3d(self)
    }
    fn direction2d(&mut self) -> Result<Direction2, Failure> {
        ContapSurfFunction::direction2d(self)
    }
    // `PSurface()->Value(U, V)` calls the adaptor member.
    fn p_surface_value(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        hst::value(&self.p_surface(), u, v)
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        ContapSurfFunction::take_failure(self)
    }
}
