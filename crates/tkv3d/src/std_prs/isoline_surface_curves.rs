//! Native canonical-versus-spline curve dispatch and bounded interval sampling.
use super::StdPrsIsolineSurfaceSource;
use super::isoline_surface_reference::IsolineSurfaceReference;
use super::{StdPrsDeflectionCurve, curve_limits::expand_curve_bounds};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::{
    ncollection::Sequence,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dIsoCurve},
    geom::GeomSurface,
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkmath::{
    geom_abs::{IsoType, SurfaceType},
    gp::Point3,
};
pub(super) enum IsolineSurfaceCurves {
    Canonical(Adaptor3dIsoCurve),
    Spline(Handle<dyn GeomSurface>, Box<GeomAdaptorCurve>),
}
impl IsolineSurfaceCurves {
    pub(super) fn new<
        S: StdPrsIsolineSurfaceSource + rustycad_tkernel::standard::RuntimeTyped + ?Sized,
    >(
        surface: &IsolineSurfaceReference<'_, S>,
    ) -> Result<Self, Failure> {
        match surface.snapshot().get_type()? {
            SurfaceType::BezierSurface => Ok(Self::Spline(
                surface.snapshot().bezier()?.cast(),
                Box::default(),
            )),
            SurfaceType::BSplineSurface => Ok(Self::Spline(
                surface.snapshot().bspline()?.cast(),
                Box::default(),
            )),
            _ => Ok(Self::Canonical(Adaptor3dIsoCurve::with_surface(
                &surface.snapshot().cast(),
            ))),
        }
    }
    fn load(
        &self,
        is_u: bool,
        parameter: f64,
        range: [f64; 2],
    ) -> Result<&dyn Adaptor3dCurve, Failure> {
        match self {
            Self::Canonical(curve) => load_canonical_isoline(curve, is_u, parameter, range),
            Self::Spline(surface, curve) => load_spline_isoline(surface, curve, is_u, parameter),
        }
    }
    pub(super) fn sample(
        &self,
        is_u: bool,
        parameter: f64,
        mut range: [f64; 2],
        drawer: &Prs3dDrawer,
        deflection: f64,
    ) -> Result<Sequence<Point3>, Failure> {
        let curve = self.load(is_u, parameter, range)?;
        range = clipped_isoline_curve_limits(curve, range, drawer.maximal_parameter_value())?;
        let mut points = Sequence::default();
        if range[1] - range[0] <= precision::confusion() {
            return Ok(points);
        }
        sample_deflection_isoline(curve, range, drawer, deflection, &mut points)?;
        Ok(points)
    }
}
fn clipped_isoline_curve_limits(
    curve: &dyn Adaptor3dCurve,
    range: [f64; 2],
    limit: f64,
) -> Result<[f64; 2], Failure> {
    let clipped = [
        std_max(curve.first_parameter()?, range[0]),
        std_min(curve.last_parameter()?, range[1]),
    ];
    // Share the exact geometric expansion with standard curves, after native interval intersection.
    Ok(expand_curve_bounds(curve, limit, clipped, None)?.expect("unbounded native isoline search"))
}

fn load_canonical_isoline(
    curve: &Adaptor3dIsoCurve,
    is_u: bool,
    parameter: f64,
    range: [f64; 2],
) -> Result<&dyn Adaptor3dCurve, Failure> {
    let iso = if is_u { IsoType::IsoU } else { IsoType::IsoV };
    curve.load_iso_range(iso, parameter, (range[0], range[1]))?;
    Ok(curve)
}
fn load_spline_isoline<'a>(
    surface: &Handle<dyn GeomSurface>,
    curve: &'a GeomAdaptorCurve,
    is_u: bool,
    parameter: f64,
) -> Result<&'a dyn Adaptor3dCurve, Failure> {
    let source = if is_u {
        surface.u_iso(parameter)?
    } else {
        surface.v_iso(parameter)?
    };
    curve.load(&source)?;
    Ok(curve)
}
fn sample_deflection_isoline(
    curve: &dyn Adaptor3dCurve,
    range: [f64; 2],
    drawer: &Prs3dDrawer,
    deflection: f64,
    points: &mut Sequence<Point3>,
) -> Result<(), Failure> {
    StdPrsDeflectionCurve::add_range_points(
        &Handle::default(),
        curve,
        range[0],
        range[1],
        deflection,
        points,
        drawer.deviation_angle(),
        false,
    )
}
