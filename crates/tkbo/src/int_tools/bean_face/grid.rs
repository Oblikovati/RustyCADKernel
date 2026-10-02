//! Knot sampling from IntTools_BeanFaceIntersector.cxx at OCCT 3d097a0.
use crate::int_tools::SurfaceRangeLocalizeData;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::{ncollection::Array1, standard::Failure};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom::GeomBSplineSurface};
#[path = "grid_points.rs"]
mod points;

pub(super) fn compute_grid(
    surface: &BRepAdaptorSurface,
    spline: &GeomBSplineSurface,
    bounds: [f64; 4],
    tolerance: f64,
    cache: &mut SurfaceRangeLocalizeData,
) -> Result<(), Failure> {
    let degrees = [surface.u_degree()?, surface.v_degree()?];
    let counts = [surface.nb_u_knots()?, surface.nb_v_knots()?];
    let knots = [spline.u_knots(), spline.v_knots()];
    for axis in 0..2 {
        if !process_axis(
            cache,
            axis,
            &knots[axis],
            counts[axis],
            degrees[axis],
            bounds,
            tolerance,
        )? {
            return Ok(());
        }
    }
    points::compute_points(surface, tolerance, cache)
}
fn process_axis(
    cache: &mut SurfaceRangeLocalizeData,
    axis: usize,
    knots: &Array1<'_, f64>,
    count: i32,
    degree: i32,
    bounds: [f64; 4],
    tolerance: f64,
) -> Result<bool, Failure> {
    let axis_bounds = [bounds[axis * 2], bounds[axis * 2 + 1]];
    let Some(samples) = sample_axis(knots, count, degree, axis_bounds, tolerance)? else {
        return Ok(false);
    };
    store_parameters(cache, axis, &samples)?;
    Ok(true)
}
fn sample_axis(
    knots: &Array1<'_, f64>,
    count: i32,
    degree: i32,
    bounds: [f64; 2],
    tolerance: f64,
) -> Result<Option<Vec<f64>>, Failure> {
    let Some(indices) = knot_indices(knots, count, bounds, tolerance)? else {
        return Ok(None);
    };
    Ok(Some(axis_parameters(
        knots, indices, degree, bounds, tolerance,
    )?))
}
fn knot_indices(
    knots: &Array1<'_, f64>,
    count: i32,
    bounds: [f64; 2],
    tolerance: f64,
) -> Result<Option<[i32; 2]>, Failure> {
    let mut indices = [-1, -1];
    for i in 1..=count {
        if indices[0] != -1 && indices[1] != -1 {
            break;
        }
        if indices[0] == -1 && bounds[0] + tolerance < knot(knots, i)? {
            indices[0] = i - 1;
        }
        let reverse = count - i + 1;
        if indices[1] == -1 && bounds[1] - tolerance > knot(knots, reverse)? {
            indices[1] = reverse + 1;
        }
    }
    Ok(normalize_indices(indices, count))
}
fn normalize_indices(mut indices: [i32; 2], count: i32) -> Option<[i32; 2]> {
    if indices[0] == -1 || indices[0] == 0 {
        indices[0] = 1;
    }
    if indices[1] == -1 || indices[1] > count {
        indices[1] = count;
    }
    if indices[1] < indices[0] {
        return None;
    }
    if indices[1] == indices[0] {
        indices = [(indices[0] - 1).max(1), (indices[1] + 1).min(count)];
    }
    Some(indices)
}
fn knot(knots: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    knots
        .value(index)
        .copied()
        .map_err(|e| e.into_failure("ComputeGridPoints knot"))
}
fn axis_parameters(
    knots: &Array1<'_, f64>,
    indices: [i32; 2],
    degree: i32,
    bounds: [f64; 2],
    tolerance: f64,
) -> Result<Vec<f64>, Failure> {
    let mut samples = Vec::new();
    for i in indices[0]..indices[1] {
        let interval = interval_bounds(knots, indices, i, bounds, tolerance)?;
        append_interval(&mut samples, interval, degree);
    }
    let maximum = last_parameter(knots, indices, bounds, tolerance)?;
    samples.push(maximum);
    Ok(samples)
}
fn interval_bounds(
    knots: &Array1<'_, f64>,
    indices: [i32; 2],
    index: i32,
    bounds: [f64; 2],
    tolerance: f64,
) -> Result<[f64; 2], Failure> {
    let mut minimum = knot(knots, index)?;
    if index == indices[0] && bounds[0] - tolerance > minimum {
        minimum = bounds[0];
    }
    let mut maximum = knot(knots, index + 1)?;
    if index == indices[1] - 1 && bounds[1] + tolerance < maximum {
        maximum = bounds[1];
    }
    Ok([minimum, maximum])
}
fn append_interval(samples: &mut Vec<f64>, [mut minimum, maximum]: [f64; 2], degree: i32) {
    let delta = (maximum - minimum) / f64::from(degree);
    for _ in 0..degree {
        samples.push(minimum);
        minimum += delta;
    }
}
fn last_parameter(
    knots: &Array1<'_, f64>,
    indices: [i32; 2],
    bounds: [f64; 2],
    tolerance: f64,
) -> Result<f64, Failure> {
    let mut maximum = bounds[1];
    let last_knot = knot(knots, indices[1])?;
    if bounds[1] + tolerance >= last_knot {
        maximum = last_knot;
    }
    Ok(maximum)
}
fn store_parameters(
    cache: &mut SurfaceRangeLocalizeData,
    axis: usize,
    samples: &[f64],
) -> Result<(), Failure> {
    let length = samples.len() as i32;
    if axis == 0 {
        cache.set_range_u_grid(length)
    } else {
        cache.set_range_v_grid(length)
    }
    .map_err(|e| e.into_failure("ComputeGridPoints grid extent"))?;
    for (offset, &value) in samples.iter().enumerate() {
        if axis == 0 {
            cache.set_u_param(offset as i32 + 1, value)
        } else {
            cache.set_v_param(offset as i32 + 1, value)
        }
        .map_err(|e| e.into_failure("ComputeGridPoints grid parameter"))?;
    }
    Ok(())
}
