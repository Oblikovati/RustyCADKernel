// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_PlanFunc.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_PlanFunc`: the signed distance `V . (C(x) - P)` from a curve point to
//! the plane of normal `V` through `P`, whose root is the plane/curve intersection.
use super::sweep_failures::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `GeomFill_PlanFunc`: a `math_FunctionWithDerivative` of the curve parameter.
///
/// A curve exception is kept and the callback answers false (docs/geomalgo-design.md,
/// section 14); [`GeomFillPlanFunc::take_failure`] raises it after the solver.
/// `let mut f = GeomFillPlanFunc::new(p, v, &curve); f.value(0.5, &mut out);`.
pub struct GeomFillPlanFunc {
    my_pnt: Xyz,
    my_vec: Xyz,
    my_curve: Handle<dyn Adaptor3dCurve>,
    capture: FailureCapture,
}

/// `-myPnt + G`, the native `V.SetLinearForm(-1, myPnt, G.XYZ())`.
fn relative(my_pnt: Xyz, g: Point3) -> Xyz {
    let mut v = Xyz::default();
    v.set_scaled_sum(-1.0, my_pnt, g.xyz());
    v
}

impl GeomFillPlanFunc {
    /// `GeomFill_PlanFunc(P, V, C)`; `GeomFillPlanFunc::new(p, v, &curve)`.
    pub fn new(p: Point3, v: Vector3, curve: &Handle<dyn Adaptor3dCurve>) -> Self {
        Self {
            my_pnt: p.xyz(),
            my_vec: *v.xyz(),
            my_curve: curve.clone(),
            capture: FailureCapture::default(),
        }
    }

    fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        live(&self.my_curve, "GeomFill_PlanFunc", "myCurve")
    }

    /// The curve exception a callback met, if any; `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    fn try_value(&self, x: f64) -> Result<f64, Failure> {
        Ok(self.my_vec.dot(relative(self.my_pnt, self.curve()?.d0(x)?)))
    }

    fn try_values(&self, x: f64) -> Result<(f64, f64), Failure> {
        let (g, dg) = self.curve()?.d1(x)?;
        Ok((
            self.my_vec.dot(relative(self.my_pnt, g)),
            self.my_vec.dot(*dg.xyz()),
        ))
    }

    /// `D2(X, F, D1, D2)`: natively empty, the outputs are left untouched;
    /// `f.d2(0.5, &mut v, &mut d1, &mut d2)`.
    pub fn d2(&self, x: f64, f: &mut f64, d1: &mut f64, d2: &mut f64) {
        let _ = (x, f, d1, d2);
    }

    /// `DEDT(X, DP, DV, DF)`: the derivative of the function with respect to the
    /// plane motion (`DP`, `DV`); `f.dedt(0.5, dp, dv)?`.
    pub fn dedt(&self, x: f64, dp: Vector3, dv: Vector3) -> Result<f64, Failure> {
        let v = relative(self.my_pnt, self.curve()?.d0(x)?);
        Ok(dv.dot(Vector3::from(v)) - self.my_vec.dot(*dp.xyz()))
    }

    /// `D2E(X, DP, D2P, DV, D2V, DFDT, D2FDT2, D2FDTDX)`: natively only `DFDT` is
    /// written, so this returns it (the second-order inputs are unused);
    /// `f.d2e(0.5, dp, d2p, dv, d2v)?`.
    pub fn d2e(
        &self,
        x: f64,
        dp: Vector3,
        d2p: Vector3,
        dv: Vector3,
        d2v: Vector3,
    ) -> Result<f64, Failure> {
        let _ = (d2p, d2v);
        let (g, _) = self.curve()?.d1(x)?;
        let v = relative(self.my_pnt, g);
        let mut dvdt = Xyz::default();
        dvdt.set_scaled_sum(-1.0, *dp.xyz(), g.xyz());
        Ok(dv.dot(Vector3::from(v)) + self.my_vec.dot(dvdt))
    }
}

impl Function for GeomFillPlanFunc {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value = self.try_value(x);
        let Some(value) = self.capture.keep(value) else {
            return false;
        };
        *result = value;
        true
    }
}

impl FunctionWithDerivative for GeomFillPlanFunc {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let values = self.try_values(x);
        let Some((_, d)) = self.capture.keep(values) else {
            return false;
        };
        *derivative = d;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let values = self.try_values(x);
        let Some((f, d)) = self.capture.keep(values) else {
            return false;
        };
        *value = f;
        *derivative = d;
        true
    }
}
