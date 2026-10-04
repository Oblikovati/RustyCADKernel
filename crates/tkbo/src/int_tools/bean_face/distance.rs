//! Native point projection with bounded isoparametric-curve fallback.
use super::{BeanFaceIntersector, Failure};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Point3;

#[derive(Clone, Copy)]
struct Projection {
    distance: f64,
    u: f64,
    v: f64,
}

#[derive(Clone, Copy)]
struct BoundaryIso {
    is_u: bool,
    fixed: f64,
    first: f64,
    last: f64,
}

impl BeanFaceIntersector {
    /// Projects a curve sample, falling back to surface boundaries; `solver.distance(parameter)?`.
    pub fn distance(&self, parameter: f64) -> Result<f64, Failure> {
        let point = self.curve.get().expect("bean curve").value(parameter)?;
        Ok(self.project_bean_point(point)?.distance)
    }

    /// Returns distance and clamps UV outputs to configured bounds; `solver.distance_uv(t,&mut u,&mut v)?`.
    pub fn distance_uv(&self, parameter: f64, u: &mut f64, v: &mut f64) -> Result<f64, Failure> {
        let point = self.curve.get().expect("bean curve").value(parameter)?;
        *u = self.bounds[0];
        *v = self.bounds[2];
        let projected = self.project_bean_point(point)?;
        *u = ordered_bound(projected.u, self.bounds[0], self.bounds[1]);
        *v = ordered_bound(projected.v, self.bounds[2], self.bounds[3]);
        Ok(projected.distance)
    }

    fn project_bean_point(&self, point: Point3) -> Result<Projection, Failure> {
        if let Some(projected) = self.project_sample_surface(point)? {
            return Ok(projected);
        }
        let mut best = Projection {
            distance: f64::MAX,
            u: self.bounds[0],
            v: self.bounds[2],
        };
        for index in 0..4 {
            self.project_boundary_iso(point, boundary_iso(self.bounds, index), &mut best)?;
        }
        Ok(best)
    }

    fn project_sample_surface(&self, point: Point3) -> Result<Option<Projection>, Failure> {
        let context = self.context.get().ok_or_else(|| {
            Failure::new("BeanFace Distance received null context; expected initialized context")
        })?;
        let cached = context.proj_ps(&self.surface.get().expect("bean surface").face())?;
        let mut projector = cached.write();
        projector.perform(point)?;
        if !projector.is_done() || projector.nb_points()? == 0 {
            return Ok(None);
        }
        let (u, v) = projector.lower_distance_parameters()?;
        Ok(Some(Projection {
            distance: projector.lower_distance()?,
            u,
            v,
        }))
    }

    fn project_boundary_iso(
        &self,
        point: Point3,
        iso: BoundaryIso,
        best: &mut Projection,
    ) -> Result<(), Failure> {
        let surface = self.surface.get().expect("bean surface");
        let first = iso.value(surface, iso.first)?;
        let last = iso.value(surface, iso.last)?;
        let middle = iso.value(surface, (iso.first + iso.last) * 0.5)?;
        let collapsed = first.is_equal(&last, self.criteria)
            && first.is_equal(&middle, self.criteria)
            && last.is_equal(&middle, self.criteria);
        if !collapsed && self.project_iso_curve(point, iso, best)? {
            return Ok(());
        }
        improve_projection(best, point.distance(&first), iso, iso.first);
        improve_projection(best, point.distance(&last), iso, iso.last);
        Ok(())
    }

    fn project_iso_curve(
        &self,
        point: Point3,
        iso: BoundaryIso,
        best: &mut Projection,
    ) -> Result<bool, Failure> {
        let surface = self
            .transformed_surface
            .get()
            .expect("initialized transformed bean surface");
        let curve = if iso.is_u {
            surface.u_iso(iso.fixed)?
        } else {
            surface.v_iso(iso.fixed)?
        };
        let projector = ProjectPointOnCurve::with_range(point, &curve, iso.first, iso.last)?;
        if projector.nb_points() == 0 {
            return Ok(false);
        }
        improve_projection(
            best,
            projector.lower_distance()?,
            iso,
            projector.lower_distance_parameter()?,
        );
        Ok(true)
    }
}

impl BoundaryIso {
    fn value(self, surface: &dyn Adaptor3dSurface, parameter: f64) -> Result<Point3, Failure> {
        if self.is_u {
            return surface.value(self.fixed, parameter);
        }
        surface.value(parameter, self.fixed)
    }
}

fn boundary_iso(bounds: [f64; 4], index: usize) -> BoundaryIso {
    let is_u = index < 2;
    let offset = if is_u { 2 } else { 0 };
    BoundaryIso {
        is_u,
        fixed: bounds[index],
        first: bounds[offset],
        last: bounds[offset + 1],
    }
}

fn improve_projection(best: &mut Projection, distance: f64, iso: BoundaryIso, parameter: f64) {
    if distance.partial_cmp(&best.distance) != Some(std::cmp::Ordering::Less) {
        return;
    }
    best.distance = distance;
    best.u = if iso.is_u { iso.fixed } else { parameter };
    best.v = if iso.is_u { parameter } else { iso.fixed };
}

fn ordered_bound(mut parameter: f64, lower: f64, upper: f64) -> f64 {
    if lower > parameter {
        parameter = lower;
    }
    if upper < parameter {
        parameter = upper;
    }
    parameter
}
