// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Profiler.hxx/.cxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Profiler`: section curves converted to B-splines of one degree, one
//! parameter range and, when possible, one knot vector.
use super::profiler_unify::{
    curve_at, unify_by_inserting_all_knots, unify_by_setting_middle_knots,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkgeombase::geom_convert::{ApproxCurve, curve_to_bspline_curve};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// Section curves brought to a common profile (`GeomFill_Profiler`), the base of
/// `GeomFill_SectionGenerator` and `GeomFill_Generator`.
///
/// `let mut p = GeomFillProfiler::new(); p.add_curve(&c1)?; p.add_curve(&c2)?; p.perform(1e-9)?;`
#[derive(Clone)]
pub struct GeomFillProfiler {
    pub(super) my_sequence: Vec<Handle<GeomBSplineCurve>>,
    pub(super) my_is_done: bool,
    pub(super) my_is_periodic: bool,
}

impl Default for GeomFillProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// The StdFail_NotDone of the consultation members before `Perform`; every native
/// message names `Degree`.
fn not_done() -> Failure {
    Failure::with_message(
        "StdFail_NotDone",
        "GeomFill_Profiler::Degree: the profile is not computed; expected Perform",
    )
}

impl GeomFillProfiler {
    /// An empty profile, periodic until a non-periodic curve is added
    /// (`GeomFill_Profiler()`); `GeomFillProfiler::new().is_periodic()`.
    pub fn new() -> Self {
        Self {
            my_sequence: Vec::new(),
            my_is_done: false,
            my_is_periodic: true,
        }
    }
    /// `AddCurve(Curve)`: a conic (possibly trimmed) is approximated as a C1
    /// B-spline within Precision::Confusion (16 spans of degree 14 at most, OCC7354),
    /// any other curve converted by `GeomConvert::CurveToBSplineCurve`; a null curve
    /// raises Standard_NullObject (a null dereference natively);
    /// `profiler.add_curve(&circle)?`.
    pub fn add_curve(&mut self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        if curve.is_null() {
            return Err(errors::null_object(
                "GeomFill_Profiler::AddCurve: null curve",
            ));
        }
        let converted: Handle<GeomBSplineCurve> = match conic_approximation(curve)? {
            Some(approximated) => approximated,
            None => curve_to_bspline_curve(curve, ParameterisationType::TgtThetaOver2)?,
        };
        let periodic: bool = live(&converted)?.is_periodic();
        self.my_sequence.push(converted);
        if self.my_is_periodic && !periodic {
            self.my_is_periodic = false;
        }
        Ok(())
    }
    /// `Perform(PTol)`: brings every curve to the highest degree and the widest
    /// knot range, then unifies the knots by inserting them all (`ptol` compares two
    /// knots) or, when the knot counts still differ, by setting the mean inner knots;
    /// an empty profile raises Standard_OutOfRange; `profiler.perform(1e-9)?`.
    pub fn perform(&mut self, ptol: f64) -> Result<(), Failure> {
        let (degree, u_first, u_last) = self.common_range()?;
        for curve in &self.my_sequence {
            let curve: &GeomBSplineCurve = live(curve)?;
            curve.increase_degree(degree)?;
            let mut knots: Array1<'static, f64> = curve.knots();
            bsplclib::reparametrize(u_first, u_last, &mut knots)?;
            curve.set_knots(&knots)?;
        }
        let copies: Vec<Handle<GeomBSplineCurve>> = self.copies()?;
        unify_by_inserting_all_knots(&copies, ptol)?;
        if same_knot_count(&copies)? {
            self.my_sequence = copies;
        } else {
            unify_by_setting_middle_knots(&self.my_sequence)?;
        }
        self.my_is_done = true;
        Ok(())
    }
    /// The first loop of `Perform`: periodic curves of a non-periodic profile are
    /// made non-periodic and segmented on their knot range; returns the highest
    /// degree and the widest knot range `(degree, first, last)`.
    fn common_range(&self) -> Result<(i32, f64, f64), Failure> {
        let (mut degree, mut u_first, mut u_last, mut ecart_max) = (0, 0.0, 0.0, 0.0);
        for curve in &self.my_sequence {
            let curve: &GeomBSplineCurve = live(curve)?;
            let u2: f64 = curve.knot(curve.last_u_knot_index()?)?;
            let u1: f64 = curve.knot(curve.first_u_knot_index()?)?;
            if !self.my_is_periodic && curve.is_periodic() {
                curve.set_not_periodic()?;
                curve.segment(u1, u2, precision::p_confusion())?;
            }
            degree = degree.max(curve.degree());
            if (u2 - u1) > ecart_max {
                (ecart_max, u_first, u_last) = (u2 - u1, u1, u2);
            }
        }
        Ok((degree, u_first, u_last))
    }
    /// `mySequence(i)->Copy()` of every curve.
    fn copies(&self) -> Result<Vec<Handle<GeomBSplineCurve>>, Failure> {
        self.my_sequence
            .iter()
            .map(|curve| Ok(Handle::new(GeomBSplineCurve::copy_of(live(curve)?)?)))
            .collect()
    }
    /// `Degree()`: StdFail_NotDone before `Perform`; `profiler.degree()?`.
    pub fn degree(&self) -> Result<i32, Failure> {
        Ok(self.done_first()?.degree())
    }
    /// `IsPeriodic()`: whether every added curve is periodic; `profiler.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        self.my_is_periodic
    }
    /// `NbPoles()`: StdFail_NotDone before `Perform`; `profiler.nb_poles()?`.
    pub fn nb_poles(&self) -> Result<i32, Failure> {
        Ok(self.done_first()?.nb_poles())
    }
    /// `Poles(Index, Poles)`: the poles of curve `index` assigned to `poles`, which
    /// must hold `NbPoles` values, and `index` must lie in `1..=NbCurves`
    /// (Standard_DomainError otherwise); `profiler.poles(1, &mut poles)?`.
    pub fn poles(&self, index: i32, poles: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        let curve: &GeomBSplineCurve = self.checked_curve(index, poles.len(), "Poles")?;
        poles.assign(&curve.poles());
        Ok(())
    }
    /// `Weights(Index, Weights)`: the weights of curve `index`, with the checks of
    /// `poles`; `profiler.weights(1, &mut weights)?`.
    pub fn weights(&self, index: i32, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let curve: &GeomBSplineCurve = self.checked_curve(index, weights.len(), "Weights")?;
        curve.weights_array(|stored| weights.assign(stored));
        Ok(())
    }
    /// `NbKnots()`: StdFail_NotDone before `Perform`; `profiler.nb_knots()?`.
    pub fn nb_knots(&self) -> Result<i32, Failure> {
        Ok(self.done_first()?.nb_knots())
    }
    /// `KnotsAndMults(Knots, Mults)`: the common knots and multiplicities; both
    /// arrays must hold `NbKnots` values (Standard_DomainError otherwise);
    /// `profiler.knots_and_mults(&mut knots, &mut mults)?`.
    pub fn knots_and_mults(
        &self,
        knots: &mut Array1<'_, f64>,
        mults: &mut Array1<'_, i32>,
    ) -> Result<(), Failure> {
        let n: i32 = self.nb_knots()?;
        if knots.len() as i32 != n || mults.len() as i32 != n {
            return Err(errors::domain(&format!(
                "GeomFill_Profiler::KnotsAndMults: {} knots and {} mults; expected {n} each",
                knots.len(),
                mults.len()
            )));
        }
        let first: &GeomBSplineCurve = curve_at(&self.my_sequence, 1)?;
        knots.assign(&first.knots());
        mults.assign(&first.multiplicities());
        Ok(())
    }
    /// `Curve(Index)`: the shared B-spline of section `index`; Standard_OutOfRange
    /// outside `1..=NbCurves`; `profiler.curve(1)?`.
    pub fn curve(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        curve_at(&self.my_sequence, index)?;
        Ok(self.my_sequence[(index - 1) as usize].cast::<dyn GeomCurve>())
    }
    /// The first curve after the done check of the consultation members.
    fn done_first(&self) -> Result<&GeomBSplineCurve, Failure> {
        if !self.my_is_done {
            return Err(not_done());
        }
        curve_at(&self.my_sequence, 1)
    }
    /// The done, length and index checks of `Poles` and `Weights`, in native order.
    fn checked_curve(
        &self,
        index: i32,
        length: usize,
        member: &str,
    ) -> Result<&GeomBSplineCurve, Failure> {
        let nb_poles: i32 = self.nb_poles()?;
        if length as i32 != nb_poles {
            return Err(errors::domain(&format!(
                "GeomFill_Profiler::{member}: array of {length}; expected {nb_poles} values"
            )));
        }
        if index < 1 || index > self.my_sequence.len() as i32 {
            return Err(errors::domain(&format!(
                "GeomFill_Profiler::{member}: index {index}; expected 1..={}",
                self.my_sequence.len()
            )));
        }
        curve_at(&self.my_sequence, index)
    }
}

/// The B-spline behind a converted curve handle.
fn live(curve: &Handle<GeomBSplineCurve>) -> Result<&GeomBSplineCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| errors::null_object("GeomFill_Profiler: null section curve"))
}

/// The OCC7354 branch of `AddCurve`: a conic, or a trimmed conic, approximated as a
/// C1 B-spline (the trimmed curve itself is approximated); `None` when the curve is
/// not a conic or the approximation has no result.
fn conic_approximation(
    curve: &Handle<dyn GeomCurve>,
) -> Result<Option<Handle<GeomBSplineCurve>>, Failure> {
    let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
    let basis: Handle<dyn GeomCurve> = match trimmed.get() {
        Some(trimmed_curve) if curve.is_instance_name("Geom_TrimmedCurve") => {
            trimmed_curve.basis_curve()
        }
        _ => curve.clone(),
    };
    if !basis.is_kind_name("Geom_Conic") {
        return Ok(None);
    }
    let approximation = ApproxCurve::from_curve(curve, precision::confusion(), Shape::C1, 16, 14)?;
    Ok(approximation.has_result().then(|| approximation.curve()))
}

/// Whether every copy has the knot count of the first one (`Unified`).
fn same_knot_count(copies: &[Handle<GeomBSplineCurve>]) -> Result<bool, Failure> {
    let count: i32 = curve_at(copies, 1)?.nb_knots();
    for i in 2..=copies.len() as i32 {
        if curve_at(copies, i)?.nb_knots() != count {
            return Ok(false);
        }
    }
    Ok(true)
}
