// PerformPlanes from IntTools_FaceFace.cxx at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Plane intersections bypass general curve tolerance recomputation.
use super::{Curve, Tools};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomCurve, GeomLine, GeomTrimmedCurve},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkgeombase::{
    int_ana::{QuadQuadGeo, ResultType},
    proj_lib::Plane,
};
use rustycad_tkmath::gp::{Line2, Line3, Plane3};
pub(super) struct PlaneIntersection<'a> {
    pub(super) surfaces: &'a [Handle<GeomAdaptorSurface>; 2],
    pub(super) face_tolerances: [f64; 2],
    pub(super) angular: f64,
    pub(super) tangential: f64,
    pub(super) approximation: [bool; 2],
}
impl PlaneIntersection<'_> {
    pub(super) fn perform(
        &self,
        curves: &mut Vec<Curve>,
        tangent: &mut bool,
    ) -> Result<(), Failure> {
        let mut intersection = QuadQuadGeo::new();
        let planes = [self.surfaces[0].plane()?, self.surfaces[1].plane()?];
        intersection.perform_planes(planes[0], planes[1], self.angular, self.tangential)?;
        *tangent = intersection.is_done() && intersection.type_inter()? == ResultType::Same;
        if !intersection.is_done() || *tangent || intersection.type_inter()? == ResultType::Empty {
            return Ok(());
        }
        self.append_line(intersection.line(1)?, planes, curves)
    }
    fn append_line(
        &self,
        line: Line3,
        planes: [Plane3; 2],
        curves: &mut Vec<Curve>,
    ) -> Result<(), Failure> {
        let pcurves = [
            Plane::with_line(&planes[0], &line)?.line()?,
            Plane::with_line(&planes[1], &line)?.line()?,
        ];
        let Some(range) = self.interval(pcurves)? else {
            return Ok(());
        };
        let mut curve = self.curve(line, pcurves, range)?;
        let angle = planes[0]
            .position()
            .direction()
            .angle(&planes[1].position().direction());
        self.set_tolerances(&mut curve, angle);
        curves.push(curve);
        Ok(())
    }
    fn set_tolerances(&self, curve: &mut Curve, angle: f64) {
        let [first, second] = self.face_tolerances;
        let distance = Tools::compute_int_range(first, second, angle);
        curve.set_tolerance(if first < second { second } else { first });
        curve.set_tangential_tolerance((distance * distance + first * first).sqrt());
    }
    fn interval(&self, pcurves: [Line2; 2]) -> Result<Option<(f64, f64)>, Failure> {
        let Some(first) = self.classified(0, pcurves[0])? else {
            return Ok(None);
        };
        let Some(second) = self.classified(1, pcurves[1])? else {
            return Ok(None);
        };
        Ok(common_interval(first, second, self.tangential))
    }
    fn classified(&self, index: usize, line: Line2) -> Result<Option<(f64, f64)>, Failure> {
        let bounds = surface_bounds(&self.surfaces[index])?;
        super::plane_clip::classify_line(bounds, line, self.tangential)
    }
    fn curve(&self, line: Line3, pcurves: [Line2; 2], range: (f64, f64)) -> Result<Curve, Failure> {
        let mut curve = spatial_line(line, range)?;
        if self.approximation[0] {
            curve.set_first_curve2d(&plane_pcurve(pcurves[0], range)?);
        }
        if self.approximation[1] {
            curve.set_second_curve2d(&plane_pcurve(pcurves[1], range)?);
        } else {
            // Pinned PerformPlanes clears the FIRST pcurve when approximation2 is false.
            curve.set_first_curve2d(&Handle::default());
        }
        Ok(curve)
    }
}
fn plane_pcurve(line: Line2, range: (f64, f64)) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let geometry = Handle::new(Geom2dLine::new(line.position())).cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(
        &geometry, range.0, range.1, true, true,
    )?)
    .cast())
}
fn surface_bounds(surface: &GeomAdaptorSurface) -> Result<[f64; 4], Failure> {
    Ok([
        surface.first_u_parameter()?,
        surface.last_u_parameter()?,
        surface.first_v_parameter()?,
        surface.last_v_parameter()?,
    ])
}

fn common_interval(
    (p11, p12): (f64, f64),
    (p21, p22): (f64, f64),
    tolerance: f64,
) -> Option<(f64, f64)> {
    if p21 >= p12 || p22 <= p11 {
        return None;
    }
    let low = if p11 < p21 { p21 } else { p11 };
    let high = if p22 < p12 { p22 } else { p12 };
    if high - low <= tolerance {
        None
    } else {
        Some((low, high))
    }
}
fn spatial_line(line: Line3, range: (f64, f64)) -> Result<Curve, Failure> {
    let geometry: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(line.position())).cast();
    let trimmed = Handle::new(GeomTrimmedCurve::new(
        &geometry, range.0, range.1, true, true,
    )?)
    .cast();
    Ok(Curve::new(
        &trimmed,
        &Handle::default(),
        &Handle::default(),
        0.,
        0.,
    ))
}
