use super::*;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{
    cslib::{self, DerivativeStatus},
    gp::{Direction3, Line3, Vector3},
};
impl GeomTool<'_> {
    /// Compute an oriented surface normal, retaining outputs on singular failure; `GeomTool::normal(&surface,u,v,&mut point,&mut normal)?`.
    pub fn normal(
        surface: &Handle<BRepAdaptorSurface>,
        u: f64,
        v: f64,
        point: &mut Point3,
        normal: &mut Direction3,
    ) -> Result<bool, Failure> {
        let derivatives = surface.d1(u, v)?;
        *point = derivatives.point;
        let result = cslib::normal(derivatives.d1u, derivatives.d1v, precision::angular())?;
        if let Some(direction) = result.normal {
            *normal = direction;
        }
        if result.status != DerivativeStatus::Done && !second_normal(surface, u, v, point, normal)?
        {
            return Ok(false);
        }
        if surface.face().orientation() == Orientation::Reversed {
            *normal = normal.reversed();
        }
        Ok(true)
    }
    /// Squared midpoint distance to the supporting line, with OCC21645's short-chord fallback; `GeomTool::square_deflection_of_segment(first,last,middle)?`.
    pub fn square_deflection_of_segment(
        first: Point3,
        last: Point3,
        middle: Point3,
    ) -> Result<f64, Failure> {
        if first.square_distance(&last) > precision::square_confusion() {
            let direction = Direction3::from_vector(Vector3::between(first, last))?;
            return Ok(Line3::new(first, direction).square_distance(middle));
        }
        Ok(first.square_distance(&middle))
    }
}
fn second_normal(
    surface: &BRepAdaptorSurface,
    u: f64,
    v: f64,
    point: &mut Point3,
    normal: &mut Direction3,
) -> Result<bool, Failure> {
    let derivatives = surface.d2(u, v)?;
    *point = derivatives.point;
    let result = cslib::normal_from_second_derivatives(
        derivatives.d1u,
        derivatives.d1v,
        derivatives.d2u,
        derivatives.d2v,
        derivatives.d2uv,
        precision::angular(),
    );
    if let Some(direction) = result.normal {
        *normal = direction;
    }
    Ok(result.is_done())
}
