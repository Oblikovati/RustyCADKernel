// Rust translation of ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Curve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_Curve`: the evaluator of a 3D curve's kind, chosen once at construction.
use super::adaptor_curve::CurveRef3d;
use super::{
    BSplineCurve, BezierCurve, Circle, Domain1D, Ellipse, ExtremaResult, Hyperbola, Line,
    OffsetCurve, OtherCurve, Parabola, SearchMode,
};
use crate::extrema_pc_core::PcCurveRef;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorTransformedCurve};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point3, Transform3, TransformForm};

mod dispatch;
mod transform;

/// The specialized evaluator of [`Curve`] (`EvaluatorVariant`); `Uninitialized` is the
/// native `std::monostate`.
pub enum EvaluatorVariant<'a> {
    /// No evaluator (`std::monostate`).
    Uninitialized,
    /// `ExtremaPC_Line`.
    Line(Line),
    /// `ExtremaPC_Circle`.
    Circle(Circle),
    /// `ExtremaPC_Ellipse`.
    Ellipse(Ellipse),
    /// `ExtremaPC_Hyperbola`.
    Hyperbola(Hyperbola),
    /// `ExtremaPC_Parabola`.
    Parabola(Parabola),
    /// `ExtremaPC_BezierCurve`.
    Bezier(BezierCurve),
    /// `ExtremaPC_BSplineCurve`.
    BSpline(BSplineCurve),
    /// `ExtremaPC_OffsetCurve`.
    Offset(OffsetCurve<'a>),
    /// `ExtremaPC_OtherCurve`.
    Other(OtherCurve<'a>),
}

/// Point-curve extrema on any 3D curve (`ExtremaPC_Curve`).
///
/// The domain is fixed at construction. A GeomAdaptor_Curve dispatched as an offset or
/// general curve is borrowed and must outlive the evaluator; a Geom curve gets its own
/// adaptor. A transformed 3D curve is solved in the original space (the query is
/// inverse-transformed, the tolerance divided by the scale) and the points mapped back;
/// `Curve::from_geom(&curve)?.perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct Curve<'a> {
    evaluator: EvaluatorVariant<'a>,
    trsf: Option<Transform3>,
    not_done: ExtremaResult,
    transform_result: ExtremaResult,
}

impl<'a> Curve<'a> {
    fn with_evaluator(evaluator: EvaluatorVariant<'a>) -> Self {
        Self {
            evaluator,
            trsf: None,
            not_done: ExtremaResult::not_done(),
            transform_result: ExtremaResult::default(),
        }
    }
    /// Evaluator over the adaptor bounds (`ExtremaPC_Curve(const GeomAdaptor_Curve&)`);
    /// uninitialized without a loaded curve; `Curve::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(curve: &'a GeomAdaptorCurve) -> Result<Self, Failure> {
        if curve.curve().is_null() {
            return Ok(Self::with_evaluator(EvaluatorVariant::Uninitialized));
        }
        let domain = Domain1D::new(curve.first_parameter()?, curve.last_parameter()?);
        Ok(Self::with_evaluator(dispatch::from_adaptor(
            PcCurveRef::Borrowed(curve as &dyn Adaptor3dCurve),
            domain,
        )?))
    }
    /// Evaluator over `[u_min, u_max]` (`ExtremaPC_Curve(theCurve, theUMin, theUMax)`);
    /// `Curve::from_adaptor_range(&adaptor, 0.0, 1.0)?`.
    pub fn from_adaptor_range(
        curve: &'a GeomAdaptorCurve,
        u_min: f64,
        u_max: f64,
    ) -> Result<Self, Failure> {
        if curve.curve().is_null() {
            return Ok(Self::with_evaluator(EvaluatorVariant::Uninitialized));
        }
        let domain = Domain1D::new(u_min, u_max);
        Ok(Self::with_evaluator(dispatch::from_adaptor(
            PcCurveRef::Borrowed(curve as &dyn Adaptor3dCurve),
            domain,
        )?))
    }
    /// Evaluator of a transformed curve over its bounds
    /// (`ExtremaPC_Curve(const GeomAdaptor_TransformedCurve&)`); `Curve::from_transformed(&adaptor)?`.
    pub fn from_transformed(curve: &'a GeomAdaptorTransformedCurve) -> Result<Self, Failure> {
        Self::transformed(curve, None)
    }
    /// Evaluator of a transformed curve over `[u_min, u_max]`
    /// (`ExtremaPC_Curve(theCurve, theUMin, theUMax)`); `Curve::from_transformed_range(&a, 0.0, 1.0)?`.
    pub fn from_transformed_range(
        curve: &'a GeomAdaptorTransformedCurve,
        u_min: f64,
        u_max: f64,
    ) -> Result<Self, Failure> {
        Self::transformed(curve, Some(Domain1D::new(u_min, u_max)))
    }
    // The null check of both transformed-curve constructors, then initFromTransformedCurve.
    fn transformed(
        curve: &'a GeomAdaptorTransformedCurve,
        domain: Option<Domain1D>,
    ) -> Result<Self, Failure> {
        if curve.is_3d_curve() && curve.geom_curve().is_null() {
            return Ok(Self::with_evaluator(EvaluatorVariant::Uninitialized));
        }
        let (evaluator, trsf) = transform::from_transformed(curve, domain)?;
        Ok(Self {
            trsf,
            ..Self::with_evaluator(evaluator)
        })
    }
    /// Evaluator of a Geom curve (`ExtremaPC_Curve(const occ::handle<Geom_Curve>&)`): the
    /// natural domain, the trimmed bounds of a trimmed curve; a null curve stays
    /// uninitialized; `Curve::from_geom(&curve)?`.
    pub fn from_geom(curve: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        if curve.is_null() {
            return Ok(Self::with_evaluator(EvaluatorVariant::Uninitialized));
        }
        if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
            let domain = Domain1D::new(trimmed.first_parameter(), trimmed.last_parameter());
            let evaluator = dispatch::from_geom(&trimmed.basis_curve(), Some(domain))?;
            return Ok(Self::with_evaluator(evaluator));
        }
        Ok(Self::with_evaluator(dispatch::from_geom(curve, None)?))
    }
    /// Evaluator over `[u_min, u_max]`, intersected with the bounds of a trimmed curve
    /// (`ExtremaPC_Curve(theCurve, theUMin, theUMax)`); `Curve::from_geom_range(&curve, 0.0, 1.0)?`.
    pub fn from_geom_range(
        curve: &Handle<dyn GeomCurve>,
        u_min: f64,
        u_max: f64,
    ) -> Result<Self, Failure> {
        if curve.is_null() {
            return Ok(Self::with_evaluator(EvaluatorVariant::Uninitialized));
        }
        let (mut base, mut first, mut last) = (curve.clone(), u_min, u_max);
        if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
            base = trimmed.basis_curve();
            // Intersect bounds: take max of mins, min of maxs
            first = rustycad_tkg2d::geom2d::native::std_max(u_min, trimmed.first_parameter());
            last = rustycad_tkg2d::geom2d::native::std_min(u_max, trimmed.last_parameter());
        }
        let evaluator = dispatch::from_geom(&base, Some(Domain1D::new(first, last)))?;
        Ok(Self::with_evaluator(evaluator))
    }
    /// True unless the evaluator is uninitialized (`IsInitialized`); `curve.is_initialized()`.
    pub fn is_initialized(&self) -> bool {
        !matches!(self.evaluator, EvaluatorVariant::Uninitialized)
    }
    /// The specialized evaluator; `curve.evaluator()`.
    pub fn evaluator(&self) -> &EvaluatorVariant<'a> {
        &self.evaluator
    }
    /// The interior extrema from the specialized evaluator (`Perform`);
    /// `curve.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.run(point, tol, mode, false)
    }
    /// The interior extrema and the extremal ends (`PerformWithEndpoints`);
    /// `curve.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.run(point, tol, mode, true)
    }
    // Both members: directly, or in the original space of a transformed curve.
    fn run(
        &mut self,
        point: Point3,
        tol: f64,
        mode: SearchMode,
        ends: bool,
    ) -> Result<&ExtremaResult, Failure> {
        let Some(trsf) = self.trsf else {
            return dispatch::run(
                &mut self.evaluator,
                &self.not_done,
                (point, tol, mode, ends),
            );
        };
        transform::run(
            &mut self.evaluator,
            &self.not_done,
            &mut self.transform_result,
            trsf,
            (point, tol, mode, ends),
        )
    }
}

/// `theCurve.Trsf().Form() != gp_Identity`.
pub(super) fn non_identity(trsf: Transform3) -> Option<Transform3> {
    (trsf.form() != TransformForm::Identity).then_some(trsf)
}

/// A shared handle on an owned GeomAdaptor_Curve.
pub(super) fn shared(adaptor: Handle<GeomAdaptorCurve>) -> CurveRef3d<'static> {
    PcCurveRef::Shared(adaptor.cast())
}

/// The curve kind of an adaptor reference.
pub(super) fn kind(curve: &CurveRef3d<'_>) -> Result<CurveType, Failure> {
    let adaptor = curve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ExtremaPC_Curve: null adaptor; expected a curve",
            "",
        )
    })?;
    adaptor.get_type()
}
