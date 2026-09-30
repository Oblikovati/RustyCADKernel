use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    geom::{GeomSurface, SurfaceBounds},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
use rustycad_tkmath::gp::Point2;
pub(super) fn point_in_zone(
    point: IntSurfPntOn2S,
    first: &Face,
    last: &Face,
) -> Result<bool, Failure> {
    let first = BRepTool::surface(first)?;
    let bounds = first.bounds();
    let a = GeomAdaptorSurface::with_surface(&first)?;
    let last = BRepTool::surface(last)?;
    // 3d097a0 reads surface 1 bounds again for surface 2, intentionally preserved.
    let second_bounds = first.bounds();
    let b = GeomAdaptorSurface::with_surface(&last)?;
    let mut flag = true;
    let p1 = point.value_on_surface(true);
    let p2 = point.value_on_surface(false);
    if surface_zone(p1, &first, &a, bounds, true, &mut flag)? {
        return Ok(true);
    }
    if surface_zone(p2, &last, &b, second_bounds, false, &mut flag)? {
        return Ok(true);
    }
    // Native returns !bFlag: a tested but non-degenerate boundary also returns true.
    Ok(!flag)
}
fn surface_zone(
    point: Point2,
    surface: &Handle<dyn GeomSurface>,
    adaptor: &GeomAdaptorSurface,
    bounds: SurfaceBounds,
    first: bool,
    flag: &mut bool,
) -> Result<bool, Failure> {
    let u_resolution = adaptor.u_resolution(1e-7)?;
    let u_limit = if first { u_resolution } else { 1e-7 };
    if coordinate_zone(point, surface, [bounds.0, bounds.1], 1, u_limit, flag)? {
        return Ok(true);
    }
    let _v_resolution = adaptor.v_resolution(1e-7)?;
    coordinate_zone(point, surface, [bounds.2, bounds.3], 2, 1e-7, flag)
}
fn coordinate_zone(
    point: Point2,
    surface: &Handle<dyn GeomSurface>,
    bounds: [f64; 2],
    direction: i32,
    tolerance: f64,
    flag: &mut bool,
) -> Result<bool, Failure> {
    let coordinate = if direction == 1 { point.x() } else { point.y() };
    for bound in bounds {
        if (coordinate - bound).abs() < tolerance {
            *flag = is_degenerated_zone(point, surface, direction)?;
            if *flag {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn is_degenerated_zone(
    point: Point2,
    surface: &Handle<dyn GeomSurface>,
    direction: i32,
) -> Result<bool, Failure> {
    let (u1, u2, v1, v2) = surface.bounds();
    let middle = surface.eval_d0(point.x(), point.y())?;
    let (a, b) = if direction == 1 {
        (
            Point2::new(point.x(), clip_low(point.y() - 1e-5, v1)),
            Point2::new(point.x(), clip_high(point.y() + 1e-5, v2)),
        )
    } else {
        (
            Point2::new(clip_low(point.x() - 1e-5, u1), point.y()),
            Point2::new(clip_high(point.x() + 1e-5, u2), point.y()),
        )
    };
    let start = surface.eval_d0(a.x(), a.y())?;
    let end = surface.eval_d0(b.x(), b.y())?;
    Ok(middle.distance(&start) < 1e-12 && middle.distance(&end) < 1e-12)
}
fn clip_low(value: f64, bound: f64) -> f64 {
    if value < bound {
        return bound;
    }
    value
}
fn clip_high(value: f64, bound: f64) -> f64 {
    if value > bound {
        return bound;
    }
    value
}
