//! Preserve native signed-span sampling, including its reversed denominator.
use super::{Adaptor3dSurface, BRepAdaptorSurface, BSurfaceTool, Failure, SurfaceType};
use crate::hlr_algo::c_int_truncation;
impl BSurfaceTool {
    /// Native U sampling count; `BSurfaceTool::nb_samples_u(&surface)?`.
    pub fn nb_samples_u(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        Ok(match surface.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + surface.nb_u_poles()?,
            SurfaceType::BSplineSurface => surface
                .nb_u_knots()?
                .wrapping_mul(surface.u_degree()?)
                .max(2),
            SurfaceType::Torus => 20,
            _ => 10,
        })
    }
    /// Native V sampling count; `BSurfaceTool::nb_samples_v(&surface)?`.
    pub fn nb_samples_v(surface: &BRepAdaptorSurface) -> Result<i32, Failure> {
        Ok(match surface.get_type()? {
            SurfaceType::Plane => 2,
            SurfaceType::BezierSurface => 3 + surface.nb_v_poles()?,
            SurfaceType::BSplineSurface => surface
                .nb_v_knots()?
                .wrapping_mul(surface.v_degree()?)
                .max(2),
            SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
            | SurfaceType::SurfaceOfRevolution
            | SurfaceType::SurfaceOfExtrusion => 15,
            _ => 10,
        })
    }
    /// Native U range rule; `BSurfaceTool::nb_samples_u_range(&surface,0.,1.)?`.
    pub fn nb_samples_u_range(
        surface: &BRepAdaptorSurface,
        first: f64,
        last: f64,
    ) -> Result<i32, Failure> {
        let count = Self::nb_samples_u(surface)?;
        if count <= 10 {
            return Ok(count);
        }
        Ok(range_samples(
            count,
            first,
            last,
            surface.first_u_parameter()?,
            surface.last_u_parameter()?,
        ))
    }
    /// Native V range rule; `BSurfaceTool::nb_samples_v_range(&surface,0.,1.)?`.
    pub fn nb_samples_v_range(
        surface: &BRepAdaptorSurface,
        first: f64,
        last: f64,
    ) -> Result<i32, Failure> {
        let count = Self::nb_samples_v(surface)?;
        if count <= 10 {
            return Ok(count);
        }
        Ok(range_samples(
            count,
            first,
            last,
            surface.first_v_parameter()?,
            surface.last_v_parameter()?,
        ))
    }
}
fn range_samples(count: i32, first: f64, last: f64, bound_first: f64, bound_last: f64) -> i32 {
    // OCCT 3d097a03 deliberately retained: First-Last, not Last-First.
    let multiplier = c_int_truncation((last - first) / (bound_first - bound_last));
    count.wrapping_mul(multiplier).min(count).max(5)
}
#[cfg(test)]
mod tests {
    use super::range_samples;
    #[test]
    fn reversed_denominator_and_integer_cast_are_preserved() {
        assert_eq!(range_samples(20, 0., 1., 0., 1.), 5);
        assert_eq!(range_samples(20, 1., 0., 0., 1.), 20);
        assert_eq!(range_samples(20, 0., 0., 0., 0.), 5);
        assert_eq!(range_samples(15, 0., 1., 0., 0.), 5);
    }
}
