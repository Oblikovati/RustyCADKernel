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
mod bezier_surface;
mod bspline_curve;
mod bspline_curve2d;
mod bspline_surface;
mod commands;
mod curve;
mod curve2d;
mod curve_common;
mod drawable;
mod geometry;
mod params;
mod point;
mod polygon;
mod spline_common;
mod style;
mod surface;
mod triangulation;
mod variables;

pub use bezier_curve::DrawTrSurfBezierCurve;
pub use bezier_curve2d::DrawTrSurfBezierCurve2d;
pub use bezier_surface::DrawTrSurfBezierSurface;
pub use bspline_curve::DrawTrSurfBSplineCurve;
pub use bspline_curve2d::DrawTrSurfBSplineCurve2d;
pub use bspline_surface::DrawTrSurfBSplineSurface;
pub use commands::basic_commands;
pub use curve::DrawTrSurfCurve;
pub(crate) use curve_common::draw_arrow;
pub use curve2d::DrawTrSurfCurve2d;
pub use drawable::DrawTrSurfDrawable;
pub(crate) use drawable::{bound, interval_bounds};
pub use params::{DrawTrSurfParams, parameters};
pub use point::DrawTrSurfPoint;
pub use polygon::{DrawTrSurfPolygon2D, DrawTrSurfPolygon3D};
pub use spline_common::{KnotsStyle, PolesStyle, SplineLook};
pub use style::{CurveLook, CurveStyle};
pub use surface::{DrawTrSurfSurface, SurfaceLook};
pub use triangulation::{DrawTrSurfTriangulation, TriangulationEdges};

#[cfg(test)]
mod tests;
