//! Public value records nested in the `Geom2dAdaptor_Curve` declaration.
use crate::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use crate::geom2d_eval::RepCurveDesc;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2};

use super::Geom2dAdaptorCurve;
use rustycad_tkmath::bsplclib::SharedBSplineCache;

/// Offset-curve evaluation state (`Geom2dAdaptor_Curve::OffsetData`).
#[derive(Clone, Default)]
pub struct OffsetData {
    /// Adaptor holding the basis curve; clones preserve handle identity.
    pub basis_adaptor: Handle<Geom2dAdaptorCurve>,
    /// Signed offset distance.
    pub offset: f64,
    /// Optional evaluation representation captured from the loaded curve.
    pub eval_rep: Handle<dyn RepCurveDesc>,
}

/// Bezier evaluation state, including an optionally shared mutable cache.
#[derive(Clone, Default)]
pub struct BezierData {
    /// Bezier curve retained without downcasting again.
    pub curve: Handle<Geom2dBezierCurve>,
    /// Nullable shared cache; cloned payloads preserve identity and mutation.
    pub cache: SharedBSplineCache,
    /// Optional evaluation representation captured at load time.
    pub eval_rep: Handle<dyn RepCurveDesc>,
}

/// B-spline evaluation state, including an optionally shared mutable cache.
#[derive(Clone, Default)]
pub struct BSplineData {
    /// B-spline curve retained without downcasting again.
    pub curve: Handle<Geom2dBSplineCurve>,
    /// Nullable shared cache; cloned payloads preserve identity and mutation.
    pub cache: SharedBSplineCache,
    /// Optional evaluation representation captured at load time.
    pub eval_rep: Handle<dyn RepCurveDesc>,
}

/// One alternative from `Geom2dAdaptor_Curve::CurveDataVariant`.
#[derive(Clone, Default)]
pub enum CurveDataVariant {
    /// Native `std::monostate` state.
    #[default]
    Uninitialized,
    /// Elementary line parameters.
    Line(Line2),
    /// Elementary circle parameters.
    Circle(Circle2),
    /// Elementary ellipse parameters.
    Ellipse(Ellipse2),
    /// Elementary hyperbola parameters.
    Hyperbola(Hyperbola2),
    /// Elementary parabola parameters.
    Parabola(Parabola2),
    /// Offset curve state.
    Offset(OffsetData),
    /// Bezier curve state.
    Bezier(BezierData),
    /// B-spline curve state.
    BSpline(BSplineData),
}
