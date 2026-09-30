// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_Function.hxx/.cxx, Law_Constant.hxx/.cxx
// and Law_Linear.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Law_Function`, the abstract scalar law, and its two closed-form laws
//! `Law_Constant` and `Law_Linear`.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkmath::geom_abs::Shape;

/// A scalar function of one parameter (`Law_Function`), reached through
/// `Handle<dyn LawFunction>`.
///
/// Every member returns `Result`, because an override can raise (the composite law
/// raises Standard_NotImplemented from `Continuity`, the B-spline laws raise from the
/// B-spline evaluators). The evaluators take `&self`: the laws that change state while
/// they evaluate (the composite law caches its current piece) keep it behind a lock.
/// C++ output references become returned tuples.
/// `let law: Handle<dyn LawFunction> = Handle::new(LawConstant::new()).cast(); law.value(0.5)?`.
pub trait LawFunction: Transient {
    /// The continuity of the law (`Continuity`).
    fn continuity(&self) -> Result<Shape, Failure>;
    /// The number of intervals of continuity `s` (`NbIntervals`).
    fn nb_intervals(&self, s: Shape) -> Result<i32, Failure>;
    /// Stores the bounds of the intervals of continuity `s` in `t`, which must hold
    /// `NbIntervals + 1` values (`Intervals`).
    fn intervals(&self, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure>;
    /// The value at `x` (`Value`).
    fn value(&self, x: f64) -> Result<f64, Failure>;
    /// The value and first derivative at `x` (`D1`).
    fn d1(&self, x: f64) -> Result<(f64, f64), Failure>;
    /// The value and two derivatives at `x` (`D2`).
    fn d2(&self, x: f64) -> Result<(f64, f64, f64), Failure>;
    /// A law equal to this one on `[p_first, p_last]` (`Trim`); `tol` tests point
    /// confusion.
    fn trim(&self, p_first: f64, p_last: f64, tol: f64)
    -> Result<Handle<dyn LawFunction>, Failure>;
    /// The parametric bounds `(PFirst, PLast)` (`Bounds`).
    fn bounds(&self) -> Result<(f64, f64), Failure>;
}
impl_transient_interface!(dyn LawFunction, "Law_Function", StandardTransient);

/// `T(T.Lower()) = first; T(T.Upper()) = last`, the `Intervals` of the one-piece laws.
fn store_bounds(t: &mut Array1<'_, f64>, first: f64, last: f64) -> Result<(), Failure> {
    let (lower, upper) = (t.lower(), t.upper());
    put(t, lower, first)?;
    put(t, upper, last)
}

/// The fields of a constant law.
#[derive(Clone, Copy, Debug, Default)]
struct ConstantFields {
    radius: f64,
    first: f64,
    last: f64,
}

/// A constant law on a range (`Law_Constant`), infinitely continuous.
///
/// `let law = LawConstant::new(); law.set(2.0, 0.0, 1.0); law.value(0.5)? == 2.0`.
#[derive(Default)]
pub struct LawConstant {
    parent: StandardTransient,
    fields: SharedState<ConstantFields>,
}
impl_transient!(LawConstant, "Law_Constant", dyn LawFunction, parent, interfaces: [dyn LawFunction]);

impl LawConstant {
    /// The constant 0 on `[0, 0]` (`Law_Constant()`); `LawConstant::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the value and the range (`Set`); `law.set(2.0, 0.0, 1.0)`.
    pub fn set(&self, radius: f64, p_first: f64, p_last: f64) {
        *self.fields.write() = ConstantFields {
            radius,
            first: p_first,
            last: p_last,
        };
    }
}

impl LawFunction for LawConstant {
    /// Always CN.
    fn continuity(&self) -> Result<Shape, Failure> {
        Ok(Shape::CN)
    }
    /// Always 1.
    fn nb_intervals(&self, _s: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    fn intervals(&self, t: &mut Array1<'_, f64>, _s: Shape) -> Result<(), Failure> {
        let fields: ConstantFields = *self.fields.read();
        store_bounds(t, fields.first, fields.last)
    }
    fn value(&self, _x: f64) -> Result<f64, Failure> {
        Ok(self.fields.read().radius)
    }
    fn d1(&self, _x: f64) -> Result<(f64, f64), Failure> {
        Ok((self.fields.read().radius, 0.0))
    }
    fn d2(&self, _x: f64) -> Result<(f64, f64, f64), Failure> {
        Ok((self.fields.read().radius, 0.0, 0.0))
    }
    fn trim(
        &self,
        p_first: f64,
        p_last: f64,
        _tol: f64,
    ) -> Result<Handle<dyn LawFunction>, Failure> {
        let law = LawConstant::new();
        law.set(self.fields.read().radius, p_first, p_last);
        Ok(Handle::new(law).cast())
    }
    fn bounds(&self) -> Result<(f64, f64), Failure> {
        let fields: ConstantFields = *self.fields.read();
        Ok((fields.first, fields.last))
    }
}

/// The fields of a linear law.
#[derive(Clone, Copy, Debug, Default)]
struct LinearFields {
    valdeb: f64,
    valfin: f64,
    pdeb: f64,
    pfin: f64,
}

impl LinearFields {
    /// `((X - pdeb) * valfin + (pfin - X) * valdeb) / (pfin - pdeb)`.
    fn at(&self, x: f64) -> f64 {
        ((x - self.pdeb) * self.valfin + (self.pfin - x) * self.valdeb) / (self.pfin - self.pdeb)
    }

    fn slope(&self) -> f64 {
        (self.valfin - self.valdeb) / (self.pfin - self.pdeb)
    }
}

/// A linear law between two parameters (`Law_Linear`), infinitely continuous; a
/// degenerate range divides by zero, as natively.
///
/// `let law = LawLinear::new(); law.set(0.0, 1.0, 1.0, 3.0); law.value(0.5)? == 2.0`.
#[derive(Default)]
pub struct LawLinear {
    parent: StandardTransient,
    fields: SharedState<LinearFields>,
}
impl_transient!(LawLinear, "Law_Linear", dyn LawFunction, parent, interfaces: [dyn LawFunction]);

impl LawLinear {
    /// The law 0 on `[0, 0]` (`Law_Linear()`); `LawLinear::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// The law from `valdeb` at `pdeb` to `valfin` at `pfin` (`Set`).
    /// `law.set(0.0, 1.0, 1.0, 3.0)`.
    pub fn set(&self, pdeb: f64, valdeb: f64, pfin: f64, valfin: f64) {
        *self.fields.write() = LinearFields {
            valdeb,
            valfin,
            pdeb,
            pfin,
        };
    }
}

impl LawFunction for LawLinear {
    /// Always CN.
    fn continuity(&self) -> Result<Shape, Failure> {
        Ok(Shape::CN)
    }
    /// Always 1.
    fn nb_intervals(&self, _s: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    fn intervals(&self, t: &mut Array1<'_, f64>, _s: Shape) -> Result<(), Failure> {
        let fields: LinearFields = *self.fields.read();
        store_bounds(t, fields.pdeb, fields.pfin)
    }
    fn value(&self, x: f64) -> Result<f64, Failure> {
        Ok(self.fields.read().at(x))
    }
    fn d1(&self, x: f64) -> Result<(f64, f64), Failure> {
        let fields: LinearFields = *self.fields.read();
        Ok((fields.at(x), fields.slope()))
    }
    fn d2(&self, x: f64) -> Result<(f64, f64, f64), Failure> {
        let fields: LinearFields = *self.fields.read();
        Ok((fields.at(x), fields.slope(), 0.0))
    }
    fn trim(
        &self,
        p_first: f64,
        p_last: f64,
        _tol: f64,
    ) -> Result<Handle<dyn LawFunction>, Failure> {
        let fields: LinearFields = *self.fields.read();
        let law = LawLinear::new();
        law.set(p_first, fields.at(p_first), p_last, fields.at(p_last));
        Ok(Handle::new(law).cast())
    }
    fn bounds(&self) -> Result<(f64, f64), Failure> {
        let fields: LinearFields = *self.fields.read();
        Ok((fields.pdeb, fields.pfin))
    }
}
