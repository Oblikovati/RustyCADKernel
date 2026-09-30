// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Sweep.hxx/.cxx
// (the class interface, Build and the result accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Sweep`: the surface swept by a section law along a location law, as
//! an elementary surface when the laws allow it (`BuildKPart`) or else as the
//! B-spline approximation of `GeomFill_SweepFunction` (docs/geomalgo-design.md,
//! section 14).
use super::approx_style::GeomFillApproxStyle;
use super::location_law::GeomFillLocationLaw;
use super::section_law::GeomFillSectionLaw;
use super::sweep_failures::{live, sweep_failure};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::geom_abs::Shape;

/// The native marker of a section domain not set by `SetDomain` (`30.081996`).
pub(crate) const UNSET_SECTION_DOMAIN: f64 = 30.081996;

/// `GeomFill_Sweep`: the sweep of a section law along a location law.
///
/// Natively `myExchUV`, `isUReversed` and `isVReversed` are not initialized before
/// the first `Build`; here they start false.
/// `let mut sweep = GeomFillSweep::new(&location, true)?; sweep.build(&section,
/// GeomFillApproxStyle::Location, Shape::C2, 10, 30)?;`.
pub struct GeomFillSweep {
    pub(crate) first: f64,
    pub(crate) last: f64,
    pub(crate) s_first: f64,
    pub(crate) s_last: f64,
    pub(crate) tol3d: f64,
    pub(crate) bound_tol: f64,
    pub(crate) tol2d: f64,
    pub(crate) tol_angular: f64,
    pub(crate) s_error: f64,
    pub(crate) my_force_approx_c1: bool,
    pub(crate) my_loc: Handle<dyn GeomFillLocationLaw>,
    pub(crate) my_sec: Handle<dyn GeomFillSectionLaw>,
    pub(crate) my_surface: Handle<dyn GeomSurface>,
    pub(crate) my_curve2d: Option<Array1<'static, Handle<dyn Geom2dCurve>>>,
    pub(crate) c_error: Option<Array2<'static, f64>>,
    pub(crate) done: bool,
    pub(crate) my_exch_uv: bool,
    pub(crate) is_u_reversed: bool,
    pub(crate) is_v_reversed: bool,
    pub(crate) my_k_part: bool,
}

impl GeomFillSweep {
    /// `GeomFill_Sweep(Location, WithKpart)`: the path domain of the law, the
    /// default tolerances and an unset section domain;
    /// `GeomFillSweep::new(&location, true)?`.
    pub fn new(
        location: &Handle<dyn GeomFillLocationLaw>,
        with_kpart: bool,
    ) -> Result<Self, Failure> {
        let (first, last) = live(location, "GeomFill_Sweep", "Location")?.get_domain()?;
        let mut sweep = Self {
            first,
            last,
            s_first: UNSET_SECTION_DOMAIN,
            s_last: UNSET_SECTION_DOMAIN,
            tol3d: 0.0,
            bound_tol: 0.0,
            tol2d: 0.0,
            tol_angular: 0.0,
            s_error: f64::MAX,
            my_force_approx_c1: false,
            my_loc: location.clone(),
            my_sec: Handle::default(),
            my_surface: Handle::default(),
            my_curve2d: None,
            c_error: None,
            done: false,
            my_exch_uv: false,
            is_u_reversed: false,
            is_v_reversed: false,
            my_k_part: with_kpart,
        };
        sweep.set_tolerance(1.0e-4, 1.0, 1.0e-5, 1.0);
        Ok(sweep)
    }

    pub(crate) fn loc(&self) -> Result<&dyn GeomFillLocationLaw, Failure> {
        live(&self.my_loc, "GeomFill_Sweep", "myLoc")
    }

    pub(crate) fn sec(&self) -> Result<&dyn GeomFillSectionLaw, Failure> {
        live(&self.my_sec, "GeomFill_Sweep", "mySec")
    }

    /// `SetDomain(LocFirst, LocLast, SectionFirst, SectionLast)`;
    /// `sweep.set_domain(0.0, 1.0, 0.0, 1.0)`.
    pub fn set_domain(&mut self, loc_first: f64, loc_last: f64, s_first: f64, s_last: f64) {
        (self.first, self.last) = (loc_first, loc_last);
        (self.s_first, self.s_last) = (s_first, s_last);
    }

    /// `SetTolerance(Tolerance3d, BoundTolerance, Tolerance2d, ToleranceAngular)`
    /// (natively defaulted to 1, 1e-5 and 1); `sweep.set_tolerance(1e-4, 1.0, 1e-5, 1.0)`.
    pub fn set_tolerance(&mut self, tol3d: f64, bound_tol: f64, tol2d: f64, tol_angular: f64) {
        (self.tol3d, self.bound_tol) = (tol3d, bound_tol);
        (self.tol2d, self.tol_angular) = (tol2d, tol_angular);
    }

    /// `SetForceApproxC1(ForceApproxC1)`: approximates a non-C1 result by a C1
    /// surface; `sweep.set_force_approx_c1(true)`.
    pub fn set_force_approx_c1(&mut self, force_approx_c1: bool) {
        self.my_force_approx_c1 = force_approx_c1;
    }

    /// `ExchangeUV()`: whether the surface U is the path; `sweep.exchange_uv()`.
    pub fn exchange_uv(&self) -> bool {
        self.my_exch_uv
    }

    /// `UReversed()`; `sweep.u_reversed()`.
    pub fn u_reversed(&self) -> bool {
        self.is_u_reversed
    }

    /// `VReversed()`; `sweep.v_reversed()`.
    pub fn v_reversed(&self) -> bool {
        self.is_v_reversed
    }

    /// `Build(Section, Methode, Continuity, Degmax, Segmax)` (natively defaulted to
    /// `GeomFill_Location`, C2, 10 and 30): an elementary surface when the laws
    /// allow it, else the approximation;
    /// `sweep.build(&section, GeomFillApproxStyle::Location, Shape::C2, 10, 30)?`.
    pub fn build(
        &mut self,
        section: &Handle<dyn GeomFillSectionLaw>,
        methode: GeomFillApproxStyle,
        continuity: Shape,
        degmax: i32,
        segmax: i32,
    ) -> Result<(), Failure> {
        self.done = false;
        self.my_exch_uv = false;
        (self.is_u_reversed, self.is_v_reversed) = (false, false);
        self.my_sec = section.clone();
        if self.s_first == self.s_last && self.s_last == UNSET_SECTION_DOMAIN {
            (self.s_first, self.s_last) = self.sec()?.get_domain()?;
        }
        let is_k_part = self.my_k_part && self.build_k_part()?;
        if !is_k_part {
            self.my_exch_uv = false;
            (self.is_u_reversed, self.is_v_reversed) = (false, false);
            if methode == GeomFillApproxStyle::Location {
                // The native code reads the product surface and drops it.
                self.sec()?.bspline_surface()?;
            }
            self.done = self.build_all(continuity, degmax, segmax)?;
            return Ok(());
        }
        self.done = self.build_2d();
        Ok(())
    }

    /// `Build2d`: the 2d curves of an elementary result; true only without traces.
    fn build_2d(&self) -> bool {
        self.my_loc.get().is_some_and(|l| l.nb_2d_curves() == 0)
    }

    /// `IsDone()`; `sweep.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `ErrorOnSurface()`: the approximation error; `sweep.error_on_surface()`.
    pub fn error_on_surface(&self) -> f64 {
        self.s_error
    }

    /// `ErrorOnRestriction(IsFirst, UError, VError)`: the errors of the first or
    /// last restriction; `let (eu, ev) = sweep.error_on_restriction(true)?;`.
    pub fn error_on_restriction(&self, is_first: bool) -> Result<(f64, f64), Failure> {
        let member = "GeomFill_Sweep::ErrorOnRestriction";
        let ind = if is_first {
            1
        } else {
            self.curves_2d(member)?.len() as i32
        };
        self.errors_at(member, ind)
    }

    /// `ErrorOnTrace(IndexOfTrace, UError, VError)`: Standard_OutOfRange above the
    /// trace count; `let (eu, ev) = sweep.error_on_trace(1)?;`.
    pub fn error_on_trace(&self, index_of_trace: i32) -> Result<(f64, f64), Failure> {
        let member = "GeomFill_Sweep::ErrorOnTrace";
        self.check_trace(index_of_trace, member)?;
        self.errors_at(member, index_of_trace + 1)
    }

    /// `Surface()`: the result (null before a successful build); `sweep.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.my_surface.clone()
    }

    /// `Restriction(IsFirst)`: the 2d curve of the first or last section;
    /// `let c = sweep.restriction(true)?;`.
    pub fn restriction(&self, is_first: bool) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let curves = self.curves_2d("GeomFill_Sweep::Restriction")?;
        let index = if is_first { 1 } else { curves.len() as i32 };
        curve_at(curves, index)
    }

    /// `NumberOfTrace()`; `sweep.number_of_trace()?`.
    pub fn number_of_trace(&self) -> Result<i32, Failure> {
        Ok(self.loc()?.trace_number())
    }

    /// `Trace(IndexOfTrace)`: Standard_OutOfRange above the trace count;
    /// `let c = sweep.trace(1)?;`.
    pub fn trace(&self, index_of_trace: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let member = "GeomFill_Sweep::Trace";
        self.check_trace(index_of_trace, member)?;
        curve_at(self.curves_2d(member)?, index_of_trace + 1)
    }

    fn check_trace(&self, index_of_trace: i32, member: &str) -> Result<(), Failure> {
        let trace_number = self.loc()?.trace_number();
        if index_of_trace > trace_number {
            return Err(sweep_failure(
                "Standard_OutOfRange",
                &format!(" {member} (IndexOfTrace = {index_of_trace}; expected <= {trace_number})"),
            ));
        }
        Ok(())
    }

    fn curves_2d(
        &self,
        member: &str,
    ) -> Result<&Array1<'static, Handle<dyn Geom2dCurve>>, Failure> {
        self.my_curve2d
            .as_ref()
            .ok_or_else(|| super::sweep_failures::null_object(member, "myCurve2d"))
    }

    fn errors_at(&self, member: &str, ind: i32) -> Result<(f64, f64), Failure> {
        let c_error = self
            .c_error
            .as_ref()
            .ok_or_else(|| super::sweep_failures::null_object(member, "CError"))?;
        let at = |row: i32| {
            c_error
                .value(row, ind)
                .copied()
                .map_err(|e| array_failure(member, row, ind, e))
        };
        Ok((at(1)?, at(2)?))
    }
}

fn curve_at(
    curves: &Array1<'static, Handle<dyn Geom2dCurve>>,
    index: i32,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    curves.value(index).cloned().map_err(|e| {
        sweep_failure(
            "Standard_OutOfRange",
            &format!(
                "GeomFill_Sweep: myCurve2d({index}): {e}; expected an index in the array bounds"
            ),
        )
    })
}

fn array_failure(member: &str, row: i32, col: i32, e: impl std::fmt::Display) -> Failure {
    sweep_failure(
        "Standard_OutOfRange",
        &format!("{member}: CError({row}, {col}): {e}; expected an index in the array bounds"),
    )
}
