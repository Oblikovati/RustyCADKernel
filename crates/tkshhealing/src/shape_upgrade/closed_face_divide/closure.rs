use super::algorithm::{interior_values, split_values};
use crate::shape_analysis::ShapeAnalysisSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomRectangularTrimmedSurface, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use std::cmp::Ordering;

type ParameterRange = (f64, f64);

struct AxisIntervals {
    natural: ParameterRange,
    face: ParameterRange,
    tolerance: f64,
}

pub(super) fn closure_cuts(
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
    split_points: i32,
    tolerance: f64,
) -> Result<Option<(bool, super::super::SurfaceSplitValues)>, Failure> {
    let (u_closed, v_closed) = closed_directions(surface, tolerance)?;
    if let Some(values) =
        closure_axis_values(surface, bounds, split_points, u_closed, true, tolerance)?
    {
        return Ok(Some((true, split_values(&values))));
    }
    if let Some(values) =
        closure_axis_values(surface, bounds, split_points, v_closed, false, tolerance)?
    {
        return Ok(Some((false, split_values(&values))));
    }
    Ok(None)
}

fn closed_directions(
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
) -> Result<(bool, bool), Failure> {
    let analysis = ShapeAnalysisSurface::new(surface)?;
    Ok((
        analysis.is_u_closed(tolerance)?,
        analysis.is_v_closed(tolerance)?,
    ))
}

fn closure_axis_values(
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
    split_points: i32,
    closed: bool,
    u_direction: bool,
    tolerance: f64,
) -> Result<Option<Vec<f64>>, Failure> {
    if !closed {
        return Ok(None);
    }
    if !divides_range(surface, bounds, u_direction, tolerance)? {
        return Ok(None);
    }
    let interval = axis_bounds(bounds, u_direction);
    Ok(Some(interior_values(interval, split_points)))
}

fn axis_bounds(bounds: (f64, f64, f64, f64), u_direction: bool) -> (f64, f64) {
    if u_direction {
        (bounds.0, bounds.1)
    } else {
        (bounds.2, bounds.3)
    }
}

fn divides_range(
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
    u_direction: bool,
    tolerance: f64,
) -> Result<bool, Failure> {
    let natural = surface.get().ok_or_else(null_surface)?.bounds();
    let adaptor = GeomAdaptorSurface::with_surface(surface)?;
    let intervals = resolution_intervals(&adaptor, natural, bounds, u_direction, tolerance)?;
    let gap = range_length(intervals.natural) - range_length(intervals.face);
    if gap.partial_cmp(&intervals.tolerance) != Some(Ordering::Less) {
        return Ok(false);
    }
    let middle = (intervals.natural.0 + intervals.natural.1) / 2.0;
    Ok(!trimmed_closure(
        surface,
        (intervals.natural.0, middle),
        u_direction,
        tolerance,
    )?)
}

fn range_length(range: ParameterRange) -> f64 {
    range.1 - range.0
}

fn resolution_intervals(
    adaptor: &GeomAdaptorSurface,
    natural: (f64, f64, f64, f64),
    bounds: (f64, f64, f64, f64),
    u_direction: bool,
    tolerance: f64,
) -> Result<AxisIntervals, Failure> {
    let tolerance = if u_direction {
        adaptor.u_resolution(tolerance)?
    } else {
        adaptor.v_resolution(tolerance)?
    };
    let natural_axis = axis_bounds(natural, u_direction);
    let face_axis = axis_bounds(bounds, u_direction);
    Ok(AxisIntervals {
        natural: natural_axis,
        face: face_axis,
        tolerance,
    })
}

fn trimmed_closure(
    surface: &Handle<dyn GeomSurface>,
    interval: (f64, f64),
    u_direction: bool,
    tolerance: f64,
) -> Result<bool, Failure> {
    let trimmed = GeomRectangularTrimmedSurface::with_one_direction(
        surface,
        interval.0,
        interval.1,
        u_direction,
        true,
    )?;
    let trimmed: Handle<dyn GeomSurface> = Handle::new(trimmed).cast();
    let analysis = ShapeAnalysisSurface::new(&trimmed)?;
    if u_direction {
        analysis.is_u_closed(tolerance)
    } else {
        analysis.is_v_closed(tolerance)
    }
}

fn null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_ClosedFaceDivide: null surface; expected a live supporting surface",
        "",
    )
}
