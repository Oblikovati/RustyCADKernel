use super::*;
use rustycad_tkernel::precision;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkmath::gp::Vector3;

struct NewtonSearch {
    bounds: SurfaceBounds,
    initial: f64,
    point: Point3,
}

struct NewtonStep {
    delta: Point2,
    residual: Vector3,
    normal: Vector3,
    normal_squared: f64,
    tangent_product: f64,
}

impl ShapeAnalysisSurface {
    /// Performs native 25-iteration Newton projection; zero leaves `solution` unchanged.
    /// `let status = analysis.surface_newton(hint, point, 1e-7, &mut solution)?;`
    pub fn surface_newton(
        &self,
        hint: Point2,
        point: Point3,
        tolerance: f64,
        solution: &mut Point2,
    ) -> Result<i32, Failure> {
        let search = NewtonSearch {
            bounds: self.newton_bounds(tolerance)?,
            initial: Vector3::between(self.value_uv(hint)?, point).square_magnitude(),
            point,
        };
        let mut current = hint;
        for _ in 0..25 {
            if let Some(status) = self.newton_iteration(&search, &mut current)? {
                if status != 0 {
                    *solution = current;
                }
                return Ok(status);
            }
        }
        Ok(0)
    }

    fn newton_iteration(
        &self,
        search: &NewtonSearch,
        current: &mut Point2,
    ) -> Result<Option<i32>, Failure> {
        let Some(step) = self.newton_step(*current, search.point)? else {
            return Ok(Some(0));
        };
        *current = Point2::new(current.x() + step.delta.x(), current.y() + step.delta.y());
        if outside_newton_bounds(*current, search.bounds) {
            return Ok(Some(0));
        }
        if step.unfinished(*current) {
            return Ok(None);
        }
        if !step.converged(search.initial) {
            return Ok(Some(0));
        }
        Ok(Some(if step.normal_squared < 0.01 * step.tangent_product {
            2
        } else {
            1
        }))
    }

    fn newton_bounds(&self, tolerance: f64) -> Result<SurfaceBounds, Failure> {
        let adaptor = self.adaptor3d();
        let (uf, ul, vf, vl) = self.bounds();
        let du = adaptor.u_resolution(tolerance)?;
        let dv = adaptor.v_resolution(tolerance)?;
        Ok((uf - du, ul + du, vf - dv, vl + dv))
    }

    fn newton_step(&self, uv: Point2, point: Point3) -> Result<Option<NewtonStep>, Failure> {
        let derivatives = self.adaptor3d().d2(uv.x(), uv.y())?;
        let normal = derivatives.d1u.crossed(derivatives.d1v);
        let normal_squared = normal.square_magnitude();
        if normal_squared < 1e-10 || precision::is_positive_infinite(normal_squared) {
            return Ok(None);
        }
        let residual = Vector3::between(self.value_uv(uv)?, point);
        Ok(newton_step_terms(
            derivatives,
            normal,
            residual,
            normal_squared,
        ))
    }
}

fn newton_step_terms(
    d: GeomSurfaceD2,
    normal: Vector3,
    residual: Vector3,
    normal_squared: f64,
) -> Option<NewtonStep> {
    let (ru2, rv2) = (d.d1u.square_magnitude(), d.d1v.square_magnitude());
    let (determinant, (suu, svv, suv)) = newton_discriminant(d, residual, normal_squared);
    if determinant.abs() < 1e-10 {
        return None;
    }
    let fraction = 1. / determinant;
    let du = residual.dot(normal.crossed(d.d1v) + d.d1u * svv - d.d1v * suv) * fraction;
    let dv = residual.dot(d.d1u.crossed(normal) + d.d1v * suu - d.d1u * suv) * fraction;
    Some(NewtonStep::from_terms(
        Point2::new(du, dv),
        residual,
        normal,
        normal_squared,
        ru2 * rv2,
    ))
}

impl NewtonStep {
    fn from_terms(
        delta: Point2,
        residual: Vector3,
        normal: Vector3,
        normal_squared: f64,
        tangent_product: f64,
    ) -> Self {
        Self {
            delta,
            residual,
            normal,
            normal_squared,
            tangent_product,
        }
    }

    fn unfinished(&self, uv: Point2) -> bool {
        let resolution = std_max(1e-12, (uv.x() + uv.y()) * 10e-16);
        self.delta.x().abs() + self.delta.y().abs() > resolution
    }

    fn converged(&self, initial_squared: f64) -> bool {
        let squared = self.residual.square_magnitude();
        if squared > initial_squared {
            return false;
        }
        let normal_residual = self.residual.dot(self.normal);
        let deviation = squared - normal_residual * normal_residual / self.normal_squared;
        deviation.partial_cmp(&precision::confusion().powi(2)) != Some(std::cmp::Ordering::Greater)
    }
}

fn outside_newton_bounds(uv: Point2, bounds: SurfaceBounds) -> bool {
    uv.x() < bounds.0 || uv.x() > bounds.1 || uv.y() < bounds.2 || uv.y() > bounds.3
}

fn newton_discriminant(
    d: GeomSurfaceD2,
    residual: Vector3,
    normal_squared: f64,
) -> (f64, (f64, f64, f64)) {
    let (ru2, rv2) = (d.d1u.square_magnitude(), d.d1v.square_magnitude());
    let (suu, svv, suv) = (
        residual.dot(d.d2u),
        residual.dot(d.d2v),
        residual.dot(d.d2uv),
    );
    let determinant = -normal_squared + rv2 * suu + ru2 * svv - 2. * suv * d.d1u.dot(d.d1v)
        + suv * suv
        - suu * svv;
    (determinant, (suu, svv, suv))
}
