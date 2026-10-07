//! The `DrawTrSurf` package of TKDraw: drawable curves, surfaces, points, polygons and
//! triangulations, the `DrawTrSurf::Set`/`Get` functions and the geometric display and
//! transformation commands.
//!
//! The C++ class hierarchy (`DrawTrSurf_BSplineCurve` derives from `DrawTrSurf_Curve`,
//! which derives from `DrawTrSurf_Drawable`) is composition here: each drawable embeds
//! its base part and answers [`crate::draw::Drawable3D::ancestor`] for it, so
//! `downcast_kind::<DrawTrSurfCurve>()` finds the curve part of a BSpline curve as
//! `occ::down_cast<DrawTrSurf_Curve>` does.
mod bezier_curve;
mod bezier_curve2d;
mod bspline_curve;
mod bspline_curve2d;
mod curve;
mod curve2d;
mod curve_common;
mod drawable;
mod geometry;
mod params;
mod spline_common;
mod style;

pub use bezier_curve::DrawTrSurfBezierCurve;
pub use bezier_curve2d::DrawTrSurfBezierCurve2d;
pub use bspline_curve::DrawTrSurfBSplineCurve;
pub use bspline_curve2d::DrawTrSurfBSplineCurve2d;
pub use curve::DrawTrSurfCurve;
pub use curve2d::DrawTrSurfCurve2d;
pub use drawable::DrawTrSurfDrawable;
pub use params::{DrawTrSurfParams, parameters};
pub use spline_common::{KnotsStyle, PolesStyle, SplineLook};
pub use style::{CurveLook, CurveStyle};
