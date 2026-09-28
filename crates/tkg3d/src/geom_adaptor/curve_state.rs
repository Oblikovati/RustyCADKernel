// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.hxx/.cxx (curve data, load, Reset, ShallowCopy) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The locked state of `GeomAdaptor_Curve`: the loaded curve, its kind, its bounds
//! and the kind-specific evaluation data (`CurveDataVariant`).
use super::curve::GeomAdaptorCurve;
use crate::geom::errors::{construction, null_object};
use crate::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use crate::geom_eval::RepCurveDesc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::bsplclib::Cache;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3};

/// Offset curve data (`OffsetData`): an adaptor on the whole basis curve, the offset
/// and its reference direction.
pub(super) struct OffsetData {
    pub(super) basis_adaptor: Handle<GeomAdaptorCurve>,
    pub(super) offset: f64,
    pub(super) direction: Direction3,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
}
/// Bezier data (`BezierData`): the curve, its lazily built span cache and the
/// evaluation representation it had when it was loaded.
pub(super) struct BezierData {
    pub(super) curve: Handle<GeomBezierCurve>,
    pub(super) cache: Option<Cache>,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
}
/// B-spline data (`BSplineData`), as `BezierData`.
pub(super) struct BSplineData {
    pub(super) curve: Handle<GeomBSplineCurve>,
    pub(super) cache: Option<Cache>,
    pub(super) eval_rep: Handle<dyn RepCurveDesc>,
}
/// Kind-specific evaluation data (`CurveDataVariant`); `None` is `std::monostate`.
pub(super) enum CurveData {
    None,
    Line(Line3),
    Circle(Circle3),
    Ellipse(Ellipse3),
    Hyperbola(Hyperbola3),
    Parabola(Parabola3),
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

/// The fields of one `GeomAdaptor_Curve`.
pub(super) struct AdaptorState {
    pub(super) curve: Handle<dyn GeomCurve>,
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
    pub(super) fn curve(&self) -> Result<&dyn GeomCurve, Failure> {
        self.curve.get().ok_or_else(|| {
            null_object("GeomAdaptor_Curve: no curve is loaded; expected Load() first")
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
    curve: &Handle<dyn GeomCurve>,
) -> Result<(), Failure> {
    let Some(source) = curve.get() else {
        return Err(null_object(
            "GeomAdaptor_Curve::Load: null curve; expected a curve",
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
    curve: &Handle<dyn GeomCurve>,
    u_first: f64,
    u_last: f64,
) -> Result<(), Failure> {
    if curve.is_null() {
        return Err(null_object(
            "GeomAdaptor_Curve::Load: null curve; expected a curve",
        ));
    }
    if u_first > u_last + precision::confusion() {
        return Err(construction(&format!(
            "GeomAdaptor_Curve::Load: UFirst = {u_first} > ULast = {u_last}; \
             expected UFirst <= ULast + Precision::Confusion()"
        )));
    }
    load(state, curve, u_first, u_last)
}

// The private `load`: a new curve resets the data; the same curve only drops the caches.
fn load(
    state: &mut AdaptorState,
    curve: &Handle<dyn GeomCurve>,
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

fn is_class<T: RuntimeTyped>(curve: &Handle<dyn GeomCurve>) -> bool {
    curve.is_instance(T::static_type())
}

// Kind and data from the exact dynamic type, as the native `DynamicType() ==` chain.
// A trimmed curve reloads its basis through the public Load, whose range check
// applies again; an unknown class keeps the previous kind as `OtherCurve`.
fn classify(
    state: &mut AdaptorState,
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    if is_class::<GeomTrimmedCurve>(curve) {
        let basis: Handle<dyn GeomCurve> = curve.cast::<GeomTrimmedCurve>().basis_curve();
        return load_range(state, &basis, first, last);
    }
    let (kind, data): (CurveType, CurveData) = if let Some(elementary) = elementary_data(curve)? {
        elementary
    } else if is_class::<GeomBezierCurve>(curve) {
        (CurveType::BezierCurve, bezier_data(curve))
    } else if is_class::<GeomBSplineCurve>(curve) {
        (CurveType::BSplineCurve, bspline_data(curve))
    } else if is_class::<GeomOffsetCurve>(curve) {
        (CurveType::OffsetCurve, offset_data(curve)?)
    } else {
        (CurveType::OtherCurve, CurveData::None)
    };
    state.type_curve = kind;
    state.data = data;
    Ok(())
}

fn elementary_data(
    curve: &Handle<dyn GeomCurve>,
) -> Result<Option<(CurveType, CurveData)>, Failure> {
    Ok(Some(if is_class::<GeomCircle>(curve) {
        (
            CurveType::Circle,
            CurveData::Circle(curve.cast::<GeomCircle>().circ()?),
        )
    } else if is_class::<GeomLine>(curve) {
        (
            CurveType::Line,
            CurveData::Line(curve.cast::<GeomLine>().lin()),
        )
    } else if is_class::<GeomEllipse>(curve) {
        (
            CurveType::Ellipse,
            CurveData::Ellipse(curve.cast::<GeomEllipse>().elips()?),
        )
    } else if is_class::<GeomParabola>(curve) {
        (
            CurveType::Parabola,
            CurveData::Parabola(curve.cast::<GeomParabola>().parab()?),
        )
    } else if is_class::<GeomHyperbola>(curve) {
        (
            CurveType::Hyperbola,
            CurveData::Hyperbola(curve.cast::<GeomHyperbola>().hypr()?),
        )
    } else {
        return Ok(None);
    }))
}

fn bezier_data(curve: &Handle<dyn GeomCurve>) -> CurveData {
    let bezier: Handle<GeomBezierCurve> = curve.cast();
    let eval_rep = bezier.eval_representation();
    CurveData::Bezier(BezierData {
        curve: bezier,
        cache: None,
        eval_rep,
    })
}

fn bspline_data(curve: &Handle<dyn GeomCurve>) -> CurveData {
    let bspline: Handle<GeomBSplineCurve> = curve.cast();
    let eval_rep = bspline.eval_representation();
    CurveData::BSpline(BSplineData {
        curve: bspline,
        cache: None,
        eval_rep,
    })
}

// A nested adaptor over the whole basis curve (unlike Geom2dAdaptor, which uses the
// adaptor range).
fn offset_data(curve: &Handle<dyn GeomCurve>) -> Result<CurveData, Failure> {
    let offset_curve: Handle<GeomOffsetCurve> = curve.cast();
    let basis: Handle<dyn GeomCurve> = offset_curve.basis_curve();
    let basis_adaptor = Handle::new(GeomAdaptorCurve::with_curve(&basis)?);
    Ok(CurveData::Offset(OffsetData {
        basis_adaptor,
        offset: offset_curve.offset(),
        direction: offset_curve.direction(),
        eval_rep: offset_curve.eval_representation(),
    }))
}

/// `ShallowCopy` of the state: the curve is shared, a nested offset adaptor is itself
/// shallow-copied, and the caches are left to be rebuilt; `shallow_state(&state)?`.
pub(super) fn shallow_state(state: &AdaptorState) -> Result<AdaptorState, Failure> {
    Ok(AdaptorState {
        curve: state.curve.clone(),
        type_curve: state.type_curve,
        first: state.first,
        last: state.last,
        data: shallow_data(&state.data)?,
    })
}

fn shallow_data(data: &CurveData) -> Result<CurveData, Failure> {
    Ok(match data {
        CurveData::Offset(offset) => CurveData::Offset(OffsetData {
            basis_adaptor: Handle::new(offset.basis_adaptor.shallow_adaptor()?),
            offset: offset.offset,
            direction: offset.direction,
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
    })
}
