// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_ApproxCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_ApproxCurve`: the polynomial B-spline approximation of a 3D curve
//! by AdvApprox, cut preferably at its C2, then C3, breaks.
use crate::curve_approx_eval::{Curve3dEvaluator, cut_points_3d};
use crate::native_failure::failure;
use crate::native_text::stream_real;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adv_approx::{AdvApproxApproxAFunction, AdvApproxPrefAndRec};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use std::fmt;

/// The result of one approximation: the B-spline, whether it meets the tolerance,
/// whether there is one at all, and the greatest error.
///
/// `let approx = ApproxCurve::from_curve(&offset, 1e-4, Shape::C2, 16, 14)?;
/// if approx.has_result() { let bspline = approx.curve(); }`
#[derive(Clone)]
pub struct ApproxCurve {
    my_is_done: bool,
    my_has_result: bool,
    my_bspl_curve: Handle<GeomBSplineCurve>,
    my_max_error: f64,
}

impl ApproxCurve {
    /// Approximates `curve` through a GeomAdaptor_Curve within `tol3d`, with the
    /// continuity `order`, at most `max_segments` spans of degree `max_degree`
    /// (`GeomConvert_ApproxCurve(Curve, Tol3d, Order, MaxSegments, MaxDegree)`);
    /// `ApproxCurve::from_curve(&curve, 1e-4, Shape::C2, 16, 14)?`.
    pub fn from_curve(
        curve: &Handle<dyn GeomCurve>,
        tol3d: f64,
        order: Shape,
        max_segments: i32,
        max_degree: i32,
    ) -> Result<Self, Failure> {
        let adaptor: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(curve)?).cast::<dyn Adaptor3dCurve>();
        Self::from_adaptor(&adaptor, tol3d, order, max_segments, max_degree)
    }

    /// Approximates the adapted `curve` (`GeomConvert_ApproxCurve(const
    /// occ::handle<Adaptor3d_Curve>&, Tol3d, Order, MaxSegments, MaxDegree)`); an
    /// exception of the adaptor propagates;
    /// `ApproxCurve::from_adaptor(&adaptor, 1e-4, Shape::C2, 16, 14)?`.
    pub fn from_adaptor(
        curve: &Handle<dyn Adaptor3dCurve>,
        tol3d: f64,
        order: Shape,
        max_segments: i32,
        max_degree: i32,
    ) -> Result<Self, Failure> {
        let mut result = Self {
            my_is_done: false,
            my_has_result: false,
            my_bspl_curve: Handle::default(),
            my_max_error: 0.0,
        };
        result.approximate(curve, tol3d, order, [max_segments, max_degree])?;
        Ok(result)
    }

    /// The approximating B-spline, null without a result (`Curve()`); `approx.curve()`.
    pub fn curve(&self) -> Handle<GeomBSplineCurve> {
        self.my_bspl_curve.clone()
    }

    /// Whether the approximation meets the tolerance (`IsDone()`); `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// Whether there is a result, within the tolerance or not (`HasResult()`);
    /// `approx.has_result()`.
    pub fn has_result(&self) -> bool {
        self.my_has_result
    }

    /// The greatest distance between the curve and its approximation, 0 without a
    /// result (`MaxError()`); `approx.max_error()`.
    pub fn max_error(&self) -> f64 {
        self.my_max_error
    }

    /// Writes the native two-line text, the error printed as `%g` (`Dump(o)`);
    /// `approx.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        writeln!(stream, "******* Dump of ApproxCurve *******")?;
        writeln!(stream, "*******Error   {}", stream_real(self.max_error()))
    }

    // Approximate: one 3D subspace of tolerance tol3d on the curve range, cut with
    // AdvApprox_PrefAndRec at the C2 (recommended) and C3 (preferred) breaks.
    fn approximate(
        &mut self,
        curve: &Handle<dyn Adaptor3dCurve>,
        tol3d: f64,
        order: Shape,
        [max_segments, max_degree]: [i32; 2],
    ) -> Result<(), Failure> {
        let adaptor: &dyn Adaptor3dCurve = live_adaptor(curve)?;
        let mut three_d_tol: Array1<'static, f64> = one_based(1)?;
        three_d_tol.init(tol3d);
        let (first, last): (f64, f64) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let cut_tool = AdvApproxPrefAndRec::new(
            &cut_points_3d(adaptor, Shape::C2)?,
            &cut_points_3d(adaptor, Shape::C3)?,
            5.0,
        )?;
        self.my_max_error = 0.0;
        let mut evaluator =
            Curve3dEvaluator::new(curve.clone(), [first, last], "GeomConvert_ApproxCurve_Eval");
        let approx = AdvApproxApproxAFunction::with_cutting(
            0,
            0,
            1,
            None,
            None,
            Some(&three_d_tol),
            first,
            last,
            order,
            max_degree,
            max_segments,
            &mut evaluator,
            &cut_tool,
        );
        evaluator.take_failure()?;
        self.store(&approx?)
    }

    // The B-spline of the approximation and its error.
    fn store(&mut self, approx: &AdvApproxApproxAFunction) -> Result<(), Failure> {
        self.my_is_done = approx.is_done();
        self.my_has_result = approx.has_result();
        if !self.my_has_result {
            return Ok(());
        }
        let mut poles: Array1<'static, Point3> = one_based(approx.nb_poles()?)?;
        approx.poles_of(1, &mut poles)?;
        let knots: &Array1<'static, f64> = approx.knots().ok_or_else(|| missing_table("Knots"))?;
        let mults: &Array1<'static, i32> = approx
            .multiplicities()
            .ok_or_else(|| missing_table("Multiplicities"))?;
        self.my_bspl_curve = Handle::new(GeomBSplineCurve::new(
            &poles,
            knots,
            mults,
            approx.degree(),
            false,
        )?);
        self.my_max_error = approx.max_error(3, 1)?;
        Ok(())
    }
}

fn live_adaptor(curve: &Handle<dyn Adaptor3dCurve>) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert_ApproxCurve: null adaptor; expected a curve",
        )
    })
}

/// The Standard_NullObject of a missing AdvApprox result table.
pub(crate) fn missing_table(name: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("AdvApprox_ApproxAFunction::{name}: no table"),
    )
}
