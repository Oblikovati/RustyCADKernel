use super::*;
use rustycad_tkmath::bnd::BoundingBox3;

pub(super) fn compute_points(
    surface: &BRepAdaptorSurface,
    tolerance: f64,
    cache: &mut SurfaceRangeLocalizeData,
) -> Result<(), Failure> {
    let counts = [cache.get_range_u_grid(), cache.get_range_v_grid()];
    let estimate = counts[0] < 30 && counts[1] < 30;
    let mut boxes = [BoundingBox3::default(), BoundingBox3::default()];
    compute_rows(surface, cache, counts, estimate, &mut boxes)?;
    let deflection = grid_deflection(&boxes, estimate, tolerance)?;
    cache.set_grid_deflection(deflection);
    Ok(())
}
fn compute_rows(
    surface: &BRepAdaptorSurface,
    cache: &mut SurfaceRangeLocalizeData,
    counts: [i32; 2],
    estimate: bool,
    boxes: &mut [BoundingBox3; 2],
) -> Result<(), Failure> {
    for i in 1..=counts[0] {
        let u = cache
            .get_u_param(i)
            .map_err(|e| e.into_failure("ComputeGridPoints U"))?;
        let du = row_half_delta(cache, i, counts[0], u, estimate)?;
        compute_row(
            surface,
            cache,
            [i, counts[0], counts[1]],
            [u, du],
            estimate,
            boxes,
        )?;
    }
    Ok(())
}
fn row_half_delta(
    cache: &SurfaceRangeLocalizeData,
    index: i32,
    count: i32,
    parameter: f64,
    estimate: bool,
) -> Result<f64, Failure> {
    if !estimate || index >= count {
        return Ok(0.);
    }
    let next = cache
        .get_u_param(index + 1)
        .map_err(|e| e.into_failure("ComputeGridPoints next U"))?;
    Ok(0.5 * (next - parameter))
}
fn grid_deflection(
    boxes: &[BoundingBox3; 2],
    estimate: bool,
    tolerance: f64,
) -> Result<f64, Failure> {
    let mut deflection = if estimate { box_deflection(boxes)? } else { 0. };
    if tolerance > deflection {
        deflection = 2. * tolerance;
    }
    Ok(deflection)
}
fn compute_row(
    surface: &BRepAdaptorSurface,
    cache: &mut SurfaceRangeLocalizeData,
    indices: [i32; 3],
    params: [f64; 2],
    estimate: bool,
    boxes: &mut [BoundingBox3; 2],
) -> Result<(), Failure> {
    let [i, nu, nv] = indices;
    for j in 1..=nv {
        let v = cache
            .get_v_param(j)
            .map_err(|e| e.into_failure("ComputeGridPoints V"))?;
        let point = compute_point(
            surface,
            cache,
            [i, nu, j, nv],
            [params[0], params[1], v],
            estimate,
            boxes,
        )?;
        cache
            .set_grid_point(i, j, point)
            .map_err(|e| e.into_failure("ComputeGridPoints point"))?;
    }
    Ok(())
}
fn compute_point(
    surface: &BRepAdaptorSurface,
    cache: &SurfaceRangeLocalizeData,
    indices: [i32; 4],
    parameters: [f64; 3],
    estimate: bool,
    boxes: &mut [BoundingBox3; 2],
) -> Result<rustycad_tkmath::gp::Point3, Failure> {
    let [i, nu, j, nv] = indices;
    let [u, du, v] = parameters;
    if !estimate {
        return surface.d0(u, v);
    }
    let derivatives = surface.d1(u, v)?;
    boxes[0].add_point(derivatives.point);
    if i < nu && j < nv {
        extend_box(cache, j, v, du, derivatives, &mut boxes[1])?;
    }
    Ok(derivatives.point)
}
fn extend_box(
    cache: &SurfaceRangeLocalizeData,
    j: i32,
    v: f64,
    du: f64,
    derivatives: rustycad_tkg3d::geom::GeomSurfaceD1,
    output: &mut BoundingBox3,
) -> Result<(), Failure> {
    let dv = 0.5
        * (cache
            .get_v_param(j + 1)
            .map_err(|e| e.into_failure("ComputeGridPoints next V"))?
            - v);
    let shift = derivatives.d1u * du + derivatives.d1v * dv;
    output.add_point(derivatives.point.translated(shift));
    Ok(())
}
fn box_deflection(boxes: &[BoundingBox3; 2]) -> Result<f64, Failure> {
    let grid = boxes[0].get()?;
    let extended = boxes[1].get()?;
    let (count, mut deflection) = outside_extent(&grid, &extended);
    if count < 3 {
        deflection /= 2.;
    }
    Ok(deflection)
}
fn outside_extent(grid: &[f64; 6], extended: &[f64; 6]) -> (i32, f64) {
    let mut count = 0;
    let mut deflection = 0.;
    for index in 0..6 {
        if is_extended(index, grid, extended) {
            let delta = extent_delta(index, grid, extended);
            deflection = rustycad_tkg2d::geom2d::native::std_max(delta, deflection);
            count += 1;
        }
    }
    (count, deflection)
}
fn is_extended(index: usize, grid: &[f64; 6], extended: &[f64; 6]) -> bool {
    if index < 3 {
        return extended[index] < grid[index];
    }
    extended[index] > grid[index]
}
fn extent_delta(index: usize, grid: &[f64; 6], extended: &[f64; 6]) -> f64 {
    if index < 3 {
        return grid[index] - extended[index];
    }
    extended[index] - grid[index]
}
