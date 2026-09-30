use super::super::{ContapSurfProps, ContapTFunction, geometry::vector_of_dir};
use super::ContapSurfFunction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    gp::Vector3,
    math::{Matrix, Vector},
};
impl ContapSurfFunction {
    /// The value of `F` for the normal `norm` at the point `solpt` (None for DraftPrs).
    fn function_of(&self, norm: Vector3) -> Option<f64> {
        let dir = vector_of_dir(self.my_dir);
        match self.my_type {
            ContapTFunction::ContourStd => Some(norm.dot(dir) / self.my_mean),
            ContapTFunction::ContourPrs => {
                Some(norm.dot(Vector3::between(self.my_eye, self.solpt)) / self.my_mean)
            }
            ContapTFunction::DraftStd => {
                Some((norm.dot(dir) - self.my_cos_ang * norm.magnitude()) / self.my_mean)
            }
            ContapTFunction::DraftPrs => None,
        }
    }

    /// The gradient of `F` from the normal and its derivatives (None for DraftPrs);
    /// the draft gradient uses the normalized normal.
    pub(super) fn gradient_of(
        &self,
        mut norm: Vector3,
        dnu: Vector3,
        dnv: Vector3,
    ) -> Result<Option<(f64, f64)>, Failure> {
        let dir = vector_of_dir(self.my_dir);
        Ok(match self.my_type {
            ContapTFunction::ContourStd => {
                Some((dnu.dot(dir) / self.my_mean, dnv.dot(dir) / self.my_mean))
            }
            ContapTFunction::ContourPrs => {
                let ep = Vector3::between(self.my_eye, self.solpt);
                Some((dnu.dot(ep) / self.my_mean, dnv.dot(ep) / self.my_mean))
            }
            ContapTFunction::DraftStd => {
                norm.normalize()?;
                let c = self.my_cos_ang;
                Some((
                    (dnu.dot(dir) - c * dnu.dot(norm)) / self.my_mean,
                    (dnv.dot(dir) - c * dnv.dot(norm)) / self.my_mean,
                ))
            }
            ContapTFunction::DraftPrs => None,
        })
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
