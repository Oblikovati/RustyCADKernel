use rustycad_tkernel::precision;
// AdjustPCurveOnSurf from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorSurface,
    topexp::Explorer,
    topods::{self, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    top_abs::{ShapeEnum, State},
};
use rustycad_tkgeomalgo::geom_int;
use rustycad_tkmath::{
    geom_abs::SurfaceType,
    gp::{Point2, Vector2},
};
use rustycad_tktopalgo::brep_class::BRepClassFaceClassifier;
struct PeriodicPosition {
    u: f64,
    v: f64,
    du: f64,
    dv: f64,
    umin: f64,
    umax: f64,
    vmin: f64,
    vmax: f64,
}
/// Translate a pcurve by native periodic offsets, preserving its handle when unchanged; `adjust_pcurve_on_surface(&surface, 0., 1., &curve)?`.
pub fn adjust_pcurve_on_surface(
    surface: &BRepAdaptorSurface,
    first: f64,
    last: f64,
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut position = PeriodicPosition::load(surface, curve.value(0.5 * (first + last))?)?;
    position.adjust_u(surface)?;
    position.adjust_v(surface)?;
    position.classify_wide_domain(surface)?;
    if position.du == 0. && position.dv == 0. {
        return Ok(curve.clone());
    }
    Ok(curve
        .translated(Vector2::new(position.du, position.dv))?
        .into_cast())
}
/// Use the face's cached or fresh restricted adaptor; `adjust_pcurve_on_face(&face, 0., 1., &curve, None)?`.
pub fn adjust_pcurve_on_face(
    face: &Face,
    first: f64,
    last: f64,
    curve: &Handle<dyn Geom2dCurve>,
    context: Option<&Context>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if let Some(context) = context {
        let cached = context.surface_adaptor(face)?;
        return adjust_pcurve_on_surface(&cached.read(), first, last, curve);
    }
    adjust_pcurve_on_surface(
        &BRepAdaptorSurface::with_face(face, true)?,
        first,
        last,
        curve,
    )
}
impl PeriodicPosition {
    fn load(surface: &BRepAdaptorSurface, point: Point2) -> Result<Self, Failure> {
        Ok(Self {
            u: point.x(),
            v: point.y(),
            du: 0.,
            dv: 0.,
            umin: surface.first_u_parameter()?,
            umax: surface.last_u_parameter()?,
            vmin: surface.first_v_parameter()?,
            vmax: surface.last_v_parameter()?,
        })
    }
    fn adjust_u(&mut self, surface: &BRepAdaptorSurface) -> Result<(), Failure> {
        if !surface.is_u_periodic()? {
            return Ok(());
        }
        let period = surface.u_period()?;
        let delta = precision::p_confusion();
        if (self.u - self.umin).abs() < delta {
            self.u = self.umin;
        } else if (self.u - self.umin - period).abs() < delta {
            self.u = self.umin + period;
        }
        let (_, parameter, offset) =
            geom_int::adjust_periodic(self.u, self.umin, self.umax, period, 0.);
        self.u = parameter;
        self.du = offset;
        if self.du == 0. && surface.get_type()? == SurfaceType::Cylinder {
            self.adjust_cylinder(surface, period)?;
        }
        Ok(())
    }
    fn adjust_cylinder(
        &mut self,
        surface: &BRepAdaptorSurface,
        period: f64,
    ) -> Result<(), Failure> {
        let mut angular = max_edge_tolerance(&surface.face())? / surface.cylinder()?.radius();
        if angular < precision::p_confusion() {
            angular = precision::p_confusion();
        }
        if self.umin - self.u > angular {
            self.du = period;
        } else if self.u - self.umax > angular {
            self.du = -period;
        }
        Ok(())
    }
    fn adjust_v(&mut self, surface: &BRepAdaptorSurface) -> Result<(), Failure> {
        if !surface.is_v_periodic()? {
            return Ok(());
        }
        let period = surface.v_period()?;
        let delta = precision::p_confusion();
        if self.vmin - self.v > delta {
            self.dv = period;
        } else if self.v - self.vmax > delta {
            self.dv = -period;
        }
        let midpoint = 0.5 * (self.vmin + self.vmax);
        if self.vmax - self.vmin < period
            && self.dv != 0.
            && (self.v - midpoint).abs() < (self.v + self.dv - midpoint).abs()
        {
            self.dv = 0.;
        }
        Ok(())
    }
    fn classify_wide_domain(&mut self, surface: &BRepAdaptorSurface) -> Result<(), Failure> {
        // Native retains the adjusted u and adds du again for classification only.
        let mut point = Point2::new(self.u + self.du, self.v + self.dv);
        if surface.is_u_periodic()? {
            self.du += classified_shift(
                surface,
                point,
                self.umin,
                self.umax,
                surface.u_period()?,
                true,
            )?;
        }
        point = Point2::new(self.u + self.du, point.y());
        if surface.is_v_periodic()? {
            self.dv += classified_shift(
                surface,
                point,
                self.vmin,
                self.vmax,
                surface.v_period()?,
                false,
            )?;
        }
        Ok(())
    }
}
fn classified_shift(
    surface: &BRepAdaptorSurface,
    point: Point2,
    min: f64,
    max: f64,
    period: f64,
    u_axis: bool,
) -> Result<f64, Failure> {
    let delta = precision::p_confusion();
    let value = if u_axis { point.x() } else { point.y() };
    let wide = max - min - 2. * delta > period;
    if !wide {
        return Ok(0.);
    }
    if !(value > min + delta + period || value < max - delta - period) {
        return Ok(0.);
    }
    let classifier =
        BRepClassFaceClassifier::with_face_point2d(&surface.face(), &point, delta, false, 0.1)?;
    if classifier.state() != State::Out {
        return Ok(0.);
    }
    Ok(if value > min + delta + period {
        -period
    } else {
        period
    })
}
fn max_edge_tolerance(face: &Face) -> Result<f64, Failure> {
    let mut maximum = 0.;
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let tolerance = BRepTool::edge_tolerance(&topods::edge(&edge)?)?;
        if tolerance > maximum {
            maximum = tolerance;
        }
    }
    Ok(maximum)
}
