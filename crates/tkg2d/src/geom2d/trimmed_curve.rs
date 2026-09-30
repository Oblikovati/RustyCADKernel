// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_TrimmedCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_TrimmedCurve`: a copy of a basis curve limited to `[U1, U2]`.
use super::bounded_curve::{Geom2dBoundedCurve, dump_bounded_curve};
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3};
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name, dump_field};
use crate::geom2d::trim_bounds::{TrimBasis, TrimmedBounds};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

type TrimmedState = TrimmedBounds<dyn Geom2dCurve>;

const CLASS: &str = "Geom2d_TrimmedCurve";

impl TrimBasis for dyn Geom2dCurve {
    fn trim_basis_range(&self) -> (f64, f64) {
        (self.first_parameter(), self.last_parameter())
    }
    fn trim_basis_periodic(&self) -> bool {
        self.is_periodic()
    }
    fn trim_basis_period(&self) -> Result<f64, Failure> {
        self.period()
    }
    fn trim_basis_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.reversed_parameter(u)
    }
    fn trim_basis_reverse(&self) -> Result<(), Failure> {
        self.reverse()
    }
}

/// A trimmed curve; `Handle::new(Geom2dTrimmedCurve::new(&circle.cast(), 0.0, PI, true, true)?)`.
///
/// The basis curve is a private copy, so its lock is never this object's lock.
pub struct Geom2dTrimmedCurve {
    parent: StandardTransient,
    state: SharedState<TrimmedState>,
}
impl_transient!(
    Geom2dTrimmedCurve,
    "Geom2d_TrimmedCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);

impl Geom2dTrimmedCurve {
    /// Trim a copy of `curve` to `[u1, u2]`; with `sense` false the result runs the
    /// other way, and `adjust_periodic` moves periodic bounds into the basis range
    /// (`Geom2d_TrimmedCurve(C, U1, U2, Sense, theAdjustPeriodic)`). A trimmed input
    /// contributes its own basis. Standard_ConstructionError for a null curve,
    /// `u1 == u2`, or bounds outside a non-periodic basis;
    /// `Geom2dTrimmedCurve::new(&curve, 0.0, 1.0, true, true)?`.
    pub fn new(
        curve: &Handle<dyn Geom2dCurve>,
        u1: f64,
        u2: f64,
        sense: bool,
        adjust_periodic: bool,
    ) -> Result<Self, Failure> {
        let Some(source) = curve.get() else {
            return Err(errors::construction("Geom2d_TrimmedCurve:: C is null"));
        };
        let trimmed: Handle<Geom2dTrimmedCurve> = curve.cast();
        let basis: Handle<dyn Geom2dCurve> = match trimmed.get() {
            Some(inner) => inner.basis_curve().copy()?.cast(),
            None => source.copy()?.cast(),
        };
        let mut state = TrimmedBounds {
            basis,
            u_trim1: u1,
            u_trim2: u2,
        };
        state.set_trim(u1, u2, sense, adjust_periodic, CLASS)?;
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// Change the bounds (`SetTrim`); see [`Geom2dTrimmedCurve::new`];
    /// `trimmed.set_trim(0.5, 1.0, true, true)?`.
    pub fn set_trim(
        &self,
        u1: f64,
        u2: f64,
        sense: bool,
        adjust_periodic: bool,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .set_trim(u1, u2, sense, adjust_periodic, CLASS)
    }
    /// The basis curve, shared with this object (`BasisCurve`); `trimmed.basis_curve()`.
    pub fn basis_curve(&self) -> Handle<dyn Geom2dCurve> {
        self.state.read().basis.clone()
    }
}
impl Geom2dGeometry for Geom2dTrimmedCurve {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.basis.transform(transform)?;
        let u1: f64 = state.basis.transformed_parameter(state.u_trim1, transform);
        let u2: f64 = state.basis.transformed_parameter(state.u_trim2, transform);
        state.set_trim(u1, u2, true, false, CLASS)
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        // The native Copy runs the constructor with the default Sense and
        // theAdjustPeriodic, so periodic bounds are adjusted again.
        let state = self.state.read();
        let copy = Self::new(&state.basis, state.u_trim1, state.u_trim2, true, true)?;
        Ok(Handle::new(copy).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom2d_TrimmedCurve")?;
        dump_base(stream, depth, dump_bounded_curve)?;
        dump_field(stream, depth, "basisCurve", |nested, level| {
            state.basis.dump_json(nested, level)
        })?;
        dump::dump_real(stream, "uTrim1", state.u_trim1)?;
        dump::dump_real(stream, "uTrim2", state.u_trim2)
    }
}
impl Geom2dCurve for Geom2dTrimmedCurve {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse(CLASS)
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let basis: Handle<dyn Geom2dCurve> = self.state.read().basis.clone();
        basis.reversed_parameter(u)
    }
    crate::basis_parameter_members!(Transform2);
    fn first_parameter(&self) -> f64 {
        self.state.read().u_trim1
    }
    fn last_parameter(&self) -> f64 {
        self.state.read().u_trim2
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        if self.state.read().spans_whole_periods()? {
            return Ok(true);
        }
        let first: Point2 = self.value(self.first_parameter())?;
        let last: Point2 = self.value(self.last_parameter())?;
        Ok(first.square_distance(last) <= precision::computational())
    }
    fn is_periodic(&self) -> bool {
        // Period() of a periodic basis cannot fail.
        self.state.read().spans_whole_periods().unwrap_or(false)
    }
    fn period(&self) -> Result<f64, Failure> {
        self.state.read().basis.period()
    }
    fn continuity(&self) -> Shape {
        self.state.read().basis.continuity()
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Geom2d_TrimmedCurve::IsCN: N = {n}; expected N >= 0"
            )));
        }
        self.state.read().basis.is_cn(n)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        self.state.read().basis.eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        self.state.read().basis.eval_d1(u)
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        self.state.read().basis.eval_d2(u)
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        self.state.read().basis.eval_d3(u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        self.state.read().basis.eval_dn(u, n)
    }
}
impl Geom2dBoundedCurve for Geom2dTrimmedCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        let state = self.state.read();
        state.basis.value(state.u_trim2)
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        let state = self.state.read();
        state.basis.value(state.u_trim1)
    }
}
