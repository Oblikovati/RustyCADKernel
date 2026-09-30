// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_MultiBSpCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_MultiBSpCurve`: a set of B-spline curves sharing their knots.
use super::multi_curve::{MultiCurve, MultiCurveEval};
use super::multi_point::MultiPoint;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bsplclib::{self, CurveSpan};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::fmt;
use std::ops::{Deref, DerefMut};

/// `AppParCurves_MultiBSpCurve`: the poles of an [`MultiCurve`] (reached through
/// `Deref`) with shared knots and multiplicities; the degree is
/// `Sum(Mults) - NbPoles - 1`, computed when the multiplicities are set.
///
/// ```
/// use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurve, MultiCurveEval, MultiPoint};
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point3;
/// let poles = MultiCurve::with_nb_poles(2)?;
/// for (i, x) in [(1, 0.0), (2, 2.0)] {
///     let pole = MultiPoint::with_counts(1, 0)?;
///     pole.set_point(1, Point3::new(x, 0.0, 0.0))?;
///     poles.set_value(i, &pole)?;
/// }
/// let curve = MultiBSpCurve::from_multi_curve(&poles, &Array1::from_shared(&[0.0, 1.0], 1), &Array1::from_shared(&[2, 2], 1));
/// assert_eq!(MultiCurveEval::degree(&curve)?, 1);
/// assert_eq!(curve.value_3d(1, 0.25)?, Point3::new(0.5, 0.0, 0.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct MultiBSpCurve {
    base: MultiCurve,
    knots: Handle<HArray1<f64>>,
    mults: Handle<HArray1<i32>>,
    degree: i32,
}

impl Deref for MultiBSpCurve {
    type Target = MultiCurve;
    fn deref(&self) -> &MultiCurve {
        &self.base
    }
}

impl DerefMut for MultiBSpCurve {
    fn deref_mut(&mut self) -> &mut MultiCurve {
        &mut self.base
    }
}

// ComputeDegree: Sum(Mults) - NbPoles - 1.
fn compute_degree(mults: &Array1<'_, i32>, nb_poles: i32) -> i32 {
    mults.iter().sum::<i32>() - nb_poles - 1
}

fn shared<T: Clone + Send + Sync + 'static>(values: &Array1<'_, T>) -> Handle<HArray1<T>> {
    Handle::new(HArray1::from_array(values))
}

impl MultiBSpCurve {
    /// `AppParCurves_MultiBSpCurve()`: no poles, degree 0; `MultiBSpCurve::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `AppParCurves_MultiBSpCurve(NbPol)`: empty MultiPoints, degree 0, no knots;
    /// `MultiBSpCurve::with_nb_poles(4)?`.
    pub fn with_nb_poles(nb_poles: i32) -> Result<Self, Failure> {
        Ok(Self {
            base: MultiCurve::with_nb_poles(nb_poles)?,
            ..Self::default()
        })
    }

    /// `AppParCurves_MultiBSpCurve(tabMU, Knots, Mults)`;
    /// `MultiBSpCurve::from_multi_points(&poles, &knots, &mults)`.
    pub fn from_multi_points(
        tab_mu: &Array1<'_, MultiPoint>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
    ) -> Self {
        Self::from_multi_curve(&MultiCurve::from_multi_points(tab_mu), knots, mults)
    }

    /// `AppParCurves_MultiBSpCurve(SC, Knots, Mults)`: shares the pole array of `sc`;
    /// `MultiBSpCurve::from_multi_curve(&curve, &knots, &mults)`.
    pub fn from_multi_curve(
        sc: &MultiCurve,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
    ) -> Self {
        let base: MultiCurve = sc.clone();
        let degree: i32 = compute_degree(mults, base.nb_poles());
        Self {
            base,
            knots: shared(knots),
            mults: shared(mults),
            degree,
        }
    }

    /// `SetKnots(theKnots)`: a copy with the same bounds; `curve.set_knots(&knots)`.
    pub fn set_knots(&mut self, knots: &Array1<'_, f64>) {
        self.knots = shared(knots);
    }

    /// `SetMultiplicities(theMults)`: a copy, and the degree recomputed;
    /// `curve.set_multiplicities(&mults)`.
    pub fn set_multiplicities(&mut self, mults: &Array1<'_, i32>) {
        self.mults = shared(mults);
        self.degree = compute_degree(mults, self.base.nb_poles());
    }

    /// `Knots()`: Standard_NullObject before any knots are set (a null dereference
    /// natively); `curve.knots()?`.
    pub fn knots(&self) -> Result<Array1<'static, f64>, Failure> {
        Ok(self
            .knots
            .get()
            .ok_or_else(|| missing("Knots"))?
            .array1()
            .clone())
    }

    /// `Multiplicities()`; see [`MultiBSpCurve::knots`]; `curve.multiplicities()?`.
    pub fn multiplicities(&self) -> Result<Array1<'static, i32>, Failure> {
        Ok(self
            .mults
            .get()
            .ok_or_else(|| missing("Multiplicities"))?
            .array1()
            .clone())
    }

    /// The `(AppParCurves_MultiCurve)` slicing copy: the Bezier view of the same
    /// shared poles; `curve.as_multi_curve()`.
    pub fn as_multi_curve(&self) -> MultiCurve {
        self.base.clone()
    }

    // The B-spline evaluation data: BSplCLib::D0/D1/D2 with index 0, not periodic.
    fn with_span<R>(
        &self,
        u: f64,
        evaluate: impl FnOnce(&CurveSpan<'_, '_>) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let knots: Array1<'static, f64> = self.knots()?;
        let mults: Array1<'static, i32> = self.multiplicities()?;
        let span = CurveSpan {
            u,
            index: 0,
            degree: self.degree,
            periodic: false,
            knots: &knots,
            mults: Some(&mults),
        };
        evaluate(&span)
    }
}

fn missing(member: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("AppParCurves_MultiBSpCurve::{member}: no knots or multiplicities set"),
    )
}

impl MultiCurveEval for MultiBSpCurve {
    fn nb_poles(&self) -> i32 {
        self.base.nb_poles()
    }

    fn degree(&self) -> Result<i32, Failure> {
        Ok(self.degree)
    }

    fn value_3d(&self, cu_index: i32, u: f64) -> Result<Point3, Failure> {
        let poles: Array1<'static, Point3> = self.base.poles_3d(cu_index)?;
        self.with_span(u, |span| bsplclib::d0(span, &poles, None))
    }

    fn value_2d(&self, cu_index: i32, u: f64) -> Result<Point2, Failure> {
        let poles: Array1<'static, Point2> = self.base.poles_2d(cu_index)?;
        self.with_span(u, |span| bsplclib::d0(span, &poles, None))
    }

    fn d1_3d(&self, cu_index: i32, u: f64) -> Result<(Point3, Vector3), Failure> {
        let poles: Array1<'static, Point3> = self.base.poles_3d(cu_index)?;
        self.with_span(u, |span| bsplclib::d1(span, &poles, None))
    }

    fn d1_2d(&self, cu_index: i32, u: f64) -> Result<(Point2, Vector2), Failure> {
        let poles: Array1<'static, Point2> = self.base.poles_2d(cu_index)?;
        self.with_span(u, |span| bsplclib::d1(span, &poles, None))
    }

    fn d2_3d(&self, cu_index: i32, u: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        let poles: Array1<'static, Point3> = self.base.poles_3d(cu_index)?;
        self.with_span(u, |span| bsplclib::d2(span, &poles, None))
    }

    fn d2_2d(&self, cu_index: i32, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        let poles: Array1<'static, Point2> = self.base.poles_2d(cu_index)?;
        self.with_span(u, |span| bsplclib::d2(span, &poles, None))
    }

    fn dump(&self, stream: &mut dyn fmt::Write) -> fmt::Result {
        let nb_curves: i32 = self.base.nb_curves().map_err(|_| fmt::Error)?;
        writeln!(stream, "AppParCurves_MultiBSpCurve dump:")?;
        writeln!(stream, " It contains {nb_curves} BSpline curves ")?;
        writeln!(stream, " The poles are: ")
    }
}
