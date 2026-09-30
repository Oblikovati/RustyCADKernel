// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_BSpFunc.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Law_BSpFunc`: a law evaluated on a `Law_BSpline` over a parameter range.
use super::bsp_func_intervals::IntervalKnots;
use super::{LawBSpline, LawFunction};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::geom_abs::Shape;

/// The fields of a B-spline law: the shared curve and the range.
#[derive(Clone, Default)]
pub(crate) struct BSpFuncFields {
    pub(crate) curv: Handle<LawBSpline>,
    pub(crate) first: f64,
    pub(crate) last: f64,
}

/// A law defined by a scalar B-spline between two parameters (`Law_BSpFunc`); the
/// curve is shared with the handle it was given.
///
/// `let law = LawBSpFunc::with_curve(&curve, 0.0, 1.0); law.value(0.5)?`.
#[derive(Default)]
pub struct LawBSpFunc {
    parent: StandardTransient,
    fields: SharedState<BSpFuncFields>,
}
impl_transient!(LawBSpFunc, "Law_BSpFunc", dyn LawFunction, parent, interfaces: [dyn LawFunction]);

/// `PosTol`: half of `Precision::PConfusion`.
fn pos_tol() -> f64 {
    precision::p_confusion() / 2.0
}

impl LawBSpFunc {
    /// A law without curve on `[0, 0]` (`Law_BSpFunc()`); its evaluators raise
    /// Standard_NullObject until a curve is set. `LawBSpFunc::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The law of `c` on `[first, last]` (`Law_BSpFunc(C, First, Last)`).
    /// `LawBSpFunc::with_curve(&curve, 0.0, 1.0)`.
    pub fn with_curve(c: &Handle<LawBSpline>, first: f64, last: f64) -> Self {
        let law = Self::default();
        *law.fields.write() = BSpFuncFields {
            curv: c.clone(),
            first,
            last,
        };
        law
    }

    /// The curve (`Curve`), possibly null; `law.curve()`.
    pub fn curve(&self) -> Handle<LawBSpline> {
        self.fields.read().curv.clone()
    }

    /// Sets the curve and takes its parameter range (`SetCurve`);
    /// Standard_NullObject for a null curve. `law.set_curve(&curve)?`.
    pub fn set_curve(&self, c: &Handle<LawBSpline>) -> Result<(), Failure> {
        let curve: &LawBSpline = live_curve(c)?;
        let (first, last) = (curve.first_parameter(), curve.last_parameter());
        *self.fields.write() = BSpFuncFields {
            curv: c.clone(),
            first,
            last,
        };
        Ok(())
    }

    /// A copy of the fields, so that no lock is held while the curve evaluates.
    pub(crate) fn snapshot(&self) -> BSpFuncFields {
        self.fields.read().clone()
    }
}

/// The curve behind a handle, or the Standard_NullObject of its native dereference.
pub(crate) fn live_curve(curv: &Handle<LawBSpline>) -> Result<&LawBSpline, Failure> {
    curv.get()
        .ok_or_else(|| errors::null_object("Law_BSpFunc: the law has no Law_BSpline curve"))
}

impl BSpFuncFields {
    /// The knot span `(Ideb, Ifin)` used at the range ends: `LocateU` of the end,
    /// widened to a non-empty span inside the knots.
    fn end_span(&self, curve: &LawBSpline, x: f64) -> Result<(i32, i32), Failure> {
        let (mut ideb, mut ifin): (i32, i32) = (0, 0);
        if x == self.first {
            (ideb, ifin) = curve.locate_u(self.first, pos_tol(), false)?;
            ideb = ideb.max(1);
            if ideb >= ifin {
                ifin = ideb + 1;
            }
        }
        if x == self.last {
            (ideb, ifin) = curve.locate_u(self.last, pos_tol(), false)?;
            ifin = ifin.min(curve.nb_knots());
            if ideb >= ifin {
                ideb = ifin - 1;
            }
        }
        Ok((ideb, ifin))
    }

    fn at_end(&self, x: f64) -> bool {
        x == self.first || x == self.last
    }

    pub(crate) fn value(&self, x: f64) -> Result<f64, Failure> {
        let curve: &LawBSpline = live_curve(&self.curv)?;
        if !self.at_end(x) {
            return curve.value(x);
        }
        let (ideb, ifin) = self.end_span(curve, x)?;
        curve.local_value(x, ideb, ifin)
    }

    pub(crate) fn d1(&self, x: f64) -> Result<(f64, f64), Failure> {
        let curve: &LawBSpline = live_curve(&self.curv)?;
        if !self.at_end(x) {
            return curve.d1(x);
        }
        let (ideb, ifin) = self.end_span(curve, x)?;
        curve.local_d1(x, ideb, ifin)
    }

    pub(crate) fn d2(&self, x: f64) -> Result<(f64, f64, f64), Failure> {
        let curve: &LawBSpline = live_curve(&self.curv)?;
        if !self.at_end(x) {
            return curve.d2(x);
        }
        let (ideb, ifin) = self.end_span(curve, x)?;
        curve.local_d2(x, ideb, ifin)
    }

    pub(crate) fn nb_intervals(&self, s: Shape) -> Result<i32, Failure> {
        let curve: &LawBSpline = live_curve(&self.curv)?;
        match IntervalKnots::compute(curve, self.first, self.last, s)? {
            Some(knots) => Ok(knots.count()),
            None => Ok(1),
        }
    }

    pub(crate) fn intervals(&self, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        let curve: &LawBSpline = live_curve(&self.curv)?;
        let mut nb_intervals: i32 = 1;
        if let Some(knots) = IntervalKnots::compute(curve, self.first, self.last, s)? {
            nb_intervals = knots.store(t)?;
        }
        let lower: i32 = t.lower();
        put(t, lower, self.first)?;
        put(t, lower + nb_intervals, self.last)
    }
}

impl LawFunction for LawBSpFunc {
    /// The continuity of the curve.
    fn continuity(&self) -> Result<Shape, Failure> {
        Ok(live_curve(&self.snapshot().curv)?.continuity())
    }
    /// The number of knot spans of the curve inside the range where it is less than
    /// C`s`; Standard_DomainError for G1 and G2.
    fn nb_intervals(&self, s: Shape) -> Result<i32, Failure> {
        self.snapshot().nb_intervals(s)
    }
    fn intervals(&self, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        self.snapshot().intervals(t, s)
    }
    /// The curve value; at the range ends it evaluates on the span inside the range.
    fn value(&self, x: f64) -> Result<f64, Failure> {
        self.snapshot().value(x)
    }
    fn d1(&self, x: f64) -> Result<(f64, f64), Failure> {
        self.snapshot().d1(x)
    }
    fn d2(&self, x: f64) -> Result<(f64, f64, f64), Failure> {
        self.snapshot().d2(x)
    }
    /// A `Law_BSpFunc` on the same curve and the new range.
    fn trim(
        &self,
        p_first: f64,
        p_last: f64,
        _tol: f64,
    ) -> Result<Handle<dyn LawFunction>, Failure> {
        let curve: Handle<LawBSpline> = self.snapshot().curv;
        Ok(Handle::new(LawBSpFunc::with_curve(&curve, p_first, p_last)).cast())
    }
    fn bounds(&self) -> Result<(f64, f64), Failure> {
        let fields: BSpFuncFields = self.snapshot();
        Ok((fields.first, fields.last))
    }
}
