use super::*;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::make_curve;
use rustycad_tkg3d::geom::GeomElementarySurface;
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::{geom_proj_lib, proj_lib::ProjectedCurve};
use rustycad_tkmath::geom_abs::CurveType;

impl ShapeConstructProjectCurveOnSurface {
    pub(super) fn project_analytic(
        &self,
        curve: &Handle<dyn GeomCurve>,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let analysis = self.analysis();
        let plane = projection_plane(&analysis.surface());
        if plane.is_null() {
            return Ok(Handle::default());
        }
        let projected =
            geom_proj_lib::project_on_plane(curve, &plane, plane.position().direction(), true)?;
        let adapted = Handle::new(GeomAdaptorCurve::with_curve(&projected)?).cast();
        let projection = ProjectedCurve::with_curve(&analysis.adaptor3d().cast(), &adapted)?;
        let result = make_curve(&projection)?;
        if let Some(trimmed) = result.cast::<Geom2dTrimmedCurve>().get() {
            return Ok(trimmed.basis_curve());
        }
        Ok(result)
    }

    /// Projects through ProjLib; `projector.perform_by_proj_lib(&curve, (first,last), &mut pcurve)`.
    pub fn perform_by_proj_lib(
        &self,
        curve: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        output: &mut Handle<dyn Geom2dCurve>,
    ) -> bool {
        *output = Handle::default();
        if self.analysis().is_null() {
            self.set_status(ShapeExtendStatus::Fail1);
            return false;
        }
        let Ok(projected) = self.proj_lib_curve(curve, range) else {
            self.set_status(ShapeExtendStatus::Fail3);
            return false;
        };
        *output = projected;
        let status = if output.is_null() {
            ShapeExtendStatus::Fail2
        } else {
            ShapeExtendStatus::Done1
        };
        self.set_status(status);
        !output.is_null()
    }

    fn proj_lib_curve(
        &self,
        curve: &Handle<dyn GeomCurve>,
        range: (f64, f64),
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let adapted = Handle::new(GeomAdaptorCurve::with_range(curve, range.0, range.1)?).cast();
        let projection = ProjectedCurve::with_curve(&self.analysis().adaptor3d().cast(), &adapted)?;
        untrimmed_projected_curve(&projection)
    }
}

fn projection_plane(surface: &Handle<dyn GeomSurface>) -> Handle<GeomPlane> {
    let plane = surface.cast::<GeomPlane>();
    if !plane.is_null() {
        return plane;
    }
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        return trimmed.basis_surface().cast();
    }
    if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
        return offset.basis_surface().cast();
    }
    Handle::default()
}

fn untrimmed_projected_curve(
    projection: &ProjectedCurve,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(match projection.get_type()? {
        CurveType::Line => Handle::new(Geom2dLine::from_lin2d(projection.line()?)).cast(),
        CurveType::Circle => Handle::new(Geom2dCircle::from_circ2d(projection.circle()?)).cast(),
        CurveType::Ellipse => {
            Handle::new(Geom2dEllipse::from_elips2d(projection.ellipse()?)).cast()
        }
        CurveType::Parabola => {
            Handle::new(Geom2dParabola::from_parab2d(projection.parabola()?)).cast()
        }
        CurveType::Hyperbola => {
            Handle::new(Geom2dHyperbola::from_hypr2d(projection.hyperbola()?)).cast()
        }
        CurveType::BSplineCurve => projection.bspline()?.cast(),
        _ => Handle::default(),
    })
}
