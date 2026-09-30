use super::super::ContapSurfProps;
use super::ContapSurfFunction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    gp::Vector3,
    math::{Matrix, Vector},
};
impl ContapSurfFunction {
    fn equation(&self) -> super::super::equation::ContourEquation {
        super::super::equation::ContourEquation {
            kind: self.my_type,
            direction: self.my_dir,
            eye: self.my_eye,
            cosine: self.my_cos_ang,
            scale: self.my_mean,
        }
    }
    fn function_of(&self, normal: Vector3) -> Option<f64> {
        self.equation().normal_value(self.solpt, normal)
    }
    pub(super) fn gradient_of(
        &self,
        normal: Vector3,
        du: Vector3,
        dv: Vector3,
    ) -> Result<Option<(f64, f64)>, Failure> {
        self.equation().normal_gradient(self.solpt, normal, du, dv)
    }

    /// Value(X, F): `F` at `x`; always true; `f.value(&x, &mut out)?`.
    pub fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        (self.usol, self.vsol) = (x[1], x[2]);
        let (p, norm) = ContapSurfProps::normale(&self.my_surf, self.usol, self.vsol)?;
        self.solpt = p;
        if let Some(value) = self.function_of(norm) {
            self.valf = value;
            f[1] = value;
        }
        self.computed = false;
        self.derived = false;
        Ok(true)
    }

    /// Derivatives(X, D): the gradient of `F` at `x`; always true;
    /// `f.derivatives(&x, &mut grad)?`.
    pub fn derivatives(&mut self, x: &Vector<'_>, grad: &mut Matrix<'_>) -> Result<bool, Failure> {
        (self.usol, self.vsol) = (x[1], x[2]);
        let (p, norm, dnu, dnv) =
            ContapSurfProps::norm_and_dn(&self.my_surf, self.usol, self.vsol)?;
        self.solpt = p;
        if let Some((gu, gv)) = self.gradient_of(norm, dnu, dnv)? {
            (grad[(1, 1)], grad[(1, 2)]) = (gu, gv);
        }
        (self.fpu, self.fpv) = (grad[(1, 1)], grad[(1, 2)]);
        self.computed = false;
        self.derived = true;
        Ok(true)
    }

    /// Values(X, F, D): `F` and its gradient at `x`; always true;
    /// `f.values(&x, &mut out, &mut grad)?`.
    pub fn values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        grad: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        (self.usol, self.vsol) = (x[1], x[2]);
        let (p, norm, dnu, dnv) =
            ContapSurfProps::norm_and_dn(&self.my_surf, self.usol, self.vsol)?;
        self.solpt = p;
        if let Some(value) = self.function_of(norm) {
            f[1] = value;
        }
        if let Some((gu, gv)) = self.gradient_of(norm, dnu, dnv)? {
            (grad[(1, 1)], grad[(1, 2)]) = (gu, gv);
        }
        self.valf = f[1];
        (self.fpu, self.fpv) = (grad[(1, 1)], grad[(1, 2)]);
        self.computed = false;
        self.derived = true;
        Ok(true)
    }
}
