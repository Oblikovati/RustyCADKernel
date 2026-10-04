use super::*;
use crate::int_tools::Tools;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomRectangularTrimmedSurface, GeomSurface, SurfaceBounds},
    geom_adaptor::GeomAdaptorSurface,
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Direction2, Point2};
struct SeamPeriods {
    bounds: SurfaceBounds,
    u: f64,
    v: f64,
}
/// Duplicate a boundary pcurve across a closed surface period; `split_seam_on_face(&split,&face)?`.
pub fn split_seam_on_face(split: &Edge, face: &Face) -> Result<bool, Failure> {
    let mut split = split.clone();
    split.set_orientation(Orientation::Forward);
    let tolerance = BRepTool::edge_tolerance(&split)?;
    let surface = BRepTool::surface(face)?;
    let Some(periods) = surface_periods(&surface, tolerance)? else {
        return Ok(false);
    };
    let curve = BRepTool::curve_on_face(&split, face)?;
    let range = (curve.first, curve.last);
    let (point, tangent) = curve
        .pcurve
        .d1(Tools::intermediate_point(range.0, range.1))?;
    let direction = Direction2::from_vector(tangent)?;
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    let (translated, left) = translated_point(
        point,
        &periods,
        adaptor.u_resolution(tolerance)?,
        adaptor.v_resolution(tolerance)?,
    );
    if translated == point {
        return Ok(false);
    }
    let product = if translated.x() == point.x() {
        -direction.x()
    } else {
        direction.y()
    };
    let translation = Vector2::new(translated.x() - point.x(), translated.y() - point.y());
    install_pair(
        &split,
        face,
        &curve.pcurve,
        range,
        translation,
        left == (product < 0.),
    )?;
    Ok(true)
}
fn surface_periods(
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
) -> Result<Option<SeamPeriods>, Failure> {
    let bounds = surface.bounds();
    let (u, v) = (surface.is_u_closed()?, surface.is_v_closed()?);
    if u || v {
        return Ok(Some(SeamPeriods {
            bounds,
            u: if u { bounds.1 - bounds.0 } else { 0. },
            v: if v { bounds.3 - bounds.2 } else { 0. },
        }));
    }
    let trimmed: Handle<GeomRectangularTrimmedSurface> = surface.cast();
    let Some(trimmed) = trimmed.get() else {
        return Ok(None);
    };
    trimmed_periods(&trimmed.basis_surface(), bounds, tolerance)
}
fn trimmed_periods(
    basis: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
    tolerance: f64,
) -> Result<Option<SeamPeriods>, Failure> {
    let (u, v) = (basis.is_u_periodic(), basis.is_v_periodic());
    if u || v {
        return Ok(Some(SeamPeriods {
            bounds,
            u: if u { basis.u_period()? } else { 0. },
            v: if v { basis.v_period()? } else { 0. },
        }));
    }
    let global = basis.bounds();
    let u = basis.is_u_closed()?
        && (bounds.0 - global.0).abs() < tolerance
        && (bounds.1 - global.1).abs() < tolerance;
    let v = basis.is_v_closed()?
        && (bounds.2 - global.2).abs() < tolerance
        && (bounds.3 - global.3).abs() < tolerance;
    if !u && !v {
        return Ok(None);
    }
    Ok(Some(SeamPeriods {
        bounds,
        u: if u { bounds.1 - bounds.0 } else { 0. },
        v: if v { bounds.3 - bounds.2 } else { 0. },
    }))
}
fn translated_point(point: Point2, periods: &SeamPeriods, du: f64, dv: f64) -> (Point2, bool) {
    let mut left = false;
    let u = shift_coordinate(
        point.x(),
        periods.bounds.0,
        periods.bounds.1,
        periods.u,
        du,
        &mut left,
    );
    let v = shift_coordinate(
        point.y(),
        periods.bounds.2,
        periods.bounds.3,
        periods.v,
        dv,
        &mut left,
    );
    (Point2::new(u, v), left)
}
fn shift_coordinate(
    value: f64,
    first: f64,
    last: f64,
    period: f64,
    tolerance: f64,
    left: &mut bool,
) -> f64 {
    if period > 0. {
        if (value - first).abs() < tolerance {
            *left = true;
            return value + period;
        }
        if (value - last).abs() < tolerance {
            *left = false;
            return value - period;
        }
    }
    value
}
