// Rust translation of the parameter law that GeomFill_SimpleBound.cxx and
// GeomFill_BoundWithSurf.cxx repeat (myPar), at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reparametrization law of `GeomFill_SimpleBound` and `GeomFill_BoundWithSurf`
//! (`myPar`): null until `Reparametrize`, then the `Law_BSpFunc` of
//! `Law::Reparametrize` on 30 points.
use crate::law::{self, LawBSpFunc, LawFunction};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;

/// `occ::handle<Law_Function> myPar`, shared state of a boundary.
#[derive(Default)]
pub(super) struct BoundParameter {
    law: SharedState<Handle<dyn LawFunction>>,
}

impl BoundParameter {
    /// `x = U`, or `myPar->Value(U)` once reparametrized.
    pub(super) fn value(&self, u: f64) -> Result<f64, Failure> {
        match self.law.read().get() {
            Some(law) => law.value(u),
            None => Ok(u),
        }
    }
    /// `x = U, dx = 1`, or `myPar->D1(U, x, dx)` once reparametrized.
    pub(super) fn d1(&self, u: f64) -> Result<(f64, f64), Failure> {
        match self.law.read().get() {
            Some(law) => law.d1(u),
            None => Ok((u, 1.)),
        }
    }
    /// `myPar->Bounds(First, Last)` once reparametrized, `curve_bounds()` before.
    pub(super) fn bounds(
        &self,
        curve_bounds: impl FnOnce() -> Result<(f64, f64), Failure>,
    ) -> Result<(f64, f64), Failure> {
        match self.law.read().get() {
            Some(law) => law.bounds(),
            None => curve_bounds(),
        }
    }
    /// `Reparametrize`: `Law::Reparametrize(curve, ..., 30)`, then a new `Law_BSpFunc`
    /// on that law (stored before `SetCurve`, as natively).
    #[allow(clippy::too_many_arguments)] // The native member signature.
    pub(super) fn reparametrize(
        &self,
        curve: &dyn Adaptor3dCurve,
        first: f64,
        last: f64,
        (has_df, has_dl): (bool, bool),
        (df, dl): (f64, f64),
        rev: bool,
    ) -> Result<(), Failure> {
        let curve = law::reparametrize(curve, first, last, has_df, has_dl, df, dl, rev, 30)?;
        let function: Handle<LawBSpFunc> = Handle::new(LawBSpFunc::new());
        *self.law.write() = function.cast::<dyn LawFunction>();
        function.set_curve(&curve)
    }
}
