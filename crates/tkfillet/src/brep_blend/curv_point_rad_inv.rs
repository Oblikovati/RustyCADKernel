// Rust translation of BRepBlend_CurvPointRadInv.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_CurvPointRadInv`: the inverse of a blend between two restrictions that ends
//! on a point: variables `(w, U)`, the parameters on the first and the second curve.
use super::live_curve_of;
use super::rad_inv_terms::GuidePlane;
use crate::blend::{BlendCurvPointFuncInv, BlendInverseBounds, curv_point_func_inv_nb_variables};
use crate::blend_func::linalg::{mset, vector, vget, vset};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{FunctionSet, FunctionSetWithDerivatives, Matrix, Vector};

const CLASS: &str = "BRepBlend_CurvPointRadInv";

/// `BRepBlend_CurvPointRadInv`: the point and the point of the second curve in the plane
/// normal to the first curve; `F = (nplan.P + theD, nplan.C2(U) + theD)`.
///
/// ```ignore
/// let mut inverse = BRepBlendCurvPointRadInv::new(&curve1, &curve2);
/// inverse.set_point(point);
/// ```
pub struct BRepBlendCurvPointRadInv {
    curv1: Handle<dyn Adaptor3dCurve>,
    curv2: Handle<dyn Adaptor3dCurve>,
    point: Point3,
    choix: i32,
    capture: FailureCapture,
}

impl BRepBlendCurvPointRadInv {
    /// `BRepBlend_CurvPointRadInv(C1, C2)`; `BRepBlendCurvPointRadInv::new(&c1, &c2)`.
    pub fn new(c1: &Handle<dyn Adaptor3dCurve>, c2: &Handle<dyn Adaptor3dCurve>) -> Self {
        Self {
            curv1: c1.clone(),
            curv2: c2.clone(),
            point: Point3::default(),
            choix: 0,
            capture: FailureCapture::default(),
        }
    }

    /// `Set(Choix)`: kept, unused by the equations; `inverse.set_choix(1)`.
    pub fn set_choix(&mut self, choix: i32) {
        self.choix = choix;
    }

    /// `Value(X, F)`: the plane normal by `Normalized` (unlike the derivatives).
    fn try_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let (ptcur1, d1cur1) = live_curve_of(&self.curv1, CLASS)?.d1(vget(x, 1)?)?;
        let nplan = *d1cur1.normalized()?.xyz();
        let the_d = -nplan.dot(ptcur1.xyz());
        let (ptcur2, _) = live_curve_of(&self.curv2, CLASS)?.d1(vget(x, 2)?)?;
        vset(f, 1, nplan.dot(self.point.xyz()) + the_d)?;
        vset(f, 2, nplan.dot(ptcur2.xyz()) + the_d)?;
        Ok(true)
    }

    /// `Derivatives(X, D)`.
    fn try_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        let (ptcur1, d1cur1, d2cur1) = live_curve_of(&self.curv1, CLASS)?.d2(vget(x, 1)?)?;
        let plane = GuidePlane::new(ptcur1, d1cur1, d2cur1);
        mset(d, 1, 1, plane.derivative_offset(self.point))?;
        mset(d, 1, 2, 0.)?;
        let (ptcur2, d1cur2) = live_curve_of(&self.curv2, CLASS)?.d1(vget(x, 2)?)?;
        mset(d, 2, 1, plane.derivative_offset(ptcur2))?;
        mset(d, 2, 2, plane.nplan.dot(d1cur2))?;
        Ok(true)
    }
}

impl FunctionSet for BRepBlendCurvPointRadInv {
    /// 2 (`Blend_CurvPointFuncInv::NbVariables`).
    fn variables(&self) -> i32 {
        curv_point_func_inv_nb_variables()
    }
    /// `NbEquations()`: 2.
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        let result = self.try_value(x, f);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl FunctionSetWithDerivatives for BRepBlendCurvPointRadInv {
    fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
        let result = self.try_derivatives(x, d);
        self.capture.keep(result).unwrap_or(false)
    }
    /// `Values(X, F, D)`: `Value` then `Derivatives`, always true.
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
        let result = self
            .try_value(x, f)
            .and_then(|_| self.try_derivatives(x, d))
            .map(|_| true);
        self.capture.keep(result).unwrap_or(false)
    }
}

impl BlendInverseBounds for BRepBlendCurvPointRadInv {
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }
    fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure> {
        vset(
            tolerance,
            1,
            live_curve_of(&self.curv1, CLASS)?.resolution(tol)?,
        )?;
        vset(
            tolerance,
            2,
            live_curve_of(&self.curv2, CLASS)?.resolution(tol)?,
        )
    }
    fn get_bounds(&self, inf: &mut Vector<'_>, sup: &mut Vector<'_>) -> Result<(), Failure> {
        let (c1, c2) = (
            live_curve_of(&self.curv1, CLASS)?,
            live_curve_of(&self.curv2, CLASS)?,
        );
        vset(inf, 1, c1.first_parameter()?)?;
        vset(sup, 1, c1.last_parameter()?)?;
        vset(inf, 2, c2.first_parameter()?)?;
        vset(sup, 2, c2.last_parameter()?)
    }
    /// `IsSolution(Sol, Tol)`: both equations within `tol`.
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure> {
        let mut valsol = vector(2)?;
        self.try_value(sol, &mut valsol)?;
        Ok(vget(&valsol, 1)?.abs() <= tol && vget(&valsol, 2)?.abs() <= tol)
    }
}

impl BlendCurvPointFuncInv for BRepBlendCurvPointRadInv {
    /// `Set(P)`: the point the section ends on.
    fn set_point(&mut self, point: Point3) {
        self.point = point;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_native() {
        let null: Handle<dyn Adaptor3dCurve> = Handle::default();
        let mut inverse = BRepBlendCurvPointRadInv::new(&null, &null);
        inverse.set_choix(2);
        assert_eq!(
            (inverse.variables(), inverse.equations(), inverse.choix),
            (2, 2, 2)
        );
        let mut f = vector(2).unwrap();
        assert!(!inverse.value(&vector(2).unwrap(), &mut f));
        assert!(inverse.take_failure().is_err());
    }
}
