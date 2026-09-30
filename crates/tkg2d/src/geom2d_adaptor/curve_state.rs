// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.hxx/.cxx (curve data, load, Reset, ShallowCopy) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The locked state of `Geom2dAdaptor_Curve`: the loaded curve, its kind, its bounds
//! and the kind-specific evaluation data (`CurveDataVariant`).
use super::curve::Geom2dAdaptorCurve;
use crate::geom2d::errors::{construction, null_object};
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use crate::geom2d_eval::RepCurveDesc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

/// Offset curve data (`OffsetData`): an adaptor on the basis over the same bounds.
pub(super) struct OffsetData {
    pub(super) basis_adaptor: Handle<Geom2dAdaptorCurve>,
    pub(super) offset: f64,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
}
/// Bezier data (`BezierData`): the curve, its lazily built span cache and the
/// evaluation representation it had when it was loaded.
pub(super) struct BezierData {
    pub(super) curve: Handle<Geom2dBezierCurve>,
    pub(super) cache: Option<Cache>,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
}
/// B-spline data (`BSplineData`), as `BezierData`.
pub(super) struct BSplineData {
    pub(super) curve: Handle<Geom2dBSplineCurve>,
    pub(super) cache: Option<Cache>,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
}
/// Kind-specific evaluation data (`CurveDataVariant`); `None` is `std::monostate`.
pub(super) enum CurveData {
    None,
    Line(Line2),
    Circle(Circle2),
    Ellipse(Ellipse2),
    Hyperbola(Hyperbola2),
    Parabola(Parabola2),
    Offset(OffsetData),
    Bezier(BezierData),
    BSpline(BSplineData),
}
impl CurveData {
    // The native file-local hasEvalRep: a representation recorded at load time.
    pub(super) fn has_eval_rep(&self) -> bool {
        match self {
            CurveData::Bezier(data) => data.eval_rep.is_some(),
            CurveData::BSpline(data) => data.eval_rep.is_some(),
            CurveData::Offset(data) => data.eval_rep.is_some(),
            _ => false,
        }
    }
}

/// The fields of one `Geom2dAdaptor_Curve`.
pub(super) struct AdaptorState {
    pub(super) curve: Handle<dyn Geom2dCurve>,
    pub(super) type_curve: CurveType,
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) data: CurveData,
}
impl Default for AdaptorState {
    fn default() -> Self {
        Self {
            curve: Handle::default(),
            type_curve: CurveType::OtherCurve,
            first: 0.0,
            last: 0.0,
            data: CurveData::None,
        }
    }
}
impl AdaptorState {
    /// The loaded curve; natively a null one is dereferenced; `state.curve()?`.
    pub(super) fn curve(&self) -> Result<&dyn Geom2dCurve, Failure> {
        self.curve.get().ok_or_else(|| {
            null_object("Geom2dAdaptor_Curve: no curve is loaded; expected Load() first")
        })
    }
}

/// `Reset`: back to the default-constructed state; `reset(&mut state)`.
pub(super) fn reset(state: &mut AdaptorState) {
    *state = AdaptorState::default();
}

/// The public `Load(C)`: the curve's own bounds; `load_whole(&mut state, &curve)?`.
pub(super) fn load_whole(
    state: &mut AdaptorState,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    let Some(source) = curve.get() else {
        return Err(null_object(
            "Geom2dAdaptor_Curve::Load: null curve; expected a curve",
        ));
    };
    load(
        state,
        curve,
        source.first_parameter(),
        source.last_parameter(),
    )
}

/// The public `Load(C, UFirst, ULast)`: Standard_NullObject for a null curve and
/// Standard_ConstructionError when `u_first > u_last + Precision::Confusion()`;
/// `load_range(&mut state, &curve, 0.0, 1.0)?`.
pub(super) fn load_range(
    state: &mut AdaptorState,
    curve: &Handle<dyn Geom2dCurve>,
    u_first: f64,
    u_last: f64,
) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(null_object(
            "Geom2dAdaptor_Curve::Load: null curve; expected a curve",
        ));
    }
    if u_first > u_last + precision::confusion() {
        return Err(construction(&format!(
            "Geom2dAdaptor_Curve::Load: UFirst = {u_first} > ULast = {u_last}; \
             expected UFirst <= ULast + Precision::Confusion()"
        )));
    }
    load(state, curve, u_first, u_last)
}

// The private `load`: a new curve resets the data; the same curve only drops the caches.
fn load(
    state: &mut AdaptorState,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    state.first = first;
    state.last = last;
    if state.curve == *curve {
        drop_caches(&mut state.data);
        return Ok(());
    }
    state.curve = curve.clone();
    state.data = CurveData::None;
    classify(state, curve, first, last)
}

fn drop_caches(data: &mut CurveData) {
    match data {
        CurveData::BSpline(bspline) => bspline.cache = None,
        CurveData::Bezier(bezier) => bezier.cache = None,
        _ => {}
    }
}

fn is_class<T: RuntimeTyped>(curve: &Handle<dyn Geom2dCurve>) -> bool {
    curve.is_instance(T::static_type())
}

// Kind and data from the exact dynamic type, as the native `DynamicType() ==` chain.
fn classify(
    state: &mut AdaptorState,
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    if is_class::<Geom2dTrimmedCurve>(curve) {
        let basis: Handle<dyn Geom2dCurve> = curve.cast::<Geom2dTrimmedCurve>().basis_curve();
        return load_range(state, &basis, first, last);
    }
    let (kind, data): (CurveType, CurveData) = if let Some(elementary) = elementary_data(curve)? {
        elementary
    } else if is_class::<Geom2dBezierCurve>(curve) {
        (CurveType::BezierCurve, bezier_data(curve))
    } else if is_class::<Geom2dBSplineCurve>(curve) {
        (CurveType::BSplineCurve, bspline_data(curve))
    } else if is_class::<Geom2dOffsetCurve>(curve) {
        (CurveType::OffsetCurve, offset_data(curve, first, last)?)
    } else {
        (CurveType::OtherCurve, CurveData::None)
    };
    state.type_curve = kind;
    state.data = data;
    Ok(())
}

fn elementary_data(
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Option<(CurveType, CurveData)>, Failure> {
    Ok(Some(if is_class::<Geom2dCircle>(curve) {
        (
            CurveType::Circle,
            CurveData::Circle(curve.cast::<Geom2dCircle>().circ2d()?),
        )
    } else if is_class::<Geom2dLine>(curve) {
        (
            CurveType::Line,
            CurveData::Line(curve.cast::<Geom2dLine>().lin2d()),
        )
    } else if is_class::<Geom2dEllipse>(curve) {
        (
            CurveType::Ellipse,
            CurveData::Ellipse(curve.cast::<Geom2dEllipse>().elips2d()?),
        )
    } else if is_class::<Geom2dParabola>(curve) {
        (
            CurveType::Parabola,
            CurveData::Parabola(curve.cast::<Geom2dParabola>().parab2d()?),
        )
    } else if is_class::<Geom2dHyperbola>(curve) {
        (
            CurveType::Hyperbola,
            CurveData::Hyperbola(curve.cast::<Geom2dHyperbola>().hypr2d()?),
        )
    } else {
        return Ok(None);
    }))
}

fn bezier_data(curve: &Handle<dyn Geom2dCurve>) -> CurveData {
    let bezier: Handle<Geom2dBezierCurve> = curve.cast();
    let eval_rep = bezier.eval_representation();
    CurveData::Bezier(BezierData {
        curve: bezier,
        cache: None,
        eval_rep,
    })
}

fn bspline_data(curve: &Handle<dyn Geom2dCurve>) -> CurveData {
    let bspline: Handle<Geom2dBSplineCurve> = curve.cast();
    let eval_rep = bspline.eval_representation();
    CurveData::BSpline(BSplineData {
        curve: bspline,
        cache: None,
        eval_rep,
    })
}

// A nested adaptor over the basis with the same bounds.
fn offset_data(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<CurveData, Failure> {
    let offset_curve: Handle<Geom2dOffsetCurve> = curve.cast();
    let basis: Handle<dyn Geom2dCurve> = offset_curve.basis_curve();
    let basis_adaptor = Handle::new(Geom2dAdaptorCurve::with_range(&basis, first, last)?);
    Ok(CurveData::Offset(OffsetData {
        basis_adaptor,
        offset: offset_curve.offset(),
        eval_rep: offset_curve.eval_representation(),
    }))
}

/// `ShallowCopy` of the state: the curve is shared, a nested offset adaptor is itself
/// shallow-copied, and the caches are left to be rebuilt; `shallow_state(&state)?`.
pub(super) fn shallow_state(state: &AdaptorState) -> Result<AdaptorState, Failure> {
    let data: CurveData = match &state.data {
        CurveData::Offset(offset) => CurveData::Offset(OffsetData {
            basis_adaptor: match offset.basis_adaptor.get() {
                Some(basis) => Handle::new(basis.shallow_adaptor()?),
                None => Handle::default(),
            },
            offset: offset.offset,
            eval_rep: offset.eval_rep.clone(),
        }),
        CurveData::BSpline(bspline) => CurveData::BSpline(BSplineData {
            curve: bspline.curve.clone(),
            cache: None,
            eval_rep: bspline.eval_rep.clone(),
        }),
        CurveData::Bezier(bezier) => CurveData::Bezier(BezierData {
            curve: bezier.curve.clone(),
            cache: None,
            eval_rep: bezier.eval_rep.clone(),
        }),
        CurveData::Line(line) => CurveData::Line(*line),
        CurveData::Circle(circle) => CurveData::Circle(*circle),
        CurveData::Ellipse(ellipse) => CurveData::Ellipse(*ellipse),
        CurveData::Hyperbola(hyperbola) => CurveData::Hyperbola(*hyperbola),
        CurveData::Parabola(parabola) => CurveData::Parabola(*parabola),
        CurveData::None => CurveData::None,
    };
    Ok(AdaptorState {
        curve: state.curve.clone(),
        type_curve: state.type_curve,
        first: state.first,
        last: state.last,
        data,
    })
}
