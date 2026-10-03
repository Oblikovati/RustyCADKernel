// TranslatePCurve retains OCCT's choice of the farther periodic boundary (S4135).
use crate::shape_analysis::ShapeAnalysisSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{Point2, Vector2};

pub(super) fn second_seam_pcurve(
    first: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    analysis: &Handle<ShapeAnalysisSurface>,
    precision: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let second: Handle<dyn Geom2dCurve> = first.copy()?.cast();
    let (uf, ul, vf, vl) = surface.bounds();
    if analysis.is_u_closed(precision)? && !analysis.is_v_closed(precision)? {
        second.translate(Vector2::new(ul - uf, 0.0))?;
    } else if analysis.is_v_closed(precision)? && !analysis.is_u_closed(precision)? {
        second.translate(Vector2::new(0.0, vl - vf))?;
    } else if analysis.is_u_closed(-1.0)? && analysis.is_v_closed(-1.0)? {
        return translate_pcurve(&analysis.surface(), &second, precision);
    }
    Ok(second)
}

fn translate_pcurve(
    surface: &Handle<dyn GeomSurface>,
    curve: &Handle<dyn Geom2dCurve>,
    tolerance: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let line: Handle<Geom2dLine> = curve.cast();
    if let Some(line) = line.get() {
        return Ok(translate_line(surface.bounds(), line, curve, tolerance));
    }
    let spline: Handle<Geom2dBSplineCurve> = curve.cast();
    let Some(spline) = spline.get() else {
        return Ok(curve.clone());
    };
    translate_spline(surface.bounds(), spline, curve, tolerance)
}

fn shifted_coordinate(value: f64, first: f64, last: f64) -> f64 {
    if (value - first).abs() < (value - last).abs() {
        value + (last - first)
    } else {
        value - (last - first)
    }
}

fn translate_line(
    bounds: (f64, f64, f64, f64),
    line: &Geom2dLine,
    original: &Handle<dyn Geom2dCurve>,
    tolerance: f64,
) -> Handle<dyn Geom2dCurve> {
    let (uf, ul, vf, vl) = bounds;
    let point = line.location();
    let direction = line.direction();
    let mut result = original.clone();
    if direction.x().abs() <= tolerance && direction.y().abs() >= tolerance {
        let location = Point2::new(shifted_coordinate(point.x(), uf, ul), point.y());
        result = Handle::new(Geom2dLine::from_point(location, direction)).cast();
    }
    if direction.x().abs() >= tolerance && direction.y().abs() <= tolerance {
        let location = Point2::new(point.x(), shifted_coordinate(point.y(), vf, vl));
        result = Handle::new(Geom2dLine::from_point(location, direction)).cast();
    }
    result
}

fn translate_spline(
    bounds: (f64, f64, f64, f64),
    spline: &Geom2dBSplineCurve,
    original: &Handle<dyn Geom2dCurve>,
    tolerance: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (uf, ul, vf, vl) = bounds;
    let first = spline.start_point()?;
    let direction = Vector2::between(first, spline.end_point()?);
    let copy: Handle<dyn Geom2dCurve> = original.copy()?.cast();
    if direction.is_parallel(Vector2::new(0., vl - vf), tolerance)? {
        let delta = if (first.x() - uf).abs() < (first.x() - ul).abs() {
            ul - uf
        } else {
            uf - ul
        };
        copy.translate(Vector2::new(delta, 0.))?;
        return Ok(copy);
    }
    if direction.is_parallel(Vector2::new(ul - uf, 0.), tolerance)? {
        let delta = if (first.y() - vf).abs() < (first.y() - vl).abs() {
            vl - vf
        } else {
            vf - vl
        };
        copy.translate(Vector2::new(0., delta))?;
        return Ok(copy);
    }
    Ok(original.clone())
}
