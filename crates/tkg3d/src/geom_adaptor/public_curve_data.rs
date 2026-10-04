//! Public aggregate types declared by `GeomAdaptor_Curve` at OCCT 3d097a0.
// These aggregates can be constructed independently of an adaptor. Their cache handles
// share identity even though the adaptor's private evaluation state uses owned caches.
use super::GeomAdaptorCurve;
use crate::geom::{GeomBSplineCurve, GeomBezierCurve};
use crate::geom_eval::RepCurveDesc;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    bsplclib::SharedBSplineCache,
    gp::{Circle3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3},
};

/// Offset aggregate with native null handles and X direction; `CurveOffsetData::default()`.
#[derive(Clone, Default)]
pub struct CurveOffsetData {
    /// Shared basis adaptor.
    pub basis_adaptor: Handle<GeomAdaptorCurve>,
    /// Signed offset distance, initially zero.
    pub offset: f64,
    /// Reference direction used by offset evaluation.
    pub direction: Direction3,
    /// Optional shared evaluation descriptor.
    pub eval_rep: Handle<dyn RepCurveDesc>,
}

/// Bezier aggregate retaining curve and cache identity; `CurveBezierData::default()`.
#[derive(Clone, Default)]
pub struct CurveBezierData {
    /// Shared Bezier curve.
    pub curve: Handle<GeomBezierCurve>,
    /// Nullable shared mutable evaluation cache.
    pub cache: SharedBSplineCache,
    /// Optional shared evaluation descriptor.
    pub eval_rep: Handle<dyn RepCurveDesc>,
}

/// B-spline aggregate retaining curve and cache identity; `CurveBSplineData::default()`.
#[derive(Clone, Default)]
pub struct CurveBSplineData {
    /// Shared B-spline curve.
    pub curve: Handle<GeomBSplineCurve>,
    /// Nullable shared mutable evaluation cache.
    pub cache: SharedBSplineCache,
    /// Optional shared evaluation descriptor.
    pub eval_rep: Handle<dyn RepCurveDesc>,
}

/// Native curve variant alternatives in declaration order; `CurveDataVariant::default()`.
#[derive(Clone, Default)]
pub enum CurveDataVariant {
    /// Native `std::monostate` default.
    #[default]
    Uninitialized,
    /// Elementary line, copied by value.
    Line(Line3),
    /// Elementary circle, copied by value.
    Circle(Circle3),
    /// Elementary ellipse, copied by value.
    Ellipse(Ellipse3),
    /// Elementary hyperbola, copied by value.
    Hyperbola(Hyperbola3),
    /// Elementary parabola, copied by value.
    Parabola(Parabola3),
    /// Offset aggregate retaining shared handles.
    Offset(CurveOffsetData),
    /// Bezier aggregate retaining shared handles.
    Bezier(CurveBezierData),
    /// B-spline aggregate retaining shared handles.
    BSpline(CurveBSplineData),
}
