use super::*;
use rustycad_tkernel::precision;
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};

impl ShapeAnalysisSurface {
    /// Projects using a previous solution and native Newton fallback; `analysis.next_value_of_uv(hint, point, 1e-7, -1.)?`.
    pub fn next_value_of_uv(
        &self,
        hint: Point2,
        point: Point3,
        tolerance: f64,
        maximum: f64,
    ) -> Result<Point2, Failure> {
        let kind = self.adaptor3d().get_type()?;
        if !super::projection::supports_general_projection(kind) {
            return self.value_of_uv(point, tolerance);
        }
        if kind == SurfaceType::BSplineSurface && self.hint_near_c0_knot(hint)? {
            return self.value_of_uv(point, tolerance);
        }
        let mut solution = Point2::default();
        let status = self.surface_newton(hint, point, tolerance, &mut solution)?;
        if status == 0 {
            return self.value_of_uv(point, tolerance);
        }
        self.finish_next_projection(point, tolerance, maximum, solution, status)
    }

    fn finish_next_projection(
        &self,
        point: Point3,
        tolerance: f64,
        maximum: f64,
        solution: Point2,
        status: i32,
    ) -> Result<Point2, Failure> {
        let gap = point.distance(&self.value_uv(solution)?);
        if status == 2 || (maximum > 0. && gap - maximum > precision::confusion()) {
            let mut candidate = solution;
            let iso_gap = self.uv_from_iso(point, tolerance, &mut candidate)?;
            self.fields.write().gap = iso_gap;
            if gap >= iso_gap {
                return Ok(candidate);
            }
        }
        self.fields.write().gap = gap;
        Ok(solution)
    }

    fn hint_near_c0_knot(&self, hint: Point2) -> Result<bool, Failure> {
        let adaptor = self.adaptor3d();
        let spline = adaptor.bspline()?;
        if adaptor.u_continuity()? == Shape::C0 && near_spline_knot(&spline, true, hint.x())? {
            return Ok(true);
        }
        Ok(adaptor.v_continuity()? == Shape::C0 && near_spline_knot(&spline, false, hint.y())?)
    }
}

fn near_spline_knot(
    spline: &GeomBSplineSurface,
    u_axis: bool,
    parameter: f64,
) -> Result<bool, Failure> {
    let (first, last) = if u_axis {
        (spline.first_u_knot_index()?, spline.last_u_knot_index()?)
    } else {
        (spline.first_v_knot_index()?, spline.last_v_knot_index()?)
    };
    for index in first..=last {
        let knot = if u_axis {
            spline.u_knot(index)?
        } else {
            spline.v_knot(index)?
        };
        if (knot - parameter).abs() < precision::confusion() {
            return Ok(true);
        }
    }
    Ok(false)
}
