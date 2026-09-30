use super::closure::restricted_bounds;
use super::*;
use crate::shape_analysis::curve::adjust_by_period;
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkgeombase::extrema::{ExtFlag, ExtPS};
use rustycad_tkmath::gp::Vector3;
use rustycad_tkmath::{
    elslib,
    geom_abs::{Shape, SurfaceType},
};

enum SurfaceProjectionState {
    Unsupported,
    Evaluated,
    NewtonShortcut,
}

impl ShapeAnalysisSurface {
    /// Projects a point with analytical or extrema/isoline algorithms; `analysis.value_of_uv(point, 1e-7)?`.
    pub fn value_of_uv(&self, point: Point3, tolerance: f64) -> Result<Point2, Failure> {
        self.fields.write().gap = -1.;
        let mut uv = Point2::default();
        let computed = match self.project_surface_point(point, tolerance, &mut uv) {
            Ok(computed) => computed,
            Err(_) => {
                uv = projection_fallback(self.bounds());
                SurfaceProjectionState::Evaluated
            }
        };
        self.finish_uv_projection(point, uv, computed)
    }

    fn finish_uv_projection(
        &self,
        point: Point3,
        uv: Point2,
        computed: SurfaceProjectionState,
    ) -> Result<Point2, Failure> {
        if matches!(computed, SurfaceProjectionState::NewtonShortcut) {
            return Ok(uv);
        }
        if matches!(computed, SurfaceProjectionState::Unsupported) {
            self.fields.write().gap = -1.;
            return Ok(Point2::default());
        }
        if self.gap() <= 0. {
            self.fields.write().gap = point.distance(&self.value_uv(uv)?);
        }
        Ok(uv)
    }

    fn project_surface_point(
        &self,
        point: Point3,
        tolerance: f64,
        uv: &mut Point2,
    ) -> Result<SurfaceProjectionState, Failure> {
        let adaptor = self.adaptor3d();
        let kind = adaptor.get_type()?;
        let parameters = elementary_parameters(&adaptor, kind, point)?;
        if let Some(parameters) = parameters {
            *uv = self.adjust_analytic_uv(parameters, kind);
            return Ok(SurfaceProjectionState::Evaluated);
        }
        if !supports_general_projection(kind) {
            return Ok(SurfaceProjectionState::Unsupported);
        }
        Ok(if self.project_general_surface(point, tolerance, uv)? {
            SurfaceProjectionState::NewtonShortcut
        } else {
            SurfaceProjectionState::Evaluated
        })
    }

    fn adjust_analytic_uv(&self, (mut u, mut v): (f64, f64), kind: SurfaceType) -> Point2 {
        let (uf, ul, vf, vl) = self.bounds();
        if kind != SurfaceType::Plane {
            u += adjust_by_period(u, 0.5 * (uf + ul), 2. * std::f64::consts::PI);
        }
        if kind == SurfaceType::Torus {
            v += adjust_by_period(v, 0.5 * (vf + vl), 2. * std::f64::consts::PI);
        }
        Point2::new(u, v)
    }

    fn project_general_surface(
        &self,
        point: Point3,
        tolerance: f64,
        uv: &mut Point2,
    ) -> Result<bool, Failure> {
        let Some(bounds) = self.prepare_projection_bounds(point, tolerance, uv)? else {
            return Ok(true);
        };
        let Some(nearest) = self.extrema_projection(point, tolerance, restricted_bounds(bounds))?
        else {
            let gap = self.uv_from_iso(point, tolerance, uv)?;
            self.fields.write().gap = gap;
            return Ok(false);
        };
        *uv = nearest;
        self.refine_extrema_point(point, tolerance, uv)?;
        Ok(false)
    }

    fn prepare_projection_bounds(
        &self,
        point: Point3,
        tolerance: f64,
        uv: &mut Point2,
    ) -> Result<Option<SurfaceBounds>, Failure> {
        let mut bounds = self.bounds();
        *uv = Point2::new((bounds.0 + bounds.1) / 2., (bounds.2 + bounds.3) / 2.);
        if self.adaptor3d().get_type()? == SurfaceType::SurfaceOfExtrusion
            && precision::is_infinite(bounds.0)
            && precision::is_infinite(bounds.1)
        {
            if self.surface_newton(*uv, point, tolerance, uv)? != 0 {
                return Ok(None);
            }
            bounds.0 = -500.;
            bounds.1 = 500.;
        }
        Ok(Some(bounds))
    }

    fn extrema_projection(
        &self,
        point: Point3,
        tolerance: f64,
        bounds: SurfaceBounds,
    ) -> Result<Option<Point2>, Failure> {
        if self.extrema.read().is_none() {
            self.initialize_extrema(tolerance, bounds)?;
        }
        let mut guard = self.extrema.write();
        let extrema = guard.as_mut().unwrap();
        extrema.perform(point)?;
        if !extrema.is_done() || extrema.nb_ext()? == 0 {
            return Ok(None);
        }
        closest_extrema_parameter(extrema).map(Some)
    }

    fn initialize_extrema(&self, tolerance: f64, bounds: SurfaceBounds) -> Result<(), Failure> {
        if self.u_close_val() < 0. {
            self.is_u_closed(precision::confusion())?;
        }
        if self.v_close_val() < 0. {
            self.is_v_closed(precision::confusion())?;
        }
        let adaptor = self.adaptor3d();
        let delta = self.extrema_extension(tolerance)?;
        let mut extrema = ExtPS::new();
        extrema.set_flag(ExtFlag::Min);
        extrema.initialize_shared(
            &adaptor.cast(),
            (bounds.0 - delta[0], bounds.1 + delta[0]),
            (bounds.2 - delta[1], bounds.3 + delta[1]),
            (precision::p_confusion(), precision::p_confusion()),
        )?;
        *self.extrema.write() = Some(extrema);
        Ok(())
    }

    fn extrema_extension(&self, tolerance: f64) -> Result<[f64; 2], Failure> {
        if self.surface().is_kind_name("Geom_OffsetSurface") {
            return Ok([0.; 2]);
        }
        let adaptor = self.adaptor3d();
        let extension = self.fields.read().extension;
        Ok([
            std_min(extension[0], adaptor.u_resolution(tolerance)?),
            std_min(extension[1], adaptor.v_resolution(tolerance)?),
        ])
    }

    fn refine_extrema_point(
        &self,
        point: Point3,
        tolerance: f64,
        uv: &mut Point2,
    ) -> Result<(), Failure> {
        // OCC486: recompute distance because revolution extrema can return stale distances.
        let mut distance = point.distance(&self.value_uv(*uv)?);
        let mut candidate = *uv;
        let mut iso_distance = f64::MAX;
        if distance > tolerance {
            self.refine_newton_point(point, tolerance, uv, &mut candidate, &mut distance)?;
            if !self.is_local_projection(point, candidate, distance, tolerance)? {
                iso_distance = self.uv_from_iso(point, tolerance, &mut candidate)?;
            }
        }
        if distance > iso_distance {
            *uv = candidate;
            distance = iso_distance;
        }
        self.fields.write().gap = distance;
        Ok(())
    }

    fn refine_newton_point(
        &self,
        point: Point3,
        tolerance: f64,
        uv: &mut Point2,
        candidate: &mut Point2,
        distance: &mut f64,
    ) -> Result<(), Failure> {
        let mut solution = *candidate;
        if self.surface_newton(*candidate, point, tolerance, &mut solution)? == 0 {
            return Ok(());
        }
        let gap = point.distance(&self.value_uv(solution)?);
        if gap < *distance {
            *distance = gap;
            *uv = solution;
            *candidate = solution;
        }
        Ok(())
    }

    fn is_local_projection(
        &self,
        point: Point3,
        uv: Point2,
        distance: f64,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if distance >= 10. * tolerance || self.surface().continuity() == Shape::C0 {
            return Ok(false);
        }
        let derivatives = self.adaptor3d().d1(uv.x(), uv.y())?;
        let normal = derivatives.d1u.crossed(derivatives.d1v);
        let residual = Vector3::between(derivatives.point, point);
        let squared = normal.square_magnitude();
        if squared <= 1e-10 {
            return Ok(false);
        }
        let product = residual.dot(normal);
        Ok(residual.square_magnitude() - product * product / squared
            < precision::confusion().powi(2))
    }
}

fn projection_fallback(bounds: SurfaceBounds) -> Point2 {
    let u = if precision::is_infinite(bounds.0) {
        0.
    } else {
        (bounds.0 + bounds.1) / 2.
    };
    let v = if precision::is_infinite(bounds.2) {
        0.
    } else {
        (bounds.2 + bounds.3) / 2.
    };
    Point2::new(u, v)
}

fn elementary_parameters(
    adaptor: &GeomAdaptorSurface,
    kind: SurfaceType,
    point: Point3,
) -> Result<Option<(f64, f64)>, Failure> {
    Ok(match kind {
        SurfaceType::Plane => Some(elslib::parameters(&adaptor.plane()?, point)),
        SurfaceType::Cylinder => Some(elslib::parameters(&adaptor.cylinder()?, point)),
        SurfaceType::Cone => Some(elslib::parameters(&adaptor.cone()?, point)),
        SurfaceType::Sphere => Some(elslib::parameters(&adaptor.sphere()?, point)),
        SurfaceType::Torus => Some(elslib::parameters(&adaptor.torus()?, point)),
        _ => None,
    })
}

pub(super) fn supports_general_projection(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::BezierSurface
            | SurfaceType::BSplineSurface
            | SurfaceType::SurfaceOfExtrusion
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::OffsetSurface
    )
}

fn closest_extrema_parameter(extrema: &ExtPS<'_>) -> Result<Point2, Failure> {
    let mut closest = 1;
    let mut minimum = extrema.square_distance(1)?;
    for index in 2..=extrema.nb_ext()? {
        let squared = extrema.square_distance(index)?;
        if minimum > squared {
            minimum = squared;
            closest = index;
        }
    }
    let (u, v) = extrema.point(closest)?.parameter();
    Ok(Point2::new(u, v))
}
