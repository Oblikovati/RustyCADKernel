// Rust translation of Extrema_GlobOptFuncCS.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GlobOptFuncCS`: `F(cu, su, sv) = |C(cu) - S(su, sv)|²` with its gradient
//! and Hessian (natively the half gradient and half Hessian), false outside the
//! curve and surface bounds.
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::math::{
    Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, MultipleVarFunctionWithHessian,
    Vector,
};

/// `Extrema_GlobOptFuncCS`; `GlobOptFuncCS::new(&curve, &surface)`.
pub struct GlobOptFuncCS<'a> {
    c: &'a dyn Adaptor3dCurve,
    s: &'a dyn Adaptor3dSurface,
    failure: FailureCapture,
}

impl<'a> GlobOptFuncCS<'a> {
    /// `Extrema_GlobOptFuncCS(C, S)`; `GlobOptFuncCS::new(&curve, &surface)`.
    pub fn new(curve: &'a dyn Adaptor3dCurve, surface: &'a dyn Adaptor3dSurface) -> Self {
        Self {
            c: curve,
            s: surface,
            failure: FailureCapture::default(),
        }
    }
    /// `NbVariables()`: 3; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        3
    }
    // `checkInputData`: the three values from the lower index, inside the bounds.
    fn check_input_data(&self, x: &Vector<'_>) -> Result<Option<(f64, f64, f64)>, Failure> {
        let start: i32 = x.lower();
        let (cu, su, sv) = (x[start], x[start + 1], x[start + 2]);
        let inside: bool = cu >= self.c.first_parameter()?
            && cu <= self.c.last_parameter()?
            && su >= self.s.first_u_parameter()?
            && su <= self.s.last_u_parameter()?
            && sv >= self.s.first_v_parameter()?
            && sv <= self.s.last_v_parameter()?;
        Ok(inside.then_some((cu, su, sv)))
    }
    fn value_at(&self, (cu, su, sv): (f64, f64, f64)) -> Result<f64, Failure> {
        Ok(self.c.value(cu)?.square_distance(&self.s.value(su, sv)?))
    }
    fn gradient_at(
        &self,
        (cu, su, sv): (f64, f64, f64),
        g: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let (cd0, cd1) = self.c.d1(cu)?;
        let s = self.s.d1(su, sv)?;
        let (sd0, sd1u, sd1v) = (s.point, s.d1u, s.d1v);
        let (dx, dy, dz) = (cd0.x() - sd0.x(), cd0.y() - sd0.y(), cd0.z() - sd0.z());
        g[1] = dx * cd1.x() + dy * cd1.y() + dz * cd1.z();
        g[2] = -dx * sd1u.x() - dy * sd1u.y() - dz * sd1u.z();
        g[3] = -dx * sd1v.x() - dy * sd1v.y() - dz * sd1v.z();
        Ok(())
    }
    fn hessian_at(&self, (cu, su, sv): (f64, f64, f64), h: &mut Matrix<'_>) -> Result<(), Failure> {
        let (cd0, cd1, cd2) = self.c.d2(cu)?;
        let s = self.s.d2(su, sv)?;
        let (sd0, sd1u, sd1v) = (s.point, s.d1u, s.d1v);
        let (dx, dy, dz) = (cd0.x() - sd0.x(), cd0.y() - sd0.y(), cd0.z() - sd0.z());
        h[(1, 1)] = cd1.x() * cd1.x()
            + cd1.y() * cd1.y()
            + cd1.z() * cd1.z()
            + dx * cd2.x()
            + dy * cd2.y()
            + dz * cd2.z();
        h[(1, 2)] = -cd1.x() * sd1u.x() - cd1.y() * sd1u.y() - cd1.z() * sd1u.z();
        h[(1, 3)] = -cd1.x() * sd1v.x() - cd1.y() * sd1v.y() - cd1.z() * sd1v.z();
        h[(2, 1)] = h[(1, 2)];
        h[(2, 2)] = sd1u.x() * sd1u.x() + sd1u.y() * sd1u.y() + sd1u.z() * sd1u.z()
            - dx * s.d2u.x()
            - dy * s.d2u.y()
            - dz * s.d2u.z();
        h[(2, 3)] = sd1u.x() * sd1v.x() + sd1u.y() * sd1v.y() + sd1u.z() * sd1v.z()
            - dx * s.d2uv.x()
            - dy * s.d2uv.y()
            - dz * s.d2uv.z();
        h[(3, 1)] = h[(1, 3)];
        h[(3, 2)] = h[(2, 3)];
        h[(3, 3)] = sd1v.x() * sd1v.x() + sd1v.y() * sd1v.y() + sd1v.z() * sd1v.z()
            - dx * s.d2v.x()
            - dy * s.d2v.y()
            - dz * s.d2v.z();
        Ok(())
    }
    /// `Value(X, F)`; `function.value(&x, &mut f)?`.
    pub fn value(&self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        let Some(at) = self.check_input_data(x)? else {
            return Ok(false);
        };
        *f = self.value_at(at)?;
        Ok(true)
    }
    /// `Gradient(X, G)`; `function.gradient(&x, &mut g)?`.
    pub fn gradient(&self, x: &Vector<'_>, g: &mut Vector<'_>) -> Result<bool, Failure> {
        let Some(at) = self.check_input_data(x)? else {
            return Ok(false);
        };
        self.gradient_at(at, g)?;
        Ok(true)
    }
    /// `Values(X, F, G)`; `function.values(&x, &mut f, &mut g)?`.
    pub fn values(&self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> Result<bool, Failure> {
        let Some(at) = self.check_input_data(x)? else {
            return Ok(false);
        };
        *f = self.value_at(at)?;
        self.gradient_at(at, g)?;
        Ok(true)
    }
    /// `Values(X, F, G, H)`; `function.values_hessian(&x, &mut f, &mut g, &mut h)?`.
    pub fn values_hessian(
        &self,
        x: &Vector<'_>,
        f: &mut f64,
        g: &mut Vector<'_>,
        h: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let Some(at) = self.check_input_data(x)? else {
            return Ok(false);
        };
        *f = self.value_at(at)?;
        self.gradient_at(at, g)?;
        self.hessian_at(at, h)?;
        Ok(true)
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl MultipleVarFunction for GlobOptFuncCS<'_> {
    fn variables(&self) -> i32 {
        3
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let result = GlobOptFuncCS::value(self, point, value);
        self.failure.keep(result).unwrap_or(false)
    }
}

impl MultipleVarFunctionWithGradient for GlobOptFuncCS<'_> {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let result = GlobOptFuncCS::gradient(self, point, gradient);
        self.failure.keep(result).unwrap_or(false)
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        let result = GlobOptFuncCS::values(self, point, value, gradient);
        self.failure.keep(result).unwrap_or(false)
    }
}

impl MultipleVarFunctionWithHessian for GlobOptFuncCS<'_> {
    fn values_hessian(
        &mut self,
        point: &Vector<'_>,
        value: &mut f64,
        gradient: &mut Vector<'_>,
        hessian: &mut Matrix<'_>,
    ) -> bool {
        let result = GlobOptFuncCS::values_hessian(self, point, value, gradient, hessian);
        self.failure.keep(result).unwrap_or(false)
    }
}
