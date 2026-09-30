//! Shared geometric construction for native open and closed conic paths.
use super::surface_policy;
use super::*;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::{
    geom::{
        GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine, GeomParabola, GeomTrimmedCurve,
    },
    top_abs::State,
};
use rustycad_tkgeomalgo::{
    geom_int::GeomIntIntSS,
    int_patch::{IntPatchGLine, IntPatchIType, IntPatchLine},
    int_start::TheTopolTool,
};
use rustycad_tkmath::gp::{Point2, Point3};
use std::rc::Rc;
pub(super) fn geometry(line: &Handle<dyn IntPatchLine>) -> Result<Handle<dyn GeomCurve>, Failure> {
    let conic = line.cast::<IntPatchGLine>();
    Ok(match line.arc_type() {
        IntPatchIType::Lin => Handle::new(GeomLine::from_lin(conic.line()?)).cast(),
        IntPatchIType::Circle => Handle::new(GeomCircle::from_circ(conic.circle()?)).cast(),
        IntPatchIType::Ellipse => Handle::new(GeomEllipse::from_elips(conic.ellipse()?)).cast(),
        IntPatchIType::Parabola => Handle::new(GeomParabola::from_parab(conic.parabola()?)).cast(),
        _ => Handle::new(GeomHyperbola::from_hypr(conic.hyperbola()?)).cast(),
    })
}
pub(super) fn trimmed(
    curve: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast())
}
pub(super) fn trimmed_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    (first, last): (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast())
}
pub(super) fn empty_descriptor(geometry: &Handle<dyn GeomCurve>) -> Curve {
    Curve::new(geometry, &Handle::default(), &Handle::default(), 0., 0.)
}
pub(super) fn set_pcurve(curve: &mut Curve, index: usize, pcurve: &Handle<dyn Geom2dCurve>) {
    if index == 0 {
        curve.set_first_curve2d(pcurve);
    } else {
        curve.set_second_curve2d(pcurve);
    }
}
impl FaceFace {
    pub(super) fn project_pcurve(
        &self,
        index: usize,
        curve: &Handle<dyn GeomCurve>,
        range: (f64, f64),
        tolerance: &mut f64,
        bounded: bool,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let mut pcurve = Handle::default();
        let surface = &self.surfaces[index];
        let geometry = surface.surface();
        if bounded {
            return bounded_projection(surface, curve, range, tolerance);
        } else {
            GeomIntIntSS::build_pcurves(
                range.0,
                range.1,
                tolerance,
                &geometry,
                curve,
                &mut pcurve,
            )?;
        }
        Ok(pcurve)
    }
    pub(super) fn inside_domains(
        &self,
        point: Point3,
        domains: &[Rc<dyn TheTopolTool>; 2],
    ) -> Result<bool, Failure> {
        let uv = surface_policy::parameters(&self.surfaces[0], &self.surfaces[1], point)?;
        let first = domains[0].classify(Point2::new(uv[0], uv[1]), precision::confusion(), true)?;
        if first == State::Out {
            return Ok(false);
        }
        Ok(
            domains[1].classify(Point2::new(uv[2], uv[3]), precision::confusion(), true)?
                != State::Out,
        )
    }
}

fn bounded_projection(
    surface: &GeomAdaptorSurface,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: &mut f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let bounds = (
        surface.first_u_parameter()?,
        surface.last_u_parameter()?,
        surface.first_v_parameter()?,
        surface.last_v_parameter()?,
    );
    let geometry = surface.surface();
    let mut pcurve = Handle::default();
    GeomIntIntSS::build_pcurves_bounds(
        range.0,
        range.1,
        bounds,
        tolerance,
        &geometry,
        curve,
        &mut pcurve,
    )?;
    Ok(pcurve)
}
