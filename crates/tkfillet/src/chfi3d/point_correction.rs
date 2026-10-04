use rustycad_tkbrep::{brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::{geom_abs::SurfaceType, gp::Point2};
pub(super) fn corrected_point(face: &Face, point: Point2) -> Result<Point2, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    if (surface.get_type()? as i32) < SurfaceType::BezierSurface as i32 {
        return Ok(point);
    }
    let surface = BRepAdaptorSurface::with_face(face, true)?;
    Ok(Point2::new(
        interior_coordinate(
            point.x(),
            surface.first_u_parameter()?,
            surface.last_u_parameter()?,
        ),
        interior_coordinate(
            point.y(),
            surface.first_v_parameter()?,
            surface.last_v_parameter()?,
        ),
    ))
}
fn interior_coordinate(mut value: f64, first: f64, last: f64) -> f64 {
    if precision::is_infinite(first) || precision::is_infinite(last) {
        return value;
    }
    let epsilon = (0.01 * (last - first)).max(precision::p_confusion());
    if (value - first).abs() < epsilon {
        value = first + epsilon;
    }
    if (value - last).abs() < epsilon {
        value = last - epsilon;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::interior_coordinate;
    #[test]
    fn native_coordinate_correction_order_and_infinite_bounds() {
        assert_eq!(interior_coordinate(0., 0., 1.), 0.01);
        assert_eq!(interior_coordinate(1., 0., 1.), 0.99);
        assert_eq!(interior_coordinate(0.5, 0., 1.), 0.5);
        assert_eq!(interior_coordinate(0., 0., f64::INFINITY), 0.);
        // Both native if statements run; the last boundary correction wins.
        let tiny = rustycad_tkernel::precision::p_confusion() * 0.5;
        assert_eq!(
            interior_coordinate(0., 0., tiny),
            tiny - rustycad_tkernel::precision::p_confusion()
        );
    }
}
