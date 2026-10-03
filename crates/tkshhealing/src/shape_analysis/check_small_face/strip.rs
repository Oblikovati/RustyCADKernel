use super::spot::{ShapeAnalysisCheckSmallFace, axis_is_small, update_bounds};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomBezierSurface};
use rustycad_tkmath::gp::Point3;

impl ShapeAnalysisCheckSmallFace {
    /// Uses the native 1e-7 fallback when deciding whether a support is a strip.
    /// `analyzer.is_strip_support(&face)?`.
    pub fn is_strip_support(&mut self, face: &Face) -> Result<bool, Failure> {
        self.is_strip_support_with_tolerance(face, -1.0)
    }

    /// Scans Bezier or B-spline surface poles and records DONE1/DONE2 when one direction
    /// collapses within `tolerance`; `analyzer.is_strip_support_with_tolerance(&face, 1e-6)?`.
    pub fn is_strip_support_with_tolerance(
        &mut self,
        face: &Face,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let tolerance = if tolerance < 0.0 { 1e-7 } else { tolerance };
        let surface = BRepTool::surface_with_location(face)?.0;
        let result = strip_status(&surface, tolerance)?;
        let Some(status) = result else {
            return Ok(false);
        };
        self.strip_status = ShapeExtend::encode_status(status);
        Ok(true)
    }

    /// Decodes the last successful `IsStripSupport` result; `analyzer.status_strip(ShapeExtendStatus::Done2)`.
    pub fn status_strip(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.strip_status, status)
    }
}

fn strip_status(
    surface: &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    tolerance: f64,
) -> Result<Option<ShapeExtendStatus>, Failure> {
    let bspline = surface.cast::<GeomBSplineSurface>();
    if let Some(surface) = bspline.get() {
        return strip_status_for_poles(
            surface.nb_u_poles(),
            surface.nb_v_poles(),
            tolerance,
            |u, v| surface.pole(u, v),
        );
    }
    let bezier = surface.cast::<GeomBezierSurface>();
    let Some(surface) = bezier.get() else {
        return Ok(None);
    };
    strip_status_for_poles(
        surface.nb_u_poles(),
        surface.nb_v_poles(),
        tolerance,
        |u, v| surface.pole(u, v),
    )
}

fn strip_status_for_poles(
    u_count: i32,
    v_count: i32,
    tolerance: f64,
    pole: impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<Option<ShapeExtendStatus>, Failure> {
    if lines_are_small(u_count, v_count, tolerance, &pole)? {
        return Ok(Some(ShapeExtendStatus::Done2));
    }
    if lines_are_small(v_count, u_count, tolerance, |v, u| pole(u, v))? {
        return Ok(Some(ShapeExtendStatus::Done1));
    }
    Ok(None)
}

fn lines_are_small(
    line_count: i32,
    pole_count: i32,
    tolerance: f64,
    pole: impl Fn(i32, i32) -> Result<Point3, Failure>,
) -> Result<bool, Failure> {
    for line in 1..=line_count {
        if !pole_line_is_small(pole_count, tolerance, |index| pole(line, index))? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn pole_line_is_small(
    pole_count: i32,
    tolerance: f64,
    mut pole: impl FnMut(i32) -> Result<Point3, Failure>,
) -> Result<bool, Failure> {
    let first = pole(1)?;
    let (mut minimum, mut maximum) = (first, first);
    for index in 2..=pole_count {
        update_bounds(&mut minimum, &mut maximum, pole(index)?, true);
    }
    Ok(axis_is_small(maximum.x() - minimum.x(), tolerance)
        && axis_is_small(maximum.y() - minimum.y(), tolerance)
        && axis_is_small(maximum.z() - minimum.z(), tolerance))
}
