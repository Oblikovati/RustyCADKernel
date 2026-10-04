//! Adaptors supply exact analytic frames to the independently ported native predicates.
use super::{BeanFaceIntersector, Failure, fast_analytic};
use fast_analytic::{AnalyticCurve, AnalyticSurface};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::{
    geom_abs::{CurveType, SurfaceType},
    gp::Axis1,
};

impl BeanFaceIntersector {
    /// Appends a coincident analytic interval or proves separation; `solver.fast_compute_analytic()?`.
    pub fn fast_compute_analytic(&mut self) -> Result<bool, Failure> {
        let curve = analytic_curve(self.curve.get().expect("bean curve"))?;
        let surface = analytic_surface(self.surface.get().expect("bean surface"))?;
        let result = fast_analytic::classify(
            curve,
            surface,
            self.range.first(),
            self.range.last(),
            self.criteria,
        );
        if let Some(range) = result.coincident_range {
            self.results.append(range);
        }
        Ok(result.successful)
    }
}

fn analytic_curve(curve: &BRepAdaptorCurve) -> Result<AnalyticCurve, Failure> {
    let kind = curve.get_type()?;
    let (axis, radius) = match kind {
        CurveType::Line => (curve.line()?.position(), 0.),
        CurveType::Circle => (curve.circle()?.axis(), curve.circle()?.radius()),
        CurveType::Ellipse => (curve.ellipse()?.axis(), 0.),
        CurveType::Hyperbola => (curve.hyperbola()?.axis(), 0.),
        CurveType::Parabola => (curve.parabola()?.axis(), 0.),
        _ => (Axis1::default(), 0.),
    };
    Ok(AnalyticCurve {
        kind: kind as i32,
        axis,
        radius,
    })
}

fn analytic_surface(surface: &BRepAdaptorSurface) -> Result<AnalyticSurface, Failure> {
    let (kind, axis, radius) = match surface.get_type()? {
        SurfaceType::Plane => (0, surface.plane()?.axis(), 0.),
        SurfaceType::Cylinder => (1, surface.cylinder()?.axis(), surface.cylinder()?.radius()),
        SurfaceType::Sphere => (
            2,
            Axis1::new(
                surface.sphere()?.location(),
                surface.sphere()?.position().direction(),
            ),
            surface.sphere()?.radius(),
        ),
        _ => (3, Axis1::default(), 0.),
    };
    Ok(AnalyticSurface { kind, axis, radius })
}
